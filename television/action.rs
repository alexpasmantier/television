use serde::{Deserialize, Serialize};

/// The different actions that can be performed by the application.
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord,
)]
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
    #[serde(alias = "go_to_input_start")]
    GoToInputStart,
    /// Move the cursor to the end of the input buffer.
    #[serde(alias = "go_to_input_end")]
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
    #[serde(alias = "toggle_selection_down")]
    ToggleSelectionDown,
    /// Add entry under cursor to the list of selected entries and move the cursor up.
    #[serde(alias = "toggle_selection_up")]
    ToggleSelectionUp,
    /// Confirm current selection (multi select or entry under cursor).
    #[serde(alias = "select_entry")]
    #[serde(alias = "confirm_selection")]
    ConfirmSelection,
    /// Select the entry currently under the cursor and exit the application.
    #[serde(alias = "select_and_exit")]
    SelectAndExit,
    /// Select the next entry in the currently focused list.
    #[serde(alias = "select_next_entry")]
    SelectNextEntry,
    /// Select the previous entry in the currently focused list.
    #[serde(alias = "select_prev_entry")]
    SelectPrevEntry,
    /// Select the next page of entries in the currently focused list.
    #[serde(alias = "select_next_page")]
    SelectNextPage,
    /// Select the previous page of entries in the currently focused list.
    #[serde(alias = "select_prev_page")]
    SelectPrevPage,
    /// Copy the currently selected entry to the clipboard.
    #[serde(alias = "copy_entry_to_clipboard")]
    CopyEntryToClipboard,
    // preview actions
    /// Scroll the preview up by one line.
    #[serde(alias = "scroll_preview_up")]
    ScrollPreviewUp,
    /// Scroll the preview down by one line.
    #[serde(alias = "scroll_preview_down")]
    ScrollPreviewDown,
    /// Scroll the preview up by half a page.
    #[serde(alias = "scroll_preview_half_page_up")]
    ScrollPreviewHalfPageUp,
    /// Scroll the preview down by half a page.
    #[serde(alias = "scroll_preview_half_page_down")]
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
    #[serde(alias = "quit")]
    Quit,
    /// Toggle the help bar.
    #[serde(alias = "toggle_help")]
    ToggleHelp,
    /// Toggle the preview panel.
    #[serde(alias = "toggle_preview")]
    TogglePreview,
    /// Signal an error with the given message.
    #[serde(skip)]
    Error(String),
    /// No operation.
    #[serde(skip)]
    NoOp,
    // channel actions
    /// Toggle the remote control channel.
    #[serde(alias = "toggle_remote_control")]
    ToggleRemoteControl,
    /// Toggle the remote control in `send to channel` mode.
    #[serde(alias = "toggle_send_to_channel")]
    ToggleSendToChannel,
    /// Toggle between different source commands.
    #[serde(alias = "cycle_sources")]
    CycleSources,
    /// Reload the current source command.
    #[serde(alias = "reload_source")]
    ReloadSource,
    /// Transition to the next channel(s) defined in the current channel prototype.
    #[serde(alias = "transition")]
    Transition,
    /// Return to the previous channel after a transition.
    #[serde(alias = "transition_back")]
    TransitionBack,
}
