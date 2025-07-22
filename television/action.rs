use serde::{Deserialize, Serialize};
use std::fmt::Display;

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
    /// Navigate to the previous entry in the history.
    SelectPrevHistory,
    /// Navigate to the next entry in the history.
    SelectNextHistory,
    // Mouse and position-aware actions
    /// Select an entry at a specific position (e.g., from mouse click)
    #[serde(skip)]
    SelectEntryAtPosition(u16, u16),
    /// Handle mouse click event at specific coordinates
    #[serde(skip)]
    MouseClickAt(u16, u16),
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::AddInputChar(_) => write!(f, "add_input_char"),
            Action::DeletePrevChar => write!(f, "delete_prev_char"),
            Action::DeletePrevWord => write!(f, "delete_prev_word"),
            Action::DeleteNextChar => write!(f, "delete_next_char"),
            Action::DeleteLine => write!(f, "delete_line"),
            Action::GoToPrevChar => write!(f, "go_to_prev_char"),
            Action::GoToNextChar => write!(f, "go_to_next_char"),
            Action::GoToInputStart => write!(f, "go_to_input_start"),
            Action::GoToInputEnd => write!(f, "go_to_input_end"),
            Action::Render => write!(f, "render"),
            Action::Resize(_, _) => write!(f, "resize"),
            Action::ClearScreen => write!(f, "clear_screen"),
            Action::ToggleSelectionDown => write!(f, "toggle_selection_down"),
            Action::ToggleSelectionUp => write!(f, "toggle_selection_up"),
            Action::ConfirmSelection => write!(f, "confirm_selection"),
            Action::SelectAndExit => write!(f, "select_and_exit"),
            Action::SelectNextEntry => write!(f, "select_next_entry"),
            Action::SelectPrevEntry => write!(f, "select_prev_entry"),
            Action::SelectNextPage => write!(f, "select_next_page"),
            Action::SelectPrevPage => write!(f, "select_prev_page"),
            Action::CopyEntryToClipboard => {
                write!(f, "copy_entry_to_clipboard")
            }
            Action::ScrollPreviewUp => write!(f, "scroll_preview_up"),
            Action::ScrollPreviewDown => write!(f, "scroll_preview_down"),
            Action::ScrollPreviewHalfPageUp => {
                write!(f, "scroll_preview_half_page_up")
            }
            Action::ScrollPreviewHalfPageDown => {
                write!(f, "scroll_preview_half_page_down")
            }
            Action::OpenEntry => write!(f, "open_entry"),
            Action::Tick => write!(f, "tick"),
            Action::Suspend => write!(f, "suspend"),
            Action::Resume => write!(f, "resume"),
            Action::Quit => write!(f, "quit"),
            Action::ToggleRemoteControl => write!(f, "toggle_remote_control"),
            Action::ToggleHelp => write!(f, "toggle_help"),
            Action::ToggleStatusBar => write!(f, "toggle_status_bar"),
            Action::TogglePreview => write!(f, "toggle_preview"),
            Action::Error(_) => write!(f, "error"),
            Action::NoOp => write!(f, "no_op"),
            Action::ToggleSendToChannel => write!(f, "toggle_send_to_channel"),
            Action::CycleSources => write!(f, "cycle_sources"),
            Action::ReloadSource => write!(f, "reload_source"),
            Action::SwitchToChannel(_) => write!(f, "switch_to_channel"),
            Action::WatchTimer => write!(f, "watch_timer"),
            Action::SelectPrevHistory => write!(f, "select_prev_history"),
            Action::SelectNextHistory => write!(f, "select_next_history"),
            Action::SelectEntryAtPosition(_, _) => {
                write!(f, "select_entry_at_position")
            }
            Action::MouseClickAt(_, _) => write!(f, "mouse_click_at"),
        }
    }
}
