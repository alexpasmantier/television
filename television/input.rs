use crate::action::Action;
use crate::utils::input::InputRequest;

pub fn convert_action_to_input_request(
    action: &Action,
) -> Option<InputRequest> {
    match action {
        Action::AddInputChar(c) => Some(InputRequest::InsertChar(*c)),
        Action::DeletePrevChar => Some(InputRequest::DeletePrevChar),
        Action::DeletePrevWord => Some(InputRequest::DeletePrevWord),
        Action::DeleteNextChar => Some(InputRequest::DeleteNextChar),
        Action::GoToPrevChar => Some(InputRequest::GoToPrevChar),
        Action::GoToNextChar => Some(InputRequest::GoToNextChar),
        Action::GoToInputStart => Some(InputRequest::GoToStart),
        Action::GoToInputEnd => Some(InputRequest::GoToEnd),
        _ => None,
    }
}
