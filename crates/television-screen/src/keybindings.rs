use std::{collections::HashMap, fmt::Display};

use crate::{
    colors::ACTION_COLOR,
    mode::{Mode, CHANNEL_COLOR, REMOTE_CONTROL_COLOR, SEND_TO_CHANNEL_COLOR},
};
use ratatui::{
    layout::Constraint,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Cell, Row, Table},
};

#[derive(Debug, Clone)]
pub struct DisplayableKeybindings {
    bindings: HashMap<DisplayableAction, Vec<String>>,
}

impl DisplayableKeybindings {
    pub fn new(bindings: HashMap<DisplayableAction, Vec<String>>) -> Self {
        Self { bindings }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DisplayableAction {
    ResultsNavigation,
    PreviewNavigation,
    SelectEntry,
    CopyEntryToClipboard,
    SendToChannel,
    ToggleRemoteControl,
    Cancel,
    Quit,
}

impl Display for DisplayableAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let action = match self {
            DisplayableAction::ResultsNavigation => "Results navigation",
            DisplayableAction::PreviewNavigation => "Preview navigation",
            DisplayableAction::SelectEntry => "Select entry",
            DisplayableAction::CopyEntryToClipboard => {
                "Copy entry to clipboard"
            }
            DisplayableAction::SendToChannel => "Send to channel",
            DisplayableAction::ToggleRemoteControl => "Toggle Remote control",
            DisplayableAction::Cancel => "Cancel",
            DisplayableAction::Quit => "Quit",
        };
        write!(f, "{}", action)
    }
}

pub fn build_keybindings_table(
    keybindings: &HashMap<Mode, DisplayableKeybindings>,
    mode: Mode,
) -> Table<'_> {
    match mode {
        Mode::Channel => {
            build_keybindings_table_for_channel(&keybindings[&mode])
        }
        Mode::RemoteControl => {
            build_keybindings_table_for_channel_selection(&keybindings[&mode])
        }
        Mode::SendToChannel => {
            build_keybindings_table_for_channel_transitions(
                &keybindings[&mode],
            )
        }
    }
}

fn build_keybindings_table_for_channel(
    keybindings: &DisplayableKeybindings,
) -> Table<'_> {
    // Results navigation
    let results_navigation_keys = keybindings
        .bindings
        .get(&DisplayableAction::ResultsNavigation)
        .unwrap();
    let results_row = Row::new(build_cells_for_group(
        "Results navigation",
        results_navigation_keys,
        CHANNEL_COLOR,
    ));

    // Preview navigation
    let preview_navigation_keys = keybindings
        .bindings
        .get(&DisplayableAction::PreviewNavigation)
        .unwrap();
    let preview_row = Row::new(build_cells_for_group(
        "Preview navigation",
        preview_navigation_keys,
        CHANNEL_COLOR,
    ));

    // Select entry
    let select_entry_keys = keybindings
        .bindings
        .get(&DisplayableAction::SelectEntry)
        .unwrap();
    let select_entry_row = Row::new(build_cells_for_group(
        "Select entry",
        select_entry_keys,
        CHANNEL_COLOR,
    ));

    // Copy entry to clipboard
    let copy_entry_keys = keybindings
        .bindings
        .get(&DisplayableAction::CopyEntryToClipboard)
        .unwrap();
    let copy_entry_row = Row::new(build_cells_for_group(
        "Copy entry to clipboard",
        copy_entry_keys,
        CHANNEL_COLOR,
    ));

    // Send to channel
    let send_to_channel_keys = keybindings
        .bindings
        .get(&DisplayableAction::SendToChannel)
        .unwrap();
    let send_to_channel_row = Row::new(build_cells_for_group(
        "Send results to",
        send_to_channel_keys,
        CHANNEL_COLOR,
    ));

    // Switch channels
    let switch_channels_keys = keybindings
        .bindings
        .get(&DisplayableAction::ToggleRemoteControl)
        .unwrap();
    let switch_channels_row = Row::new(build_cells_for_group(
        "Toggle Remote control",
        switch_channels_keys,
        CHANNEL_COLOR,
    ));

    // MISC line (quit, help, etc.)
    // Quit ⏼
    let quit_keys =
        keybindings.bindings.get(&DisplayableAction::Quit).unwrap();
    let quit_row =
        Row::new(build_cells_for_group("Quit", quit_keys, CHANNEL_COLOR));

    let widths = vec![Constraint::Fill(1), Constraint::Fill(2)];

    Table::new(
        vec![
            results_row,
            preview_row,
            select_entry_row,
            copy_entry_row,
            send_to_channel_row,
            switch_channels_row,
            quit_row,
        ],
        widths,
    )
}

