use std::{
    io::{stderr, LineWriter, Write},
    ops::{Deref, DerefMut},
};

use color_eyre::Result;
use crossterm::{
    cursor, execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, is_raw_mode_enabled,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use ratatui::{backend::CrosstermBackend, layout::Size};
use tokio::task::JoinHandle;
use tracing::debug;

pub struct Tui<W>
where
    W: Write,
{
    pub task: JoinHandle<()>,
    pub frame_rate: f64,
    pub terminal: ratatui::Terminal<CrosstermBackend<W>>,
}

impl<W> Tui<W>
where
    W: Write,
{
    pub fn new(writer: W) -> Result<Self> {
        Ok(Self {
            task: tokio::spawn(async {}),
            frame_rate: 60.0,
            terminal: ratatui::Terminal::new(CrosstermBackend::new(writer))?,
        })
    }

    pub fn frame_rate(mut self, frame_rate: f64) -> Self {
        self.frame_rate = frame_rate;
        self
    }

    pub fn size(&self) -> Result<Size> {
        Ok(self.terminal.size()?)
    }

    pub fn enter(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut buffered_stderr = LineWriter::new(stderr());
        execute!(buffered_stderr, EnterAlternateScreen)?;
        self.terminal.clear()?;
        execute!(buffered_stderr, cursor::Hide)?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        if is_raw_mode_enabled()? {
            debug!("Exiting terminal");

            disable_raw_mode()?;
            let mut buffered_stderr = LineWriter::new(stderr());
            execute!(buffered_stderr, cursor::Show)?;
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
            Ok(_) => debug!("Successfully exited terminal"),
            Err(e) => debug!("Failed to exit terminal: {:?}", e),
        }
    }
}
