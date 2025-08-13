use crate::{event::Key, screen::constants::ACTION_PREFIX};
use serde::{Deserialize, Serialize};
use serde_with::{OneOrMany, serde_as};
use std::fmt::Display;

/// The different actions that can be performed by the application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    // input actions
    /// Add a character to the input buffer.
    #[serde(skip)]
    AddInputChar(char),
    /// Delete the character before the cursor from the input buffer.
    DeletePrevChar,
    /// Delete the previous word from the input buffer.
    DeletePrevWord,
    /// Delete the character after the cursor from the input buffer.
    DeleteNextChar,
    /// Delete the current line from the input buffer.
    DeleteLine,
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
    /// Confirm selection using one of the `expect` keys.
    Expect(Key),
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
    /// Switch between the portrait and landscape modes.
    #[serde(rename = "toggle_layout")]
    ToggleOrientation,
    /// Signal an error with the given message.
    #[serde(skip)]
    Error(String),
    /// No operation.
    #[serde(skip)]
    NoOp,
    // Channel actions
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
    /// Execute an external action
    ExternalAction(String),
}

/// Container for one or more actions that can be executed together.
///
/// This enum enables binding single keys to multiple actions, allowing for
/// complex behaviors triggered by a single key press. It supports both
/// single action bindings (for backward compatibility) and multiple action
/// sequences.
///
/// # Variants
///
/// - `Single(Action)` - A single action binding
/// - `Multiple(Vec<Action>)` - Multiple actions executed in sequence
///
/// # Configuration Examples
///
/// ```toml
/// # Single action (traditional)
/// esc = "quit"
///
/// # Multiple actions (new feature)
/// "ctrl-r" = ["reload_source", "copy_entry_to_clipboard"]
/// ```
///
/// # Usage
///
/// ```rust
/// use television::action::{Action, Actions};
///
/// // Single action
/// let single = Actions::single(Action::Quit);
/// assert_eq!(single.as_slice(), &[Action::Quit]);
///
/// // Multiple actions
/// let multiple = Actions::multiple(vec![Action::ReloadSource, Action::Quit]);
/// assert_eq!(multiple.as_slice(), &[Action::ReloadSource, Action::Quit]);
///
/// // Convert to vector for execution
/// let actions_vec = multiple.into_vec();
/// assert_eq!(actions_vec, vec![Action::ReloadSource, Action::Quit]);
/// ```
#[serde_as]
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord,
)]
#[serde(transparent)]
pub struct Actions {
    #[serde_as(as = "OneOrMany<_>")]
    inner: Vec<Action>,
}

impl Actions {
    /// Creates a new `Actions` from a single action.
    pub fn single(action: Action) -> Self {
        Self {
            inner: vec![action],
        }
    }

    /// Creates a new `Actions` from multiple actions.
    pub fn multiple(actions: Vec<Action>) -> Self {
        Self { inner: actions }
    }

    /// Converts the `Actions` into a `Vec<Action>` for execution.
    ///
    /// This method consumes the `Actions` and returns a vector containing
    /// all actions to be executed.
    ///
    /// # Returns
    ///
    /// A `Vec<Action>` containing all actions to execute.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use television::action::{Action, Actions};
    ///
    /// let single = Actions::single(Action::Quit);
    /// assert_eq!(single.into_vec(), vec![Action::Quit]);
    ///
    /// let multiple = Actions::multiple(vec![Action::ReloadSource, Action::Quit]);
    /// assert_eq!(multiple.into_vec(), vec![Action::ReloadSource, Action::Quit]);
    /// ```
    pub fn into_vec(self) -> Vec<Action> {
        self.inner
    }

    /// Returns a slice view of the actions without consuming the `Actions`.
    ///
    /// This method provides efficient access to the contained actions as a slice,
    /// useful for iteration and inspection without taking ownership.
    ///
    /// # Returns
    ///
    /// A `&[Action]` slice containing all actions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use television::action::{Action, Actions};
    ///
    /// let single = Actions::single(Action::Quit);
    /// assert_eq!(single.as_slice(), &[Action::Quit]);
    ///
    /// let multiple = Actions::multiple(vec![Action::ReloadSource, Action::Quit]);
    /// assert_eq!(multiple.as_slice(), &[Action::ReloadSource, Action::Quit]);
    /// ```
    pub fn as_slice(&self) -> &[Action] {
        &self.inner
    }

    /// Returns `true` if this contains only a single action.
    pub fn is_single(&self) -> bool {
        self.inner.len() == 1
    }

    /// Returns `true` if this contains multiple actions.
    pub fn is_multiple(&self) -> bool {
        self.inner.len() > 1
    }