fn build_keybindings_table_for_channel_selection(
    keybindings: &DisplayableKeybindings,
) -> Table<'_> {
    // Results navigation
    let navigation_keys = keybindings
        .bindings
        .get(&DisplayableAction::ResultsNavigation)
        .unwrap();
    let results_row = Row::new(build_cells_for_group(
        "Browse channels",
        navigation_keys,
        REMOTE_CONTROL_COLOR,
    ));

    // Select entry
    let select_entry_keys = keybindings
        .bindings
        .get(&DisplayableAction::SelectEntry)
        .unwrap();
    let select_entry_row = Row::new(build_cells_for_group(
        "Select channel",
        select_entry_keys,
        REMOTE_CONTROL_COLOR,
    ));

    // Remote control
    let switch_channels_keys = keybindings
        .bindings
        .get(&DisplayableAction::ToggleRemoteControl)
        .unwrap();
    let switch_channels_row = Row::new(build_cells_for_group(
        "Toggle Remote control",
        switch_channels_keys,
        REMOTE_CONTROL_COLOR,
    ));

    Table::new(
        vec![results_row, select_entry_row, switch_channels_row],
        vec![Constraint::Fill(1), Constraint::Fill(2)],
    )
}

fn build_keybindings_table_for_channel_transitions(
    keybindings: &DisplayableKeybindings,
) -> Table<'_> {
    // Results navigation
    let results_navigation_keys = keybindings
        .bindings
        .get(&DisplayableAction::ResultsNavigation)
        .unwrap();
    let results_row = Row::new(build_cells_for_group(
        "Browse channels",
        results_navigation_keys,
        SEND_TO_CHANNEL_COLOR,
    ));

    // Select entry
    let select_entry_keys = keybindings
        .bindings
        .get(&DisplayableAction::SelectEntry)
        .unwrap();
    let select_entry_row = Row::new(build_cells_for_group(
        "Send to channel",
        select_entry_keys,
        SEND_TO_CHANNEL_COLOR,
    ));

    // Cancel
    let cancel_keys = keybindings
        .bindings
        .get(&DisplayableAction::Cancel)
        .unwrap();
    let cancel_row = Row::new(build_cells_for_group(
        "Cancel",
        cancel_keys,
        SEND_TO_CHANNEL_COLOR,
    ));

    Table::new(
        vec![results_row, select_entry_row, cancel_row],
        vec![Constraint::Fill(1), Constraint::Fill(2)],
    )
}

fn build_cells_for_group<'a>(
    group_name: &str,
    keys: &'a [String],
    key_color: Color,
) -> Vec<Cell<'a>> {
    // group name
    let mut cells = vec![Cell::from(Span::styled(
        group_name.to_owned() + ": ",
        Style::default().fg(ACTION_COLOR),
    ))];

    let spans = keys.iter().skip(1).fold(
        vec![Span::styled(
            keys[0].clone(),
            Style::default().fg(key_color),
        )],
        |mut acc, key| {
            acc.push(Span::raw(" / "));
            acc.push(Span::styled(
                key.to_owned(),
                Style::default().fg(key_color),
            ));
            acc
        },
    );

    cells.push(Cell::from(Line::from(spans)));

    cells
}
