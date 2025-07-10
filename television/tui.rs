use std::{
    fs::OpenOptions,
    io::{BufReader, LineWriter, Read, Write, stderr, stdout},
    ops::{Deref, DerefMut},
};

use anyhow::Result;
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{
        ClearType, EnterAlternateScreen, LeaveAlternateScreen, ScrollUp,
        disable_raw_mode, enable_raw_mode, is_raw_mode_enabled,
    },
};
use ratatui::{
    Terminal, TerminalOptions, Viewport,
    backend::CrosstermBackend,
    layout::{Position, Size},
    prelude::Backend,
};
use tracing::debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TuiMode {
    Fullscreen,
    Inline,
    Fixed { width: Option<u16>, height: u16 },
}

#[derive(Debug, Clone)]
pub enum IoStream {
    Stdout,
    BufferedStderr,
}

impl IoStream {
    pub fn to_stream(&self) -> Box<dyn std::io::Write + Send> {
        match self {
            IoStream::Stdout => Box::new(stdout()),
            IoStream::BufferedStderr => Box::new(LineWriter::new(stderr())),
        }
    }
}

#[allow(dead_code)]
pub struct Tui<W>
where
    W: Write,
{
    pub terminal: ratatui::Terminal<CrosstermBackend<W>>,
    pub viewport: Viewport,
}

pub const TESTING_ENV_VAR: &str = "TV_TEST";
pub const MIN_VIEWPORT_HEIGHT: u16 = 15;

