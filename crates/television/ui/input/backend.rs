use super::{Input, InputRequest, StateChanged};
use ratatui::crossterm::event::{
    Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
};

/// Converts crossterm event into input requests.
/// TODO: make these keybindings configurable.
pub fn to_input_request(evt: &CrosstermEvent) -> Option<InputRequest> {
    use InputRequest::*;
    use KeyCode::*;
    match evt {
        CrosstermEvent::Key(KeyEvent {
            code,
            modifiers,
            kind,
            state: _,
        }) if *kind == KeyEventKind::Press => match (*code, *modifiers) {
            (Backspace, KeyModifiers::NONE) => Some(DeletePrevChar),
            (Delete, KeyModifiers::NONE) => Some(DeleteNextChar),
            (Tab, KeyModifiers::NONE) => None,
            (Left, KeyModifiers::NONE) => Some(GoToPrevChar),
            //(Left, KeyModifiers::CONTROL) => Some(GoToPrevWord),
            (Right, KeyModifiers::NONE) => Some(GoToNextChar),
            //(Right, KeyModifiers::CONTROL) => Some(GoToNextWord),
            //(Char('u'), KeyModifiers::CONTROL) => Some(DeleteLine),
            (Char('w'), KeyModifiers::CONTROL)
            | (Backspace, KeyModifiers::ALT) => Some(DeletePrevWord),
            (Delete, KeyModifiers::CONTROL) => Some(DeleteNextWord),
            //(Char('k'), KeyModifiers::CONTROL) => Some(DeleteTillEnd),
            (Char('a'), KeyModifiers::CONTROL)
            | (Home, KeyModifiers::NONE) => Some(GoToStart),
            (Char('e'), KeyModifiers::CONTROL) | (End, KeyModifiers::NONE) => {
                Some(GoToEnd)
            }
            (Char(c), KeyModifiers::NONE) => Some(InsertChar(c)),
            (Char(c), KeyModifiers::SHIFT) => Some(InsertChar(c)),
            (_, _) => None,
        },
        _ => None,
    }
}

/// Import this trait to implement `Input::handle_event()` for crossterm.
pub trait EventHandler {
    /// Handle crossterm event.
    fn handle_event(&mut self, evt: &CrosstermEvent) -> Option<StateChanged>;
}

impl EventHandler for Input {
    /// Handle crossterm event.
    fn handle_event(&mut self, evt: &CrosstermEvent) -> Option<StateChanged> {
        to_input_request(evt).and_then(|req| self.handle(req))
    }
}

#[cfg(test)]
mod tests {
    use ratatui::crossterm::event::{KeyEventKind, KeyEventState};

    use super::*;

    #[test]
    fn handle_tab() {
        let evt = CrosstermEvent::Key(KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        });

        let req = to_input_request(&evt);

        assert!(req.is_none());
    }
}
