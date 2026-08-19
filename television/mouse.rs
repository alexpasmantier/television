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

    // if the mouse is over the results list (which the remote control takes
    // over in remote mode), scroll the selection
    if matches!(mode, Mode::Channel | Mode::RemoteControl)
        && ui_layout.results.contains(position)
    {
        match event.kind {
            MouseEventKind::ScrollUp => return Action::SelectPrevEntry,
            MouseEventKind::ScrollDown => return Action::SelectNextEntry,
            _ => return Action::NoOp,
        }
    // if the mouse is over the preview window in channel mode — or over the
    // help panel, which borrows the preview pane and receives the preview
    // scroll actions while it's open — scroll it
    } else if (matches!(mode, Mode::Channel)
        && ui_layout
            .preview_window
            .is_some_and(|preview| preview.contains(position)))
        || ui_layout
            .help_panel
            .is_some_and(|help| help.contains(position))
    {
        match event.kind {
            MouseEventKind::ScrollUp => return Action::ScrollPreviewUp,
            MouseEventKind::ScrollDown => return Action::ScrollPreviewDown,
            _ => return Action::NoOp,
        }
    }
    Action::NoOp
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;
    use ratatui::layout::Rect;

    fn scroll_event(kind: MouseEventKind, x: u16, y: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column: x,
            row: y,
            modifiers: KeyModifiers::empty(),
        }
    }

    #[test]
    fn scrolling_over_the_help_panel_scrolls_it() {
        // the help panel borrows the preview pane: `preview_window` is None
        // and `help_panel` holds the pane rect
        let layout = Layout::new(
            Rect::new(0, 0, 50, 30),
            Rect::new(0, 30, 100, 1),
            None,
            None,
            Some(Rect::new(50, 0, 50, 30)),
            None,
        );

        assert_eq!(
            get_action_for_mouse_event(
                scroll_event(MouseEventKind::ScrollDown, 75, 10),
                &layout,
                Mode::Channel,
            ),
            Action::ScrollPreviewDown
        );
        assert_eq!(
            get_action_for_mouse_event(
                scroll_event(MouseEventKind::ScrollUp, 75, 10),
                &layout,
                Mode::Channel,
            ),
            Action::ScrollPreviewUp
        );
        // over the results list, scrolling still moves the selection
        assert_eq!(
            get_action_for_mouse_event(
                scroll_event(MouseEventKind::ScrollDown, 10, 10),
                &layout,
                Mode::Channel,
            ),
            Action::SelectNextEntry
        );
    }
}
