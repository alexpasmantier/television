use television_screen::input::{Input, InputRequest, StateChanged};
use tv::action::Action;

/// This makes the `Action` type compatible with the `Input` logic.
pub trait InputActionHandler {
    // Handle Key event.
    fn handle_action(&mut self, action: &Action) -> Option<StateChanged>;
}

impl InputActionHandler for Input {
    /// Handle Key event.
    fn handle_action(&mut self, action: &Action) -> Option<StateChanged> {
        match action {
            Action::AddInputChar(c) => {
                self.handle(InputRequest::InsertChar(*c))
            }
            Action::DeletePrevChar => {
                self.handle(InputRequest::DeletePrevChar)
            }
            Action::DeleteNextChar => {
                self.handle(InputRequest::DeleteNextChar)
            }
            Action::GoToPrevChar => self.handle(InputRequest::GoToPrevChar),
            Action::GoToNextChar => self.handle(InputRequest::GoToNextChar),
            Action::GoToInputStart => self.handle(InputRequest::GoToStart),
            Action::GoToInputEnd => self.handle(InputRequest::GoToEnd),
            _ => None,
        }
    }
}
