use serde::{Deserialize, Serialize};
use strum::Display;

use crate::app::Mode;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Command {
    Quit,
    NextTab,
    PreviousTab,
    ClosePopup,
    SwitchMode(Mode),
    SwitchToLastMode,
    IncrementPage,
    DecrementPage,
    NextSummaryMode,
    PreviousSummaryMode,
    ToggleSortBy { reload: bool, forward: bool },
    ScrollBottom,
    ScrollTop,
    ScrollDown,
    ScrollUp,
    ScrollCrateInfoDown,
    ScrollCrateInfoUp,
    ScrollSearchResultsDown,
    ScrollSearchResultsUp,
    SubmitSearch,
    ReloadData,
    ToggleShowCrateInfo,
    CopyCargoAddCommandToClipboard,
    OpenDocsUrlInBrowser,
    OpenCratesIOUrlInBrowser,
}

pub const HELP_COMMANDS: &[Command] = &[Command::SwitchToLastMode];
pub const PICKER_COMMANDS: &[Command] = &[
    Command::SwitchMode(Mode::Help),
    Command::SwitchMode(Mode::Summary),
    Command::SwitchMode(Mode::Search),
    Command::SwitchMode(Mode::Filter),
    Command::ScrollUp,
    Command::ScrollDown,
    Command::ScrollCrateInfoUp,
    Command::ScrollCrateInfoDown,
    Command::ToggleSortBy {
        reload: true,
        forward: true,
    },
    Command::ToggleSortBy {
        reload: true,
        forward: false,
    },
    Command::ToggleSortBy {
        reload: false,
        forward: true,
    },
    Command::ToggleSortBy {
        reload: false,
        forward: false,
    },
    Command::IncrementPage,
    Command::DecrementPage,
    Command::ReloadData,
    Command::ToggleShowCrateInfo,
    Command::OpenDocsUrlInBrowser,
    Command::OpenCratesIOUrlInBrowser,
    Command::CopyCargoAddCommandToClipboard,
];
pub const SUMMARY_COMMANDS: &[Command] = &[
    Command::Quit,
    Command::ScrollDown,
    Command::ScrollUp,
    Command::PreviousSummaryMode,
    Command::NextSummaryMode,
    Command::SwitchMode(Mode::Help),
    Command::SwitchMode(Mode::Search),
    Command::SwitchMode(Mode::Filter),
];
pub const SEARCH_COMMANDS: &[Command] = &[
    Command::SwitchMode(Mode::PickerHideCrateInfo),
    Command::SubmitSearch,
    Command::ToggleSortBy {
        reload: false,
        forward: true,
    },
    Command::ToggleSortBy {
        reload: false,
        forward: false,
    },
    Command::ToggleSortBy {
        reload: true,
        forward: true,
    },
    Command::ToggleSortBy {
        reload: true,
        forward: false,
    },
    Command::ScrollSearchResultsUp,
    Command::ScrollSearchResultsDown,
    Command::SwitchMode(Mode::PickerHideCrateInfo),
    Command::ScrollSearchResultsUp,
    Command::ScrollSearchResultsDown,
];
pub const ALL_COMMANDS: &[(Mode, &[Command])] = &[
    (Mode::Help, HELP_COMMANDS),
    (Mode::PickerHideCrateInfo, PICKER_COMMANDS),
    (Mode::Summary, SUMMARY_COMMANDS),
    (Mode::Search, SEARCH_COMMANDS),
];
