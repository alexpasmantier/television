use crate::action::Action;
use crate::screen::keybinding_utils::{
    ActionMapping, extract_keys_for_actions,
};
use crate::television::Mode;
use crate::{config::KeyBindings, screen::colors::Colorscheme};
use ratatui::{
    layout::Constraint,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Cell, Row, Table},
};

/// Build a keybindings table for the help bar
pub fn build_keybindings_table<'a>(
    keybindings: &KeyBindings,
    mode: Mode,
    colorscheme: &'a Colorscheme,
) -> Table<'a> {
    match mode {
        Mode::Channel => {
            build_help_bar_table_for_channel(keybindings, colorscheme)
        }
        Mode::RemoteControl => {
            build_help_bar_table_for_remote(keybindings, colorscheme)
        }
    }
}

fn build_help_bar_table_for_channel<'a>(
    keybindings: &KeyBindings,
    colorscheme: &'a Colorscheme,
) -> Table<'a> {
    let mut rows = Vec::new();

    // Get all relevant action mappings for channel mode
    let mut all_mappings = ActionMapping::navigation_actions();
    all_mappings.extend(ActionMapping::mode_specific_actions(Mode::Channel));

    // Convert each mapping to a table row
    for mapping in all_mappings {
        let actions: Vec<Action> = mapping
            .actions
            .iter()
            .map(|(action, _)| action.clone())
            .collect();
        let keys = extract_keys_for_actions(keybindings, &actions);

        if !keys.is_empty() {
            let category_string = mapping.category.to_string();
            let row = Row::new(build_cells_for_group(
                &category_string,
                &keys,
                colorscheme.help.metadata_field_name_fg,
                colorscheme.mode.channel,
            ));
            rows.push(row);
        }
    }

    let widths = vec![Constraint::Fill(1), Constraint::Fill(2)];
    Table::new(rows, widths)
}

fn build_help_bar_table_for_remote<'a>(
    keybindings: &KeyBindings,
    colorscheme: &'a Colorscheme,
) -> Table<'a> {
    let mut rows = Vec::new();

    // Get all relevant action mappings for remote control mode
    let mut all_mappings = ActionMapping::navigation_actions();
    all_mappings
        .extend(ActionMapping::mode_specific_actions(Mode::RemoteControl));

    // Convert each mapping to a table row with custom labels
    for mapping in all_mappings {
        let actions: Vec<Action> = mapping
            .actions
            .iter()
            .map(|(action, _)| action.clone())
            .collect();
        let keys = extract_keys_for_actions(keybindings, &actions);

        if !keys.is_empty() {
            let category_string = mapping.category.to_string();
            let display_name = match category_string.as_str() {
                "Results navigation" => "Browse channels",
                "Select entry" => "Select channel",
                other => other,
            };

            let row = Row::new(build_cells_for_group(
                display_name,
                &keys,
                colorscheme.help.metadata_field_name_fg,
                colorscheme.mode.remote_control,
            ));
            rows.push(row);
        }
    }

    Table::new(rows, vec![Constraint::Fill(1), Constraint::Fill(2)])
}

fn build_cells_for_group<'a>(
    group_name: &str,
    keys: &[String],
    key_color: Color,
    value_color: Color,
) -> Vec<Cell<'a>> {
    if keys.is_empty() {
        return vec![Cell::from(""), Cell::from("")];
    }

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
