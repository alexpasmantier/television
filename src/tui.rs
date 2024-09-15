use std::io::{stdout, Result, Stdout};
use std::panic::{set_hook, take_hook};

use crossterm::{event::*, execute, terminal::*};
use ratatui::prelude::*;

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn init() -> Result<Tui> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, DisableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

pub fn restore() -> Result<()> {
    execute!(stdout(), LeaveAlternateScreen, EnableMouseCapture)?;
    disable_raw_mode()?;
    Ok(())
}

pub fn init_panic_hook() {
    let original_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
        // intentionally ignore errors here since we're already in a panic
        let _ = restore();
        original_hook(panic_info);
    }));
}
