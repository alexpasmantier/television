use crate::event::Key;
use serde::Deserialize;
use serde_with::{OneOrMany, serde_as};

/// The different actions that can be performed by the application.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Hash, PartialOrd, Ord)]
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
    ToggleActionPicker,
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
    NoOp,
    // Channel actions
    /// Cycle between different source commands.
    CycleSources,
    /// Cycle between different preview commands.
    CyclePreviews,
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
    #[serde(untagged)]
    ExternalAction(String),
}

/// Prefix used to identify custom external actions defined by the user in a channel's prototype.
pub const CUSTOM_ACTION_PREFIX: &str = "actions:";

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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Hash, PartialOrd, Ord)]
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

    /// Gets the first action, if any.
    ///
    /// This is used by the help panel to display a representative action
    /// when multiple actions are bound to a single key.
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
    pub fn description(&self) -> &str {
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
            Action::ToggleActionPicker => "Toggle action picker",
            Action::ToggleHelp => "Toggle help",
            Action::ToggleStatusBar => "Toggle status bar",
            Action::TogglePreview => "Toggle preview",
            Action::ToggleOrientation => "Toggle layout",

            // Error and no-op
            Action::Error(_) => "Error",
            Action::NoOp => "No operation",

            // Channel actions
            Action::CycleSources => "Cycle sources",
            Action::CyclePreviews => "Cycle previews",
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
            Action::ExternalAction(a) => a,
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
}
