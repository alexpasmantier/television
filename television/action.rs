use serde::{Deserialize, Serialize};

/// The different actions that can be performed by the application.
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    // input actions
    /// Add a character to the input buffer.
    #[serde(skip)]
    AddInputChar(char),
    /// Delete the character before the cursor from the input buffer.
    #[serde(skip)]
    DeletePrevChar,
    /// Delete the previous word from the input buffer.
    #[serde(skip)]
    DeletePrevWord,
    /// Delete the character after the cursor from the input buffer.
    #[serde(skip)]
    DeleteNextChar,
    /// Delete the current line from the input buffer.
    #[serde(skip)]
    DeleteLine,
    /// Move the cursor to the character before the current cursor position.
    #[serde(skip)]
    GoToPrevChar,
    /// Move the cursor to the character after the current cursor position.
    #[serde(skip)]
    GoToNextChar,
    /// Move the cursor to the start of the input buffer.
    GoToInputStart,
    /// Move the cursor to the end of the input buffer.
    GoToInputEnd,
    // rendering actions
    /// Render the terminal user interface screen.
    #[serde(skip)]
    Render,
    /// Resize the terminal user interface screen to the given dimensions.
    #[serde(skip)]
    Resize(u16, u16),
    /// Clear the terminal user interface screen.
    #[serde(skip)]
    ClearScreen,
    // results actions
    /// Add entry under cursor to the list of selected entries and move the cursor down.
    ToggleSelectionDown,
    /// Add entry under cursor to the list of selected entries and move the cursor up.
    ToggleSelectionUp,
    /// Confirm current selection (multi select or entry under cursor).
    ConfirmSelection,
    /// Select the entry currently under the cursor and exit the application.
    SelectAndExit,
    /// Select the next entry in the currently focused list.
    SelectNextEntry,
    /// Select the previous entry in the currently focused list.
    SelectPrevEntry,
    /// Select the next page of entries in the currently focused list.
    SelectNextPage,
    /// Select the previous page of entries in the currently focused list.
    SelectPrevPage,
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
    #[serde(skip)]
    OpenEntry,
    // application actions
    /// Tick the application state.
    #[serde(skip)]
    Tick,
    /// Suspend the application.
    #[serde(skip)]
    Suspend,
    /// Resume the application.
    #[serde(skip)]
    Resume,
    /// Quit the application.
    Quit,
    /// Toggle a UI feature.
    ToggleRemoteControl,
    ToggleHelp,
    ToggleStatusBar,
    TogglePreview,
    /// Signal an error with the given message.
    #[serde(skip)]
    Error(String),
    /// No operation.
    #[serde(skip)]
    NoOp,
    // Channel actions
    /// FIXME: clean this up
    ToggleSendToChannel,
    /// Toggle between different source commands.
    CycleSources,
    /// Reload the current source command.
    ReloadSource,
    /// Switch to the specified channel directly via shortcut.
    #[serde(skip)]
    SwitchToChannel(String),
    /// Timer action for watch mode to trigger periodic reloads.
    #[serde(skip)]
    WatchTimer,
}
