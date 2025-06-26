use crate::{
    action::Action,
    config::{Binding, KeyBindings},
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

/// Unified key extraction function that works for both systems
pub fn extract_keys_from_binding(binding: &Binding) -> Vec<String> {
    match binding {
        Binding::SingleKey(key) => {
            vec![key.to_string()]
        }
        Binding::MultipleKeys(keys) => {
            keys.iter().map(ToString::to_string).collect()
        }
    }
}

/// Extract keys for multiple actions and return them as a flat vector
pub fn extract_keys_for_actions(
    keybindings: &KeyBindings,
    actions: &[Action],
) -> Vec<String> {
    actions
        .iter()
        .filter_map(|action| keybindings.get(action))
        .flat_map(extract_keys_from_binding)
        .collect()
}
