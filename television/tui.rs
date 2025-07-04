use std::{
    io::Write,
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
    Terminal, TerminalOptions, Viewport, backend::CrosstermBackend,
    layout::Size,
};
use tracing::debug;

#[allow(dead_code)]
pub struct Tui<W>
where
    W: Write,
{
    pub terminal: ratatui::Terminal<CrosstermBackend<W>>,
    pub viewport: Viewport,
}

pub const TESTING_ENV_VAR: &str = "TV_TEST";

#[allow(dead_code)]
impl<W> Tui<W>
where
    W: Write,
{
    pub fn new(
        writer: W,
        height: Option<u16>,
        width: Option<u16>,
    ) -> Result<Self> {
        let mut backend = CrosstermBackend::new(writer);
        let mut options = TerminalOptions::default();

        match (width, height) {
            (None, None) => {
                options.viewport = Viewport::Fullscreen;
            }
            (None, Some(h)) => {
                options.viewport = Viewport::Inline(h);
            }
            (Some(w), Some(h)) => {
                // get cursor position
                let mut cursor_pos = Self::cursor_position()?;
                let term_size = crossterm::terminal::size()?;
                // scroll if we don't have enough space
                if cursor_pos.1 + h > term_size.1 {
                    execute!(
                        backend,
                        ScrollUp(cursor_pos.1 + h - term_size.1)
                    )?;
                    cursor_pos.1 = term_size.1.saturating_sub(h);
                }
                options.viewport =
                    Viewport::Fixed(ratatui::layout::Rect::new(
                        cursor_pos.0,
                        cursor_pos.1,
                        w.min(term_size.0 - cursor_pos.0),
                        h.min(term_size.1 - cursor_pos.1),
                    ));
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "TUI viewport: Width cannot be set without a given height.",
                ));
            }
        }

        let viewport = options.viewport.clone();
        let terminal = Terminal::with_options(backend, options)?;
        Ok(Self { terminal, viewport })
    }

    pub fn cursor_position() -> Result<(u16, u16)> {
        if std::env::var(TESTING_ENV_VAR).is_ok() {
            // For testing purposes, return a fixed position
            return Ok((0, 0));
        }
        crossterm::cursor::position().map_err(|e| {
            anyhow::anyhow!("Failed to get cursor position: {}", e)
        })
    }

    pub fn size(&self) -> Result<Size> {
        Ok(self.terminal.size()?)
    }

    pub fn resize_viewport(&mut self, w: u16, h: u16) -> Result<()> {
        debug!("Resizing viewport to: {:?}", (w, h));
        let layout = match self.viewport {
            Viewport::Fullscreen | Viewport::Inline(_) => {
                ratatui::layout::Rect::new(0, 0, w, h)
            }
            Viewport::Fixed(rect) => ratatui::layout::Rect::new(
                rect.x,
                rect.y,
                w.min(rect.width),
                h.min(rect.height),
            ),
        };

        self.resize(layout)?;
        Ok(())
    }

    pub fn enter(&mut self) -> Result<()> {
        enable_raw_mode()?;
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

            // Move cursor up one line to avoid leaving artefacts on the top border
            execute!(backend, cursor::MoveToPreviousLine(1))?;
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
