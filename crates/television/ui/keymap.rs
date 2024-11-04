use color_eyre::eyre::{OptionExt, Result};
use ratatui::{
    layout::Constraint,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Cell, Row, Table},
};
use std::collections::HashMap;

use crate::ui::mode::mode_color;
use crate::{
    action::Action,
    event::Key,
    television::{Mode, Television},
};

const ACTION_COLOR: Color = Color::DarkGray;

impl Television {
    pub fn build_keymap_table<'a>(&self) -> Result<Table<'a>> {
        match self.mode {
            Mode::Channel => self.build_keymap_table_for_channel(),
            Mode::RemoteControl => {
                self.build_keymap_table_for_channel_selection()
            }
            Mode::SendToChannel => {
                self.build_keymap_table_for_channel_transitions()
            }
        }
    }

    fn build_keymap_table_for_channel<'a>(&self) -> Result<Table<'a>> {
        let keymap = self.keymap_for_mode()?;
        let key_color = mode_color(self.mode);

        // Results navigation
        let prev = keys_for_action(keymap, &Action::SelectPrevEntry);
        let next = keys_for_action(keymap, &Action::SelectNextEntry);
        let results_row = Row::new(build_cells_for_key_groups(
            "Results navigation",
            vec![prev, next],
            key_color,
        ));

        // Preview navigation
        let up_keys =
            keys_for_action(keymap, &Action::ScrollPreviewHalfPageUp);
        let down_keys =
            keys_for_action(keymap, &Action::ScrollPreviewHalfPageDown);
        let preview_row = Row::new(build_cells_for_key_groups(
            "Preview navigation",
            vec![up_keys, down_keys],
            key_color,
        ));

        // Select entry
        let select_entry_keys = keys_for_action(keymap, &Action::SelectEntry);
        let select_entry_row = Row::new(build_cells_for_key_groups(
            "Select entry",
            vec![select_entry_keys],
            key_color,
        ));

        // Copy entry to clipboard
        let copy_entry_keys =
            keys_for_action(keymap, &Action::CopyEntryToClipboard);
        let copy_entry_row = Row::new(build_cells_for_key_groups(
            "Copy entry to clipboard",
            vec![copy_entry_keys],
            key_color,
        ));

        // Send to channel
        let send_to_channel_keys =
            keys_for_action(keymap, &Action::ToggleSendToChannel);
        let send_to_channel_row = Row::new(build_cells_for_key_groups(
            "Send results to",
            vec![send_to_channel_keys],
            key_color,
        ));

        // Switch channels
        let switch_channels_keys =
            keys_for_action(keymap, &Action::ToggleRemoteControl);
        let switch_channels_row = Row::new(build_cells_for_key_groups(
            "Toggle Remote control",
            vec![switch_channels_keys],
            key_color,
        ));

        // MISC line (quit, help, etc.)
        // Quit ⏼
        let quit_keys = keys_for_action(keymap, &Action::Quit);
        let quit_row = Row::new(build_cells_for_key_groups(
            "Quit",
            vec![quit_keys],
            key_color,
        ));

        let widths = vec![Constraint::Fill(1), Constraint::Fill(2)];

        Ok(Table::new(
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
        ))
    }

