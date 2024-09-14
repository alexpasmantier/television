use ratatui::crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::CrosstermBackend;
use std::io::{stdout, Result};
use std::process::Command;
use tracing::info;

use crate::ui::input::backend::EventHandler;

use crate::app::{self, App};

use super::editor::get_default_editor;

type Terminal = ratatui::Terminal<CrosstermBackend<std::io::Stdout>>;

pub fn handle_key_event(key: KeyEvent, app: &mut App, terminal: &mut Terminal) {
    // global key bindings
    match key {
        // quitting the app
        KeyEvent {
            code: KeyCode::Char('q') | KeyCode::Esc,
            ..
        } => {
            app.should_quit = true;
        }
        // moving between blocks
        KeyEvent {
            code: KeyCode::Tab, ..
        } => {
            app.next_block();
        }
        KeyEvent {
            code: KeyCode::BackTab,
            ..
        } => {
            app.previous_block();
        }
        KeyEvent {
            code: KeyCode::Char('k') | KeyCode::Up,
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            app.move_to_block_on_top();
        }
        KeyEvent {
            code: KeyCode::Char('j') | KeyCode::Down,
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            app.move_to_block_below();
        }
        KeyEvent {
            code: KeyCode::Char('h') | KeyCode::Left,
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            app.move_to_block_left();
        }
        KeyEvent {
            code: KeyCode::Char('l') | KeyCode::Right,
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            app.move_to_block_right();
        }
        // moving between search results
        KeyEvent {
            code: KeyCode::Char('n'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            handle_results_key_event(
                KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                    ..key
                },
                app,
            );
        }
        KeyEvent {
            code: KeyCode::Char('p'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            handle_results_key_event(
                KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                    ..key
                },
                app,
            );
        }
        // scrolling preview
        KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            handle_preview_key_event(
                KeyEvent {
                    code: KeyCode::Char('d'),
                    modifiers: KeyModifiers::NONE,
                    ..key
                },
                app,
                terminal,
            );
        }
        KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            handle_preview_key_event(
                KeyEvent {
                    code: KeyCode::Char('u'),
                    modifiers: KeyModifiers::NONE,
                    ..key
                },
                app,
                terminal,
            );
        }
        _ => {}
    }

    // block specific key bindings
    match app.current_block {
        app::CurrentBlock::Search => {
            handle_search_key_event(key, app);
        }
        app::CurrentBlock::Results => {
            handle_results_key_event(key, app);
        }
        app::CurrentBlock::Preview => {
            handle_preview_key_event(key, app, terminal);
        }
    }
}

fn handle_search_key_event(key: KeyEvent, app: &mut App) {
    match key.code {
        _ => {
            app.input.handle_event(&Event::Key(key));
        }
    }
}

fn handle_results_key_event(key: KeyEvent, app: &mut App) {
    match key {
        KeyEvent {
            code: KeyCode::Up | KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            if let Some(selected_index) = app.results_list.state.selected() {
                app.results_list
                    .state
                    .select(Some((selected_index + 1) % app.results_list.results.len()));
            }
        }
        KeyEvent {
            code: KeyCode::Down | KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            if let Some(selected_index) = app.results_list.state.selected() {
                if selected_index == 0 {
                    app.results_list
                        .state
                        .select(Some(app.results_list.results.len() - 1));
                } else {
                    app.results_list.state.select_previous();
                }
            }
        }
        _ => {}
    }
    if app.results_list.state.selected().is_none() {
        app.results_list.state.select_first();
    }
}

fn handle_preview_key_event(key: KeyEvent, app: &mut App, terminal: &mut Terminal) {
    match key {
        KeyEvent {
            code: KeyCode::Up | KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.scroll_preview_up(1);
        }
        KeyEvent {
            code: KeyCode::Down | KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.scroll_preview_down(1);
        }
        KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.scroll_preview_down(20);
        }
        KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.scroll_preview_up(20);
        }
        KeyEvent {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            run_editor(terminal, app).unwrap();
        }
        _ => {}
    }
}

const VI: &str = "vi";
const VIM: &str = "vim";
const NVIM: &str = "nvim";

fn run_editor(terminal: &mut Terminal, app: &mut App) -> Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    if let Some(selected_result) = &app.selected_result {
        if let Ok(editor) = get_default_editor() {
            // TODO: maybe add extra logic for opening other editors at a specific line
            match editor.as_str() {
                VI | VIM | NVIM => {
                    Command::new(editor)
                        .arg(format!("+{}", selected_result.line_number))
                        .arg(selected_result.path.canonicalize().unwrap())
                        .status()?;
                }
                _ => {
                    Command::new(editor)
                        .arg(selected_result.path.canonicalize().unwrap())
                        .status()?;
                }
            }
        }
    }
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    terminal.clear()?;
    Ok(())
}
