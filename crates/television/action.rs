use serde::{Deserialize, Serialize};
use strum::Display;

/// The different actions that can be performed by the application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display)]
pub enum Action {
    // input actions
    /// Add a character to the input buffer.
    AddInputChar(char),
    /// Delete the character before the cursor from the input buffer.
    DeletePrevChar,
    /// Delete the character after the cursor from the input buffer.
    DeleteNextChar,
    /// Move the cursor to the character before the current cursor position.
    GoToPrevChar,
    /// Move the cursor to the character after the current cursor position.
    GoToNextChar,
    /// Move the cursor to the start of the input buffer.
    GoToInputStart,
    /// Move the cursor to the end of the input buffer.
    GoToInputEnd,
    // rendering actions
    /// Render the terminal user interface screen.
    Render,
    /// Resize the terminal user interface screen to the given dimensions.
    Resize(u16, u16),
    /// Clear the terminal user interface screen.
    ClearScreen,
    // results actions
    /// Select the entry currently under the cursor.
    SelectEntry,
    /// Select the entry currently under the cursor and exit the application.
    SelectAndExit,
    /// Select the next entry in the currently focused list.
    SelectNextEntry,
    /// Select the previous entry in the currently focused list.
    SelectPrevEntry,
    /// Copy the currently selected entry to the clipboard.
    CopyEntryToClipboard,
    // preview actions
    /// Scroll the preview up by one line.
    ScrollPreviewUp,
    /// Scroll the preview down by one line.
    ScrollPreviewDown,
    /// Scroll the preview up by half a page.
    ScrollPreviewHalfPageUp,
    /// Scroll the preview down by half a page.
    ScrollPreviewHalfPageDown,
    /// Open the currently selected entry in the default application.
    OpenEntry,
    // application actions
    /// Tick the application state.
    Tick,
    /// Suspend the application.
    Suspend,
    /// Resume the application.
    Resume,
    /// Quit the application.
    Quit,
    /// Toggle the help screen.
    Help,
    /// Signal an error with the given message.
    Error(String),
    /// No operation.
    NoOp,
    // channel actions
    /// Toggle the remote control channel.
    ToggleRemoteControl,
    /// Toggle the remote control in `send to channel` mode.
    ToggleSendToChannel,
}