#[allow(dead_code)]
impl<W> Tui<W>
where
    W: Write,
{
    /// NOTE:
    /// We use ratatui's `Viewport::Fixed` to handle inline instead of the builtin
    /// `Viewport::Inline` because we need control over which stream is used to query the cursor
    /// position and the `Inline` viewport always uses `stdout` under the hood
    /// (<https://github.com/crossterm-rs/crossterm/blob/master/src/cursor/sys/unix.rs#L35-L36>)
    /// which makes ratatui (crossterm) panic when trying to read the cursor position from a stream
    /// that is not connected to a tty.
    ///
    /// This allows us to query the cursor position ourselves using `/dev/tty` and to not rely on
    /// crossterm's current implementation.
    ///
    /// More info: <https://github.com/crossterm-rs/crossterm/pull/957>
    pub fn new(writer: W, mode: &TuiMode) -> Result<Self> {
        let mut backend = CrosstermBackend::new(writer);
        let mut options = TerminalOptions::default();
        enable_raw_mode()?;

        let terminal_size = backend.size()?;
        let viewport = match mode {
            TuiMode::Fullscreen => Viewport::Fullscreen,
            TuiMode::Inline => {
                let cursor_position = Self::get_cursor_position();
                let cursor_position = Self::handle_viewport_scrolling(
                    &mut backend,
                    cursor_position,
                    terminal_size,
                    MIN_VIEWPORT_HEIGHT,
                )?;

                // Calculate final available height after potential scrolling
                let available_height = terminal_size
                    .height
                    .saturating_sub(cursor_position.y)
                    .max(MIN_VIEWPORT_HEIGHT);

                Viewport::Fixed(ratatui::layout::Rect::new(
                    0,
                    cursor_position.y,
                    terminal_size.width,
                    available_height,
                ))
            }
            TuiMode::Fixed { width, height } => {
                let cursor_position = Self::get_cursor_position();
                let cursor_position = Self::handle_viewport_scrolling(
                    &mut backend,
                    cursor_position,
                    terminal_size,
                    *height,
                )?;

                let w = width.unwrap_or(terminal_size.width);
                Viewport::Fixed(ratatui::layout::Rect::new(
                    0,
                    cursor_position.y,
                    w.min(terminal_size.width - cursor_position.x),
                    *height,
                ))
            }
        };

        options.viewport = viewport.clone();
        let terminal = Terminal::with_options(backend, options)?;
        Ok(Self { terminal, viewport })
    }

    /// Handles scrolling logic when there's insufficient space for the requested height.
    /// Returns the updated cursor position after scrolling.
    fn handle_viewport_scrolling(
        backend: &mut CrosstermBackend<W>,
        mut cursor_position: Position,
        terminal_size: Size,
        required_height: u16,
    ) -> Result<Position> {
        let available_height =
            terminal_size.height.saturating_sub(cursor_position.y);

        debug!(
            "Terminal height: {}, Available height: {}, cursor position: {:?}",
            terminal_size.height, available_height, cursor_position.y
        );

        // We need to add one to the required height to account for the cursor position.
        let required_height = required_height + 1;

        // If we don't have enough space for the required height we need to scroll up.
        if available_height < required_height {
            // Minus one to account for the cursor position.
            let scroll_amount = required_height - available_height - 1;

            // Special case: when we're at the very bottom (available_height == 1),
            // we need to scroll one less line to avoid creating an empty line
            // between the TUI and the prompt due to terminal cursor positioning.
            let actual_scroll = if available_height == 1 {
                scroll_amount - 1
            } else {
                scroll_amount
            };

            // Scroll up by as needed to reach the required height.
            debug!("Scrolling up by: {}", actual_scroll);
            execute!(backend, ScrollUp(actual_scroll))?;

            // Update cursor position to account for the scroll.
            debug!("New cursor position: {}", cursor_position.y);
            cursor_position.y =
                cursor_position.y.saturating_sub(scroll_amount);
        }

        Ok(cursor_position)
    }

    const DSR: &'static str = "\x1b[6n";
    /// This is manually implemented as a workaround to a [crossterm issue](https://github.com/crossterm-rs/crossterm/pull/957).
    ///
    /// See the `Tui::new` method for more details.
    fn get_cursor_position() -> Position {
        if std::env::var(TESTING_ENV_VAR).is_ok() {
            // In tests, return a fixed position
            return Position { x: 0, y: 0 };
        } else if cfg!(windows) {
            let position = crossterm::cursor::position()
                .expect("Failed to get cursor position on Windows");
            return Position {
                x: position.0,
                y: position.1,
            };
        }
        let mut tty = OpenOptions::new()
            .read(true)
            // .write(true)
            .append(true)
            .open("/dev/tty")
            .expect("Failed to open /dev/tty");

        writeln!(tty, "{}", Self::DSR).expect("Failed to write to /dev/tty");

        let mut response = Vec::new();
        for byte in BufReader::new(tty).bytes() {
            match byte {
                Ok(b'R') => break,       // End of response
                Ok(b'\x1B' | b'[') => {} // Ignore CSI sequences
                Ok(b) => response.push(b),
                Err(e) => panic!("Error reading from /dev/tty: {}", e),
            }
        }

        let pos = response.split(|e| *e == b';').collect::<Vec<_>>();
        if pos.len() == 2 {
            let x =
                String::from_utf8_lossy(pos[1]).parse::<u16>().unwrap_or(0);
            let y =
                String::from_utf8_lossy(pos[0]).parse::<u16>().unwrap_or(0);
            Position {
                x: x.saturating_sub(1),
                y: y.saturating_sub(1),
            } // Convert to zero-based index
        } else {
            Position::default() // Default position if parsing fails
        }
    }

    pub fn resize_viewport(&mut self, w: u16, h: u16) -> Result<()> {
        debug!("Resizing viewport to: {:?}", (w, h));
        // simpler implementation: just resize the terminal to the new size
        // yes, we lose the previous data from the terminal, but that's an
        // acceptable trade-off for the sake of simplicity
        let layout = ratatui::layout::Rect::new(0, 0, w, h);

        // Update the stored viewport and only if it's not fullscreen
        if self.viewport != Viewport::Fullscreen {
            self.viewport = Viewport::Fixed(layout);
        }
        self.terminal.resize(layout)?;

        Ok(())
    }

    pub fn enter(&mut self) -> Result<()> {
        let backend = self.terminal.backend_mut();

        execute!(backend, EnableMouseCapture)?;

        if self.viewport == Viewport::Fullscreen {
            execute!(backend, EnterAlternateScreen)?;
            self.terminal.clear()?;
        }
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        if is_raw_mode_enabled()? {
            debug!("Exiting terminal");
            let backend = self.terminal.backend_mut();

            disable_raw_mode()?;

            // Move cursor to the top of the application area.
            if let Viewport::Fixed(rect) = self.viewport {
                execute!(backend, cursor::MoveTo(0, rect.y))?;
            }

            execute!(
                backend,
                crossterm::terminal::Clear(ClearType::FromCursorDown)
            )?;

            execute!(backend, cursor::Show)?;
            execute!(backend, DisableMouseCapture)?;

            if self.viewport == Viewport::Fullscreen {
                execute!(backend, LeaveAlternateScreen)?;
            }
        }

        Ok(())
    }

    pub fn suspend(&mut self) -> Result<()> {
        self.exit()?;
        #[cfg(not(windows))]
        signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)?;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        self.enter()?;
        Ok(())
    }
}

impl<W> Deref for Tui<W>
where
    W: Write,
{
    type Target = ratatui::Terminal<CrosstermBackend<W>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl<W> DerefMut for Tui<W>
where
    W: Write,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl<W> Drop for Tui<W>
where
    W: Write,
{
    fn drop(&mut self) {
        match self.exit() {
            Ok(()) => debug!("Successfully exited terminal"),
            Err(e) => debug!("Failed to exit terminal: {:?}", e),
        }
    }
}
