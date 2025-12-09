use crossterm::event::{MouseEvent, MouseEventKind};
use ratatui::layout::Position;

use crate::{action::Action, screen::layout::Layout, television::Mode};

/// Handles mouse events (scrolling) and returns the corresponding action
/// based on the cursor position, the current UI layout and mode.
pub fn get_action_for_mouse_event(
    event: MouseEvent,
    ui_layout: &Layout,
    mode: Mode,
) -> Action {
    let position = Position::new(event.column, event.row);

    // if the mouse is over the results or remote control, scroll the selection
    if matches!(mode, Mode::Channel) && ui_layout.results.contains(position)
        || matches!(mode, Mode::RemoteControl)
            && ui_layout
                .remote_control
                .is_some_and(|rc| rc.contains(position))
    {
        match event.kind {
            MouseEventKind::ScrollUp => return Action::SelectPrevEntry,
            MouseEventKind::ScrollDown => return Action::SelectNextEntry,
            _ => return Action::NoOp,
        }
    // if the mouse is over the preview window in channel mode, scroll the preview
    } else if matches!(mode, Mode::Channel)
        && ui_layout
            .preview_window
            .is_some_and(|preview| preview.contains(position))
    {
        match event.kind {
            MouseEventKind::ScrollUp => return Action::ScrollPreviewUp,
            MouseEventKind::ScrollDown => return Action::ScrollPreviewDown,
            _ => return Action::NoOp,
        }
    }
    Action::NoOp
}
