use color_eyre::eyre::{OptionExt, Result};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use std::collections::HashMap;

use crate::{
    action::Action,
    event::Key,
    television::{Mode, Television},
};

const SEPARATOR: &str = "  ";
const ACTION_COLOR: Color = Color::DarkGray;
const KEY_COLOR: Color = Color::LightYellow;

impl Television {
    pub fn build_help_paragraph<'a>(&self) -> Result<Paragraph<'a>> {
        match self.mode {
            Mode::Channel => self.build_help_paragraph_for_channel(),
            Mode::ChannelSelection => {
                self.build_help_paragraph_for_channel_selection()
            }
            Mode::SendToChannel => self.build_help_paragraph_for_channel(),
        }
    }

    fn build_help_paragraph_for_channel<'a>(&self) -> Result<Paragraph<'a>> {
        let keymap = self.keymap_for_mode()?;
        let mut lines = Vec::new();

        // NAVIGATION and SELECTION line
        let mut ns_line = Line::default();

        // Results navigation
        let prev = keys_for_action(keymap, Action::SelectPrevEntry);
        let next = keys_for_action(keymap, Action::SelectNextEntry);
        let results_spans =
            build_spans_for_key_groups("↕ Results", vec![prev, next]);

        ns_line.extend(results_spans);
        ns_line.push_span(Span::styled(SEPARATOR, Style::default()));

        // Preview navigation
        let up_keys = keys_for_action(keymap, Action::ScrollPreviewHalfPageUp);
        let down_keys =
            keys_for_action(keymap, Action::ScrollPreviewHalfPageDown);
        let preview_spans =
            build_spans_for_key_groups("↕ Preview", vec![up_keys, down_keys]);

        ns_line.extend(preview_spans);
        ns_line.push_span(Span::styled(SEPARATOR, Style::default()));

        // Send to channel
        let send_to_channel_keys =
            keys_for_action(keymap, Action::SendToChannel);
        // TODO: add send icon
        let send_to_channel_spans =
            build_spans_for_key_groups("Send to", vec![send_to_channel_keys]);

        ns_line.extend(send_to_channel_spans);
        ns_line.push_span(Span::styled(SEPARATOR, Style::default()));

        // Select entry
        let select_entry_keys = keys_for_action(keymap, Action::SelectEntry);
        let select_entry_spans = build_spans_for_key_groups(
            "Select entry",
            vec![select_entry_keys],
        );

        ns_line.extend(select_entry_spans);
        ns_line.push_span(Span::styled(SEPARATOR, Style::default()));

        // Switch channels
        let switch_channels_keys =
            keys_for_action(keymap, Action::ToggleChannelSelection);
        let switch_channels_spans = build_spans_for_key_groups(
            "Switch channels",
            vec![switch_channels_keys],
        );

        ns_line.extend(switch_channels_spans);
        lines.push(ns_line);

        // MISC line (quit, help, etc.)
        // let mut misc_line = Line::default();
        //
        // // Quit
        // let quit_keys = keys_for_action(keymap, Action::Quit);
        // let quit_spans = build_spans_for_key_groups("Quit", vec![quit_keys]);
        //
        // misc_line.extend(quit_spans);
        //
        // lines.push(misc_line);

        Ok(Paragraph::new(lines))
    }

    fn build_help_paragraph_for_channel_selection<'a>(
        &self,
    ) -> Result<Paragraph<'a>> {
        let keymap = self.keymap_for_mode()?;
        let mut lines = Vec::new();

        // NAVIGATION + SELECTION line
        let mut ns_line = Line::default();

        // Results navigation
        let prev = keys_for_action(keymap, Action::SelectPrevEntry);
        let next = keys_for_action(keymap, Action::SelectNextEntry);
        let results_spans =
            build_spans_for_key_groups("↕ Results", vec![prev, next]);

        ns_line.extend(results_spans);
        ns_line.push_span(Span::styled(SEPARATOR, Style::default()));

        // Select entry
        let select_entry_keys = keys_for_action(keymap, Action::SelectEntry);
        let select_entry_spans = build_spans_for_key_groups(
            "Select entry",
            vec![select_entry_keys],
        );

        ns_line.extend(select_entry_spans);
        ns_line.push_span(Span::styled(SEPARATOR, Style::default()));

        // Switch channels
        let switch_channels_keys =
            keys_for_action(keymap, Action::ToggleChannelSelection);
        let switch_channels_spans = build_spans_for_key_groups(
            "Switch channels",
            vec![switch_channels_keys],
        );

        ns_line.extend(switch_channels_spans);
        
        lines.push(ns_line);

        // MISC line (quit, help, etc.)
        // let mut misc_line = Line::default();

        // Quit
        // let quit_keys = keys_for_action(keymap, Action::Quit);
        // let quit_spans = build_spans_for_key_groups("Quit", vec![quit_keys]);

        // misc_line.extend(quit_spans);

        // lines.push(misc_line);

        Ok(Paragraph::new(lines))
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
/// # Arguments
///     - `group_name`: The name of the group.
///     - `key_groups`: A vector of vectors of strings representing the keys for each group.
///        Each vector of strings represents a group of alternate keys for a given `Action`.
///
/// # Returns
/// A vector of `Span`s representing the key groups.
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
fn build_spans_for_key_groups(
    group_name: &str,
    key_groups: Vec<Vec<String>>,
) -> Vec<Span> {
    if key_groups.is_empty() || key_groups.iter().all(|keys| keys.is_empty()) {
        return vec![];
    }
    let non_empty_groups = key_groups.iter().filter(|keys| !keys.is_empty());
    let mut spans = vec![
        Span::styled(
            group_name.to_owned() + ": ",
            Style::default().fg(ACTION_COLOR),
        ),
        Span::styled("[", Style::default().fg(KEY_COLOR)),
    ];
    let key_group_spans: Vec<Span> = non_empty_groups
        .map(|keys| {
            let key_group = keys.join(", ");
            Span::styled(key_group, Style::default().fg(KEY_COLOR))
        })
        .collect();
    key_group_spans.iter().enumerate().for_each(|(i, span)| {
        spans.push(span.clone());
        if i < key_group_spans.len() - 1 {
            spans.push(Span::styled(" | ", Style::default().fg(KEY_COLOR)));
        }
    });

    spans.push(Span::styled("]", Style::default().fg(KEY_COLOR)));
    spans
}

/// Get the keys for a given action.
///
/// # Arguments
///    - `keymap`: A hashmap of keybindings.
///    - `action`: The action to get the keys for.
///
/// # Returns
/// A vector of strings representing the keys for the given action.
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
    action: Action,
) -> Vec<String> {
    keymap
        .iter()
        .filter(|(_key, act)| **act == action)
        .map(|(key, _act)| format!("{key}"))
        .collect()
}
