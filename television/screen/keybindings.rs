use rustc_hash::FxHashMap;
use std::fmt::Display;

use crate::action::Action;
use crate::television::Mode;
use crate::{config::KeyBindings, screen::colors::Colorscheme};
use ratatui::{
    layout::Constraint,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Cell, Row, Table},
};

impl KeyBindings {
    pub fn to_displayable(&self) -> FxHashMap<Mode, DisplayableKeybindings> {
        // channel mode keybindings
        let channel_bindings: FxHashMap<DisplayableAction, Vec<String>> =
            FxHashMap::from_iter(vec![
                (
                    DisplayableAction::ResultsNavigation,
                    serialized_keys_for_actions(
                        self,
                        &[
                            Action::SelectPrevEntry,
                            Action::SelectNextEntry,
                            Action::SelectPrevPage,
                            Action::SelectNextPage,
                        ],
                    ),
                ),
                (
                    DisplayableAction::PreviewNavigation,
                    serialized_keys_for_actions(
                        self,
                        &[
                            Action::ScrollPreviewHalfPageUp,
                            Action::ScrollPreviewHalfPageDown,
                        ],
                    ),
                ),
                (
                    DisplayableAction::SelectEntry,
                    serialized_keys_for_actions(
                        self,
                        &[
                            Action::ConfirmSelection,
                            Action::ToggleSelectionDown,
                            Action::ToggleSelectionUp,
                        ],
                    ),
                ),
                (
                    DisplayableAction::CopyEntryToClipboard,
                    serialized_keys_for_actions(
                        self,
                        &[Action::CopyEntryToClipboard],
                    ),
                ),
                (
                    DisplayableAction::ToggleRemoteControl,
                    serialized_keys_for_actions(
                        self,
                        &[Action::ToggleRemoteControl],
                    ),
                ),
                (
                    DisplayableAction::ToggleHelpBar,
                    serialized_keys_for_actions(self, &[Action::ToggleHelp]),
                ),
            ]);

        // remote control mode keybindings
        let remote_control_bindings: FxHashMap<
            DisplayableAction,
            Vec<String>,
        > = FxHashMap::from_iter(vec![
            (
                DisplayableAction::ResultsNavigation,
                serialized_keys_for_actions(
                    self,
                    &[Action::SelectPrevEntry, Action::SelectNextEntry],
                ),
            ),
            (
                DisplayableAction::SelectEntry,
                serialized_keys_for_actions(self, &[Action::ConfirmSelection]),
            ),
            (
                DisplayableAction::ToggleRemoteControl,
                serialized_keys_for_actions(
                    self,
                    &[Action::ToggleRemoteControl],
                ),
            ),
        ]);

        FxHashMap::from_iter(vec![
            (Mode::Channel, DisplayableKeybindings::new(channel_bindings)),
            (
                Mode::RemoteControl,
                DisplayableKeybindings::new(remote_control_bindings),
            ),
        ])
    }
}

fn serialized_keys_for_actions(
    keybindings: &KeyBindings,
    actions: &[Action],
) -> Vec<String> {
    actions
        .iter()
        .map(|a| keybindings.get(a).unwrap().clone().to_string())
        .collect()
}

#[derive(Debug, Clone)]
pub struct DisplayableKeybindings {
    bindings: FxHashMap<DisplayableAction, Vec<String>>,
}

impl DisplayableKeybindings {
    pub fn new(bindings: FxHashMap<DisplayableAction, Vec<String>>) -> Self {
        Self { bindings }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DisplayableAction {
    ResultsNavigation,
    PreviewNavigation,
    SelectEntry,
    CopyEntryToClipboard,
    ToggleRemoteControl,
    Cancel,
    Quit,
    ToggleHelpBar,
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
            DisplayableAction::ToggleRemoteControl => "Toggle Remote control",
            DisplayableAction::Cancel => "Cancel",
            DisplayableAction::Quit => "Quit",
            DisplayableAction::ToggleHelpBar => "Toggle help bar",
        };
        write!(f, "{action}")
    }
}

pub fn build_keybindings_table<'a>(
    keybindings: &'a FxHashMap<Mode, DisplayableKeybindings>,
    mode: Mode,
    colorscheme: &'a Colorscheme,
) -> Table<'a> {
    match mode {
        Mode::Channel => build_keybindings_table_for_channel(
            &keybindings[&mode],
            colorscheme,
        ),
        Mode::RemoteControl => build_keybindings_table_for_channel_selection(
            &keybindings[&mode],
            colorscheme,
        ),
    }
}

