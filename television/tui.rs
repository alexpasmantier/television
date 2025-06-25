use std::{
    io::{LineWriter, Write, stderr},
    ops::{Deref, DerefMut},
};

use anyhow::Result;
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode, is_raw_mode_enabled,
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
}

#[allow(dead_code)]
impl<W> Tui<W>
where
    W: Write,
{
    pub fn new(writer: W) -> Result<Self> {
        Ok(Self {
            terminal: ratatui::Terminal::new(CrosstermBackend::new(writer))?,
        })
    }

    pub fn size(&self) -> Result<Size> {
        Ok(self.terminal.size()?)
    }

    pub fn enter(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut buffered_stderr = LineWriter::new(stderr());
        execute!(buffered_stderr, EnterAlternateScreen)?;
        execute!(buffered_stderr, EnableMouseCapture)?;
        self.terminal.clear()?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        if is_raw_mode_enabled()? {
            debug!("Exiting terminal");

            disable_raw_mode()?;
            let mut buffered_stderr = LineWriter::new(stderr());
            execute!(buffered_stderr, cursor::Show)?;
            execute!(buffered_stderr, DisableMouseCapture)?;
            execute!(buffered_stderr, LeaveAlternateScreen)?;
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
