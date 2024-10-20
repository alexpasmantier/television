use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display)]
pub enum Action {
    // input actions
    AddInputChar(char),
    DeletePrevChar,
    DeleteNextChar,
    GoToPrevChar,
    GoToNextChar,
    GoToInputStart,
    GoToInputEnd,
    // rendering actions
    Render,
    Resize(u16, u16),
    ClearScreen,
    // results actions
    SelectEntry,
    SelectAndExit,
    SelectNextEntry,
    SelectPrevEntry,
    // navigation actions
    GoToPaneUp,
    GoToPaneDown,
    GoToPaneLeft,
    GoToPaneRight,
    GoToNextPane,
    GoToPrevPane,
    // preview actions
    ScrollPreviewUp,
    ScrollPreviewDown,
    ScrollPreviewHalfPageUp,
    ScrollPreviewHalfPageDown,
    OpenEntry,
    // application actions
    Tick,
    Suspend,
    Resume,
    Quit,
    Help,
    Error(String),
    NoOp,
    // channel actions
    ToggleChannelSelection,
    SendToChannel,
}