    /// Gets the first action, if any.
    pub fn first(&self) -> Option<&Action> {
        self.inner.first()
    }
}

impl From<Action> for Actions {
    /// Converts a single `Action` into `Actions`.
    ///
    /// This conversion allows seamless use of single actions where
    /// `Actions` is expected, maintaining backward compatibility.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use television::action::{Action, Actions};
    ///
    /// let actions: Actions = Action::Quit.into();
    /// assert_eq!(actions, Actions::single(Action::Quit));
    /// ```
    fn from(action: Action) -> Self {
        Self::single(action)
    }
}

impl From<Vec<Action>> for Actions {
    /// Converts a `Vec<Action>` into `Actions`.
    ///
    /// # Arguments
    ///
    /// * `actions` - Vector of actions to convert
    ///
    /// # Examples
    ///
    /// ```rust
    /// use television::action::{Action, Actions};
    ///
    /// let single_vec = vec![Action::Quit];
    /// let actions: Actions = single_vec.into();
    /// assert_eq!(actions, Action::Quit.into());
    ///
    /// let multi_vec = vec![Action::ReloadSource, Action::Quit];
    /// let actions: Actions = multi_vec.into();
    /// assert_eq!(actions, vec![Action::ReloadSource, Action::Quit].into());
    /// ```
    fn from(actions: Vec<Action>) -> Self {
        Self::multiple(actions)
    }
}

impl Display for Action {
    /// Formats the action as its string representation for configuration files.
    ///
    /// This implementation provides the `snake_case` string representation of each
    /// action as used in TOML configuration files. The output matches the
    /// `#[serde(rename_all = "snake_case")]` serialization format.
    ///
    /// # Returns
    ///
    /// The `snake_case` string representation of the action.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use television::action::Action;
    ///
    /// assert_eq!(Action::Quit.to_string(), "quit");
    /// assert_eq!(Action::SelectNextEntry.to_string(), "select_next_entry");
    /// assert_eq!(Action::TogglePreview.to_string(), "toggle_preview");
    /// ```
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
            Action::Expect(_) => write!(f, "expect"),
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
            Action::ToggleOrientation => write!(f, "toggle_layout"),
            Action::Error(_) => write!(f, "error"),
            Action::NoOp => write!(f, "no_op"),
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
            Action::ExternalAction(name) => write!(f, "{}", name),
        }
    }
}

// FIXME: we shouldn't need to rely on hardcoding the action names here.
impl<'de> serde::Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let action = match s.as_str() {
            "add_input_char" => Action::AddInputChar(' '),
            "delete_prev_char" => Action::DeletePrevChar,
            "delete_prev_word" => Action::DeletePrevWord,
            "delete_next_char" => Action::DeleteNextChar,
            "delete_line" => Action::DeleteLine,
            "go_to_prev_char" => Action::GoToPrevChar,
            "go_to_next_char" => Action::GoToNextChar,
            "go_to_input_start" => Action::GoToInputStart,
            "go_to_input_end" => Action::GoToInputEnd,
            "render" => Action::Render,
            "resize" => Action::Resize(0, 0),
            "clear_screen" => Action::ClearScreen,
            "toggle_selection_down" => Action::ToggleSelectionDown,
            "toggle_selection_up" => Action::ToggleSelectionUp,
            "confirm_selection" => Action::ConfirmSelection,
            "select_and_exit" => Action::SelectAndExit,
            "expect" => Action::Expect(Key::Char(' ')),
            "select_next_entry" => Action::SelectNextEntry,
            "select_prev_entry" => Action::SelectPrevEntry,
            "select_next_page" => Action::SelectNextPage,
            "select_prev_page" => Action::SelectPrevPage,
            "copy_entry_to_clipboard" => Action::CopyEntryToClipboard,
            "scroll_preview_up" => Action::ScrollPreviewUp,
            "scroll_preview_down" => Action::ScrollPreviewDown,
            "scroll_preview_half_page_up" => Action::ScrollPreviewHalfPageUp,
            "scroll_preview_half_page_down" => {
                Action::ScrollPreviewHalfPageDown
            }
            "open_entry" => Action::OpenEntry,
            "tick" => Action::Tick,
            "suspend" => Action::Suspend,
            "resume" => Action::Resume,
            "quit" => Action::Quit,
            "toggle_remote_control" => Action::ToggleRemoteControl,
            "toggle_help" => Action::ToggleHelp,
            "toggle_status_bar" => Action::ToggleStatusBar,
            "toggle_preview" => Action::TogglePreview,
            "toggle_layout" => Action::ToggleOrientation,
            "error" => Action::Error(String::new()),
            "no_op" => Action::NoOp,
            "cycle_sources" => Action::CycleSources,
            "reload_source" => Action::ReloadSource,
            "switch_to_channel" => Action::SwitchToChannel(String::new()),
            "watch_timer" => Action::WatchTimer,
            "select_prev_history" => Action::SelectPrevHistory,
            "select_next_history" => Action::SelectNextHistory,
            s if s.starts_with(ACTION_PREFIX) => {
                let action_name = s.trim_start_matches(ACTION_PREFIX);
                Action::ExternalAction(action_name.to_string())
            }
            _ => {
                return Err(serde::de::Error::unknown_variant(
                    &s,
                    &[
                        "add_input_char",
                        "delete_prev_char",
                        "delete_prev_word",
                        "delete_next_char",
                        "delete_line",
                        "go_to_prev_char",
                        "go_to_next_char",
                        "go_to_input_start",
                        "go_to_input_end",
                        "render",
                        "resize",
                        "clear_screen",
                        "toggle_selection_down",
                        "toggle_selection_up",
                        "confirm_selection",
                        "select_and_exit",
                        "expect",
                        "select_next_entry",
                        "select_prev_entry",
                        "select_next_page",
                        "select_prev_page",
                        "copy_entry_to_clipboard",
                        "scroll_preview_up",
                        "scroll_preview_down",
                        "scroll_preview_half_page_up",
                        "scroll_preview_half_page_down",
                        "open_entry",
                        "tick",
                        "suspend",
                        "resume",
                        "quit",
                        "toggle_remote_control",
                        "toggle_help",
                        "toggle_status_bar",
                        "toggle_preview",
                        "toggle_layout",
                        "error",
                        "no_op",
                        "cycle_sources",
                        "reload_source",
                        "switch_to_channel",
                        "watch_timer",
                        "select_prev_history",
                        "select_next_history",
                        "actions:*",
                    ],
                ));
            }
        };

        Ok(action)
    }
}