    fn build_keymap_table_for_channel_selection<'a>(
        &self,
    ) -> Result<Table<'a>> {
        let keymap = self.keymap_for_mode()?;
        let key_color = mode_color(self.mode);

        // Results navigation
        let prev = keys_for_action(keymap, &Action::SelectPrevEntry);
        let next = keys_for_action(keymap, &Action::SelectNextEntry);
        let results_row = Row::new(build_cells_for_key_groups(
            "Browse channels",
            vec![prev, next],
            key_color,
        ));

        // Select entry
        let select_entry_keys = keys_for_action(keymap, &Action::SelectEntry);
        let select_entry_row = Row::new(build_cells_for_key_groups(
            "Select channel",
            vec![select_entry_keys],
            key_color,
        ));

        // Remote control
        let switch_channels_keys =
            keys_for_action(keymap, &Action::ToggleRemoteControl);
        let switch_channels_row = Row::new(build_cells_for_key_groups(
            "Toggle Remote control",
            vec![switch_channels_keys],
            key_color,
        ));

        // Quit
        let quit_keys = keys_for_action(keymap, &Action::Quit);
        let quit_row = Row::new(build_cells_for_key_groups(
            "Quit",
            vec![quit_keys],
            key_color,
        ));

        Ok(Table::new(
            vec![results_row, select_entry_row, switch_channels_row, quit_row],
            vec![Constraint::Fill(1), Constraint::Fill(2)],
        ))
    }

    fn build_keymap_table_for_channel_transitions<'a>(
        &self,
    ) -> Result<Table<'a>> {
        let keymap = self.keymap_for_mode()?;
        let key_color = mode_color(self.mode);

        // Results navigation
        let prev = keys_for_action(keymap, &Action::SelectPrevEntry);
        let next = keys_for_action(keymap, &Action::SelectNextEntry);
        let results_row = Row::new(build_cells_for_key_groups(
            "Browse channels",
            vec![prev, next],
            key_color,
        ));

        // Select entry
        let select_entry_keys = keys_for_action(keymap, &Action::SelectEntry);
        let select_entry_row = Row::new(build_cells_for_key_groups(
            "Send to channel",
            vec![select_entry_keys],
            key_color,
        ));

        // Cancel
        let cancel_keys =
            keys_for_action(keymap, &Action::ToggleSendToChannel);
        let cancel_row = Row::new(build_cells_for_key_groups(
            "Cancel",
            vec![cancel_keys],
            key_color,
        ));

        // Quit
        let quit_keys = keys_for_action(keymap, &Action::Quit);
        let quit_row = Row::new(build_cells_for_key_groups(
            "Quit",
            vec![quit_keys],
            key_color,
        ));

        Ok(Table::new(
            vec![results_row, select_entry_row, cancel_row, quit_row],
            vec![Constraint::Fill(1), Constraint::Fill(2)],
        ))
    }

    /// Get the keymap for the current mode.
    ///
    /// # Returns
    /// A reference to the keymap for the current mode.
    fn keymap_for_mode(&self) -> Result<&HashMap<Key, Action>> {
        let keymap = self
            .config
            .keybindings
            .get(&self.mode)
            .ok_or_eyre("No keybindings found for the current Mode")?;
        Ok(keymap)
    }
}

/// Build the corresponding spans for a group of keys.
///
/// # Example
/// ```rust
/// use ratatui::text::Span;
/// use television::ui::help::build_spans_for_key_groups;
///
/// let key_groups = vec![
///     // alternate keys for the `SelectNextEntry` action
///     vec!["j".to_string(), "n".to_string()],
///     // alternate keys for the `SelectPrevEntry` action
///     vec!["k".to_string(), "p".to_string()],
/// ];
/// let spans = build_spans_for_key_groups("↕ Results", key_groups);
///
/// assert_eq!(spans.len(), 5);
/// ```
fn build_cells_for_key_groups(
    group_name: &str,
    key_groups: Vec<Vec<String>>,
    key_color: Color,
) -> Vec<Cell> {
    if key_groups.is_empty() || key_groups.iter().all(Vec::is_empty) {
        return vec![group_name.into(), "No keybindings".into()];
    }
    let non_empty_groups = key_groups.iter().filter(|keys| !keys.is_empty());
    let mut cells = vec![Cell::from(Span::styled(
        group_name.to_owned() + ": ",
        Style::default().fg(ACTION_COLOR),
    ))];

    let mut spans = Vec::new();

    let key_group_spans: Vec<Span> = non_empty_groups
        .map(|keys| {
            let key_group = keys.join(", ");
            Span::styled(key_group, Style::default().fg(key_color))
        })
        .collect();
    key_group_spans.iter().enumerate().for_each(|(i, span)| {
        spans.push(span.clone());
        if i < key_group_spans.len() - 1 {
            spans.push(Span::styled(" / ", Style::default().fg(key_color)));
        }
    });

    cells.push(Cell::from(Line::from(spans)));

    cells
}

/// Get the keys for a given action.
///
/// # Example
/// ```rust
/// use std::collections::HashMap;
/// use television::action::Action;
/// use television::ui::help::keys_for_action;
///
/// let mut keymap = HashMap::new();
/// keymap.insert('j', Action::SelectNextEntry);
/// keymap.insert('k', Action::SelectPrevEntry);
///
/// let keys = keys_for_action(&keymap, Action::SelectNextEntry);
///
/// assert_eq!(keys, vec!["j"]);
/// ```
fn keys_for_action(
    keymap: &HashMap<Key, Action>,
    action: &Action,
) -> Vec<String> {
    keymap
        .iter()
        .filter(|(_key, act)| *act == action)
        .map(|(key, _act)| format!("{key}"))
        .collect()
}