fn build_keybindings_table_for_channel<'a>(
    keybindings: &'a DisplayableKeybindings,
    colorscheme: &'a Colorscheme,
) -> Table<'a> {
    // Results navigation
    let results_navigation_keys = keybindings
        .bindings
        .get(&DisplayableAction::ResultsNavigation)
        .unwrap();
    let results_row = Row::new(build_cells_for_group(
        "Results navigation",
        results_navigation_keys,
        colorscheme.help.metadata_field_name_fg,
        colorscheme.mode.channel,
    ));

    // Preview navigation
    let preview_navigation_keys = keybindings
        .bindings
        .get(&DisplayableAction::PreviewNavigation)
        .unwrap();
    let preview_row = Row::new(build_cells_for_group(
        "Preview navigation",
        preview_navigation_keys,
        colorscheme.help.metadata_field_name_fg,
        colorscheme.mode.channel,
    ));

    // Select entry
    let select_entry_keys = keybindings
        .bindings
        .get(&DisplayableAction::SelectEntry)
        .unwrap();
    let select_entry_row = Row::new(build_cells_for_group(
        "Select entry",
        select_entry_keys,
        colorscheme.help.metadata_field_name_fg,
        colorscheme.mode.channel,
    ));

    // Copy entry to clipboard
    let copy_entry_keys = keybindings
        .bindings
        .get(&DisplayableAction::CopyEntryToClipboard)
        .unwrap();
    let copy_entry_row = Row::new(build_cells_for_group(
        "Copy entry to clipboard",
        copy_entry_keys,
        colorscheme.help.metadata_field_name_fg,
        colorscheme.mode.channel,
    ));

    // Switch channels
    let switch_channels_keys = keybindings
        .bindings
        .get(&DisplayableAction::ToggleRemoteControl)
        .unwrap();
    let switch_channels_row = Row::new(build_cells_for_group(
        "Toggle Remote control",
        switch_channels_keys,
        colorscheme.help.metadata_field_name_fg,
        colorscheme.mode.channel,
    ));

    let widths = vec![Constraint::Fill(1), Constraint::Fill(2)];

    Table::new(
        vec![
            results_row,
            preview_row,
            select_entry_row,
            copy_entry_row,
            switch_channels_row,
        ],
        widths,
    )
}

fn build_keybindings_table_for_channel_selection<'a>(
    keybindings: &'a DisplayableKeybindings,
    colorscheme: &'a Colorscheme,
) -> Table<'a> {
    // Results navigation
    let navigation_keys = keybindings
        .bindings
        .get(&DisplayableAction::ResultsNavigation)
        .unwrap();
    let results_row = Row::new(build_cells_for_group(
        "Browse channels",
        navigation_keys,
        colorscheme.help.metadata_field_name_fg,
        colorscheme.mode.remote_control,
    ));

    // Select entry
    let select_entry_keys = keybindings
        .bindings
        .get(&DisplayableAction::SelectEntry)
        .unwrap();
    let select_entry_row = Row::new(build_cells_for_group(
        "Select channel",
        select_entry_keys,
        colorscheme.help.metadata_field_name_fg,
        colorscheme.mode.remote_control,
    ));

    // Remote control
    let switch_channels_keys = keybindings
        .bindings
        .get(&DisplayableAction::ToggleRemoteControl)
        .unwrap();
    let switch_channels_row = Row::new(build_cells_for_group(
        "Toggle Remote control",
        switch_channels_keys,
        colorscheme.help.metadata_field_name_fg,
        colorscheme.mode.remote_control,
    ));

    Table::new(
        vec![results_row, select_entry_row, switch_channels_row],
        vec![Constraint::Fill(1), Constraint::Fill(2)],
    )
}

fn build_cells_for_group<'a>(
    group_name: &str,
    keys: &'a [String],
    key_color: Color,
    value_color: Color,
) -> Vec<Cell<'a>> {
    // group name
    let mut cells = vec![Cell::from(Span::styled(
        group_name.to_owned() + ": ",
        Style::default().fg(key_color),
    ))];

    let spans = keys.iter().skip(1).fold(
        vec![Span::styled(
            keys[0].clone(),
            Style::default().fg(value_color),
        )],
        |mut acc, key| {
            acc.push(Span::raw(" / "));
            acc.push(Span::styled(
                key.to_owned(),
                Style::default().fg(value_color),
            ));
            acc
        },
    );

    cells.push(Cell::from(Line::from(spans)));

    cells
}