impl Action {
    /// Returns a user-friendly description of the action for help panels and UI display.
    ///
    /// This method provides human-readable descriptions of actions that are suitable
    /// for display in help panels, tooltips, and other user interfaces. Unlike the
    /// `Display` implementation which returns `snake_case` configuration names, this
    /// method returns descriptive text.
    ///
    /// # Returns
    ///
    /// A static string slice containing the user-friendly description.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use television::action::Action;
    ///
    /// assert_eq!(Action::Quit.description(), "Quit");
    /// assert_eq!(Action::SelectNextEntry.description(), "Navigate down");
    /// assert_eq!(Action::TogglePreview.description(), "Toggle preview");
    /// ```
    pub fn description(&self) -> &'static str {
        match self {
            // Input actions
            Action::AddInputChar(_) => "Add character",
            Action::DeletePrevChar => "Delete previous char",
            Action::DeletePrevWord => "Delete previous word",
            Action::DeleteNextChar => "Delete next char",
            Action::DeleteLine => "Delete line",
            Action::GoToPrevChar => "Move cursor left",
            Action::GoToNextChar => "Move cursor right",
            Action::GoToInputStart => "Move to start",
            Action::GoToInputEnd => "Move to end",

            // Rendering actions (typically not shown in help)
            Action::Render => "Render",
            Action::Resize(_, _) => "Resize",
            Action::ClearScreen => "Clear screen",

            // Selection actions
            Action::ToggleSelectionDown => "Toggle selection down",
            Action::ToggleSelectionUp => "Toggle selection up",
            Action::ConfirmSelection => "Select entry",
            Action::SelectAndExit => "Select and exit",
            Action::Expect(_) => "Expect key",

            // Navigation actions
            Action::SelectNextEntry => "Navigate down",
            Action::SelectPrevEntry => "Navigate up",
            Action::SelectNextPage => "Page down",
            Action::SelectPrevPage => "Page up",
            Action::CopyEntryToClipboard => "Copy to clipboard",

            // Preview actions
            Action::ScrollPreviewUp => "Preview scroll up",
            Action::ScrollPreviewDown => "Preview scroll down",
            Action::ScrollPreviewHalfPageUp => "Preview scroll half page up",
            Action::ScrollPreviewHalfPageDown => {
                "Preview scroll half page down"
            }
            Action::OpenEntry => "Open entry",

            // Application actions
            Action::Tick => "Tick",
            Action::Suspend => "Suspend",
            Action::Resume => "Resume",
            Action::Quit => "Quit",

            // Toggle actions
            Action::ToggleRemoteControl => "Toggle remote control",
            Action::ToggleHelp => "Toggle help",
            Action::ToggleStatusBar => "Toggle status bar",
            Action::TogglePreview => "Toggle preview",
            Action::ToggleOrientation => "Toggle layout",

            // Error and no-op
            Action::Error(_) => "Error",
            Action::NoOp => "No operation",

            // Channel actions
            Action::CycleSources => "Cycle sources",
            Action::ReloadSource => "Reload source",
            Action::SwitchToChannel(_) => "Switch to channel",
            Action::WatchTimer => "Watch timer",

            // History actions
            Action::SelectPrevHistory => "Previous history",
            Action::SelectNextHistory => "Next history",

            // Mouse actions
            Action::SelectEntryAtPosition(_, _) => "Select at position",
            Action::MouseClickAt(_, _) => "Mouse click",

            // External actions
            Action::ExternalAction(_) => "External action",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actions_single() {
        let single_action = Actions::single(Action::Quit);
        assert_eq!(single_action.into_vec(), vec![Action::Quit]);

        let single_from_action = Actions::from(Action::SelectNextEntry);
        assert_eq!(
            single_from_action,
            Actions::single(Action::SelectNextEntry)
        );
        assert_eq!(single_from_action.as_slice(), &[Action::SelectNextEntry]);
    }

    #[test]
    fn test_actions_multiple() {
        let actions_vec = vec![Action::CopyEntryToClipboard, Action::Quit];
        let multiple_actions: Actions = Actions::multiple(actions_vec.clone());
        assert_eq!(multiple_actions.into_vec(), actions_vec);

        let multiple_from_vec = Actions::from(actions_vec.clone());
        assert_eq!(multiple_from_vec, Actions::multiple(actions_vec.clone()));
        assert_eq!(multiple_from_vec.as_slice(), actions_vec.as_slice());
    }

    #[test]
    fn test_actions_as_slice() {
        let single: Actions = Actions::single(Action::DeleteLine);
        assert_eq!(single.as_slice(), &[Action::DeleteLine]);

        let multiple: Actions = Actions::multiple(vec![
            Action::ScrollPreviewUp,
            Action::ScrollPreviewDown,
        ]);
        assert_eq!(
            multiple.as_slice(),
            &[Action::ScrollPreviewUp, Action::ScrollPreviewDown]
        );
    }

    #[test]
    fn test_actions_into_vec() {
        let single: Actions = Actions::single(Action::ConfirmSelection);
        assert_eq!(single.into_vec(), vec![Action::ConfirmSelection]);

        let multiple: Actions = Actions::multiple(vec![
            Action::ToggleHelp,
            Action::ToggleStatusBar,
        ]);
        assert_eq!(
            multiple.into_vec(),
            vec![Action::ToggleHelp, Action::ToggleStatusBar]
        );
    }

    #[test]
    fn test_actions_hash_and_eq() {
        use std::collections::HashMap;

        let actions1: Actions = Actions::single(Action::Quit);
        let actions2: Actions = Actions::single(Action::Quit);
        let actions3: Actions =
            Actions::multiple(vec![Action::Quit, Action::ClearScreen]);
        let actions4: Actions =
            Actions::multiple(vec![Action::Quit, Action::ClearScreen]);

        assert_eq!(actions1, actions2);
        assert_eq!(actions3, actions4);
        assert_ne!(actions1, actions3);

        // Test that they can be used as HashMap keys
        let mut map = HashMap::new();
        map.insert(actions1.clone(), "single");
        map.insert(actions3.clone(), "multiple");

        assert_eq!(map.get(&actions2), Some(&"single"));
        assert_eq!(map.get(&actions4), Some(&"multiple"));
    }

    #[test]
    fn test_action_description() {
        // Test that description() returns user-friendly text
        assert_eq!(Action::Quit.description(), "Quit");
        assert_eq!(Action::SelectNextEntry.description(), "Navigate down");
        assert_eq!(Action::SelectPrevEntry.description(), "Navigate up");
        assert_eq!(Action::TogglePreview.description(), "Toggle preview");
        assert_eq!(Action::ToggleHelp.description(), "Toggle help");
        assert_eq!(Action::ConfirmSelection.description(), "Select entry");
        assert_eq!(
            Action::CopyEntryToClipboard.description(),
            "Copy to clipboard"
        );

        // Test that description() differs from Display (snake_case)
        assert_ne!(
            Action::SelectNextEntry.description(),
            Action::SelectNextEntry.to_string()
        );
        assert_eq!(Action::SelectNextEntry.to_string(), "select_next_entry");
        assert_eq!(Action::SelectNextEntry.description(), "Navigate down");
    }
}
