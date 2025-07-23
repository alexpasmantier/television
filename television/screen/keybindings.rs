use crate::{
    action::{Action, Actions},
    config::KeyBindings,
    television::Mode,
};
use std::fmt::Display;

/// Centralized action descriptions to avoid duplication between keybinding panel and help bar
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ActionCategory {
    // Global actions
    Quit,
    ToggleFeature,

    // Navigation actions (common to both modes)
    ResultsNavigation,
    PreviewNavigation,

    // Selection actions
    SelectEntry,
    ToggleSelection,

    // Channel-specific actions
    CopyEntryToClipboard,
    ToggleRemoteControl,
    CycleSources,
    ReloadSource,
}

impl Display for ActionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let description = match self {
            ActionCategory::Quit => "Quit",
            ActionCategory::ToggleFeature => "Toggle features",
            ActionCategory::ResultsNavigation => "Results navigation",
            ActionCategory::PreviewNavigation => "Preview navigation",
            ActionCategory::SelectEntry => "Select entry",
            ActionCategory::ToggleSelection => "Toggle selection",
            ActionCategory::CopyEntryToClipboard => "Copy entry to clipboard",
            ActionCategory::ToggleRemoteControl => "Toggle Remote control",
            ActionCategory::CycleSources => "Cycle through sources",
            ActionCategory::ReloadSource => "Reload source",
        };
        write!(f, "{description}")
    }
}

/// Defines what actions belong to each category and their individual descriptions
pub struct ActionMapping {
    pub category: ActionCategory,
    pub actions: Vec<(Action, &'static str)>,
}

impl ActionMapping {
    /// Get all action mappings for global actions
    pub fn global_actions() -> Vec<ActionMapping> {
        vec![
            ActionMapping {
                category: ActionCategory::Quit,
                actions: vec![(Action::Quit, "Quit")],
            },
            ActionMapping {
                category: ActionCategory::ToggleFeature,
                actions: vec![
                    (Action::TogglePreview, "Toggle preview"),
                    (Action::ToggleHelp, "Toggle help"),
                    (Action::ToggleStatusBar, "Toggle status bar"),
                ],
            },
        ]
    }

    /// Get all action mappings for navigation actions (common to both modes)
    pub fn navigation_actions() -> Vec<ActionMapping> {
        vec![
            ActionMapping {
                category: ActionCategory::ResultsNavigation,
                actions: vec![
                    (Action::SelectPrevEntry, "Navigate up"),
                    (Action::SelectNextEntry, "Navigate down"),
                    (Action::SelectPrevPage, "Page up"),
                    (Action::SelectNextPage, "Page down"),
                ],
            },
            ActionMapping {
                category: ActionCategory::PreviewNavigation,
                actions: vec![
                    (Action::ScrollPreviewHalfPageUp, "Preview scroll up"),
                    (Action::ScrollPreviewHalfPageDown, "Preview scroll down"),
                ],
            },
        ]
    }

    /// Get mode-specific action mappings
    pub fn mode_specific_actions(mode: Mode) -> Vec<ActionMapping> {
        match mode {
            Mode::Channel => vec![
                ActionMapping {
                    category: ActionCategory::SelectEntry,
                    actions: vec![
                        (Action::ConfirmSelection, "Select entry"),
                        (Action::ToggleSelectionDown, "Toggle selection down"),
                        (Action::ToggleSelectionUp, "Toggle selection up"),
                    ],
                },
                ActionMapping {
                    category: ActionCategory::CopyEntryToClipboard,
                    actions: vec![(
                        Action::CopyEntryToClipboard,
                        "Copy to clipboard",
                    )],
                },
                ActionMapping {
                    category: ActionCategory::ToggleRemoteControl,
                    actions: vec![(
                        Action::ToggleRemoteControl,
                        "Remote Control",
                    )],
                },
                ActionMapping {
                    category: ActionCategory::CycleSources,
                    actions: vec![(Action::CycleSources, "Cycle sources")],
                },
                ActionMapping {
                    category: ActionCategory::ReloadSource,
                    actions: vec![(Action::ReloadSource, "Reload source")],
                },
            ],
            Mode::RemoteControl => vec![
                ActionMapping {
                    category: ActionCategory::SelectEntry,
                    actions: vec![(Action::ConfirmSelection, "Select entry")],
                },
                ActionMapping {
                    category: ActionCategory::ToggleRemoteControl,
                    actions: vec![(
                        Action::ToggleRemoteControl,
                        "Back to Channel",
                    )],
                },
            ],
        }
    }

    /// Get all actions for a specific category, flattened for help bar usage
    pub fn actions_for_category(&self) -> &[Action] {
        // This is a bit of a hack to return just the Action part of the tuples
        // We'll need to handle this differently in the help bar system
        &[]
    }
}

/// Extract keys for a single action from the new Key->Action keybindings format
pub fn find_keys_for_action(
    keybindings: &KeyBindings,
    target_action: &Actions,
) -> Vec<String> {
    keybindings
        .bindings
        .iter()
        .filter_map(|(key, action)| {
            if action == target_action {
                Some(key.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Extract keys for a single action (convenience function)
pub fn find_keys_for_single_action(
    keybindings: &KeyBindings,
    target_action: &Action,
) -> Vec<String> {
    keybindings
        .bindings
        .iter()
        .filter_map(|(key, actions)| {
            // Check if this actions contains the target action
            if actions.as_slice().contains(target_action) {
                Some(key.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Extract keys for multiple actions and return them as a flat vector
pub fn extract_keys_for_actions(
    keybindings: &KeyBindings,
    actions: &[Actions],
) -> Vec<String> {
    actions
        .iter()
        .flat_map(|action| find_keys_for_action(keybindings, action))
        .collect()
}

/// Remove all keybindings for a specific action from `KeyBindings`
pub fn remove_action_bindings(
    keybindings: &mut KeyBindings,
    target_action: &Actions,
) {
    keybindings
        .bindings
        .retain(|_, action| action != target_action);
}
