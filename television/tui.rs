use std::{
    io::{LineWriter, StdoutLock, Write, stderr, stdout},
    ops::{Deref, DerefMut},
};

use anyhow::Result;
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{
        ClearType, EnterAlternateScreen, LeaveAlternateScreen,
        disable_raw_mode, enable_raw_mode, is_raw_mode_enabled,
    },
};
use ratatui::{backend::CrosstermBackend, layout::Size};
use tracing::debug;

#[allow(dead_code)]
pub struct Tui<W>
where
    W: Write,
{
    pub terminal: ratatui::Terminal<CrosstermBackend<W>>,
    pub fullscreen: bool,
    pub height: Option<u16>,
    /// Row (0-based) where the overlay begins when running in non-fullscreen
    /// mode. Defaults to 0 in fullscreen.
    base_row: u16,
}

pub const TESTING_ENV_VAR: &str = "TV_TEST";

#[allow(dead_code)]
impl<W> Tui<W>
where
    W: Write,
{
    pub fn new(writer: W, height: Option<u16>) -> Result<Self> {
        let fullscreen = height.is_none();
        Ok(Self {
            terminal: ratatui::Terminal::new(CrosstermBackend::new(writer))?,
            fullscreen,
            height,
            base_row: 0,
        })
    }

    pub fn size(&self) -> Result<Size> {
        Ok(self.terminal.size()?)
    }

    pub fn enter(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut buffered_stderr = LineWriter::new(stderr());

        if self.fullscreen {
            execute!(buffered_stderr, EnterAlternateScreen)?;
            self.terminal.clear()?;
        } else {
            // Detect if we're in a testing environment
            // FIXME: find a better way to do this
            let is_testing = std::env::var(TESTING_ENV_VAR).is_ok();
            if is_testing {
                // Simplified approach for testing overlay mode
                // This avoids cursor positioning issues that interfere with pty testing
                execute!(buffered_stderr, EnableMouseCapture)?;
                self.terminal.clear()?;
                self.base_row = 0;
            } else {
                self.init_overlay()?;
            }
        }

        execute!(buffered_stderr, EnableMouseCapture)?;

        Ok(())
    }

    pub fn init_overlay(&mut self) -> Result<()> {
        let ui_height = self
            .height
            .expect("`height` should be set when not in fullscreen mode")
            .min(self.terminal.size()?.height);

        // print `ui_height` new-lines on stdout – this may cause scroll
        {
            let mut out: StdoutLock<'_> = stdout().lock();
            for _ in 0..ui_height {
                writeln!(out)?;
            }
            out.flush()?;
        }

        // move cursor back up `ui_height` rows so we can draw overlay.
        let mut b = LineWriter::new(stderr());
        execute!(b, cursor::MoveUp(ui_height))?;
        execute!(b, cursor::SavePosition)?;

        // record the row where overlay starts (after move-up)
        let (_, row_after_up) = cursor::position()?;
        self.base_row = row_after_up;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        if is_raw_mode_enabled()? {
            debug!("Exiting terminal");

            let mut buffered_stderr = LineWriter::new(stderr());

            if !self.fullscreen {
                // Restore cursor to saved position, then clear overlay area (erase below)
                execute!(buffered_stderr, cursor::RestorePosition)?;
                execute!(
                    buffered_stderr,
                    crossterm::terminal::Clear(ClearType::FromCursorDown)
                )?;
            }

            disable_raw_mode()?;
            execute!(buffered_stderr, cursor::Show)?;
            execute!(buffered_stderr, DisableMouseCapture)?;

            if self.fullscreen {
                execute!(buffered_stderr, LeaveAlternateScreen)?;
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

    pub fn base_row(&self) -> u16 {
        self.base_row
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
