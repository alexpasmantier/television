use crate::channels::entry::Entry;
use crate::screen::colors::{Colorscheme, ResultsColorscheme};
use crate::screen::layout::InputPosition;
use crate::utils::indices::truncate_highlighted_string;
use crate::utils::strings::{
    make_matched_string_printable, next_char_boundary,
    slice_at_char_boundaries,
};
use anyhow::Result;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Color, Line, Span, Style};
use ratatui::style::Stylize;
use ratatui::widgets::{
    Block, BorderType, Borders, List, ListDirection, ListState, Padding,
};
use ratatui::Frame;
use rustc_hash::FxHashSet;
use std::str::FromStr;

const POINTER_SYMBOL: &str = "> ";
const SELECTED_SYMBOL: &str = "● ";
const DESELECTED_SYMBOL: &str = "  ";

/// The max width for each part of the entry (name and value) depending on various factors.
///
/// - name only: `available_width - 2 * (use_icons as u16) - 2 * (is_selected as u16) - line_number_width`
/// - name and value: `(available_width - 2 * (use_icons as u16) - 2 * (is_selected as u16) - line_number_width) / 2`
fn max_widths(
    entry: &Entry,
    available_width: u16,
    use_icons: bool,
    is_selected: bool,
) -> (u16, u16) {
    let available_width = available_width.saturating_sub(
        2 // pointer and space
            + 2 * (u16::from(use_icons))
            + 2 * (u16::from(is_selected))
            + entry
                .line_number
                // ":{line_number}: "
                .map_or(0, |l| 1 + u16::try_from(l.checked_ilog10().unwrap_or(0)).unwrap() + 3),
    );

    if entry.value.is_none() {
        return (available_width, 0);
    }

    // otherwise, use up the available space for both name and value as nicely as possible
    let name_len =
        u16::try_from(entry.name.chars().count()).unwrap_or(u16::MAX);
    let value_len = entry
        .value
        .as_ref()
        .map_or(0, |v| u16::try_from(v.chars().count()).unwrap_or(u16::MAX));

    if name_len < available_width / 2 {
        (name_len, available_width - name_len - 2)
    } else if value_len < available_width / 2 {
        (available_width - value_len, value_len - 2)
    } else {
        (available_width / 2, available_width / 2 - 2)
    }
}

fn build_result_line<'a>(
    entry: &'a Entry,
    selected_entries: Option<&FxHashSet<Entry>>,
    use_icons: bool,
    colorscheme: &ResultsColorscheme,
    area_width: u16,
) -> Line<'a> {
    let mut spans = Vec::new();
    let (name_max_width, value_max_width) = max_widths(
        entry,
        area_width,
        use_icons,
        selected_entries.map_or(false, |selected| selected.contains(entry)),
    );
    // optional selection symbol
    if let Some(selected_entries) = selected_entries {
        if !selected_entries.is_empty() {
            spans.push(if selected_entries.contains(entry) {
                Span::styled(
                    SELECTED_SYMBOL,
                    Style::default().fg(colorscheme.result_selected_fg),
                )
            } else {
                Span::from(DESELECTED_SYMBOL)
            });
        }
    }
    // optional icon
    if let Some(icon) = entry.icon.as_ref() {
        if use_icons {
            spans.push(Span::styled(
                icon.to_string(),
                Style::default().fg(Color::from_str(icon.color).unwrap()),
            ));

            spans.push(Span::raw(" "));
        }
    }
    // entry name
    let (mut entry_name, mut value_match_ranges) =
        make_matched_string_printable(
            &entry.name,
            entry.name_match_ranges.as_deref(),
        );
    // if the name is too long, we need to truncate it and add an ellipsis
    if entry_name.len() > name_max_width as usize {
        (entry_name, value_match_ranges) = truncate_highlighted_string(
            &entry_name,
            &value_match_ranges,
            name_max_width,
        );
    }
    let mut last_match_end = 0;
    for (start, end) in value_match_ranges
        .iter()
        .map(|(s, e)| (*s as usize, *e as usize))
    {
        // from the end of the last match to the start of the current one
        spans.push(Span::styled(
            slice_at_char_boundaries(&entry_name, last_match_end, start)
                .to_string(),
            Style::default().fg(colorscheme.result_name_fg),
        ));
        // the current match
        spans.push(Span::styled(
            slice_at_char_boundaries(&entry_name, start, end).to_string(),
            Style::default().fg(colorscheme.match_foreground_color),
        ));
        last_match_end = end;
    }
    // we need to push a span for the remainder of the entry name
    // but only if there's something left
    let next_boundary = next_char_boundary(&entry_name, last_match_end);
    if next_boundary < entry_name.len() {
        let remainder = entry_name[next_boundary..].to_string();
        spans.push(Span::styled(
            remainder,
            Style::default().fg(colorscheme.result_name_fg),
        ));
    }
    // optional line number
    if let Some(line_number) = entry.line_number {
        spans.push(Span::styled(
            format!(":{line_number}"),
            Style::default().fg(colorscheme.result_line_number_fg),
        ));
    }
    // optional value
    if let Some(value) = &entry.value {
        spans.push(Span::raw(": "));

        let (mut value, mut value_match_ranges) =
            make_matched_string_printable(
                value,
                entry.value_match_ranges.as_deref(),
            );
        // if the value is too long, we need to truncate it and add an ellipsis
        if value.len() > value_max_width as usize {
            (value, value_match_ranges) = truncate_highlighted_string(
                &value,
                &value_match_ranges,
                value_max_width,
            );
        }

        let mut last_match_end = 0;
        for (start, end) in value_match_ranges
            .iter()
            .map(|(s, e)| (*s as usize, *e as usize))
        {
            spans.push(Span::styled(
                slice_at_char_boundaries(&value, last_match_end, start)
                    .to_string(),
                Style::default().fg(colorscheme.result_preview_fg),
            ));
            spans.push(Span::styled(
                slice_at_char_boundaries(&value, start, end).to_string(),
                Style::default().fg(colorscheme.match_foreground_color),
            ));
            last_match_end = end;
        }
        let next_boundary = next_char_boundary(&value, last_match_end);
        if next_boundary < value.len() {
            spans.push(Span::styled(
                value[next_boundary..].to_string(),
                Style::default().fg(colorscheme.result_preview_fg),
            ));
        }
    }
    Line::from(spans)
}

pub fn build_results_list<'a, 'b>(
    results_block: Block<'b>,
    entries: &'a [Entry],
    selected_entries: Option<&FxHashSet<Entry>>,
    list_direction: ListDirection,
    use_icons: bool,
    colorscheme: &ResultsColorscheme,
    area_width: u16,
) -> List<'a>
where
    'b: 'a,
{
    List::new(entries.iter().map(|entry| {
        build_result_line(
            entry,
            selected_entries,
            use_icons,
            colorscheme,
            area_width,
        )
    }))
    .direction(list_direction)
    .highlight_style(
        Style::default().bg(colorscheme.result_selected_bg).bold(),
    )
    .highlight_symbol(POINTER_SYMBOL)
    .block(results_block)
}

#[allow(clippy::too_many_arguments)]
pub fn draw_results_list(
    f: &mut Frame,
    rect: Rect,
    entries: &[Entry],
    selected_entries: &FxHashSet<Entry>,
    relative_picker_state: &mut ListState,
    input_bar_position: InputPosition,
    use_nerd_font_icons: bool,
    colorscheme: &Colorscheme,
    help_keybinding: &str,
    preview_keybinding: &str,
    preview_togglable: bool,
) -> Result<()> {
    let mut toggle_hints = format!(" help: <{help_keybinding}> ",);
    if preview_togglable {
        toggle_hints.push_str(&format!(" preview: <{preview_keybinding}> ",));
    }

    let results_block = Block::default()
        .title_top(Line::from(" Results ").alignment(Alignment::Center))
        .title_bottom(Line::from(toggle_hints).alignment(Alignment::Center))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colorscheme.general.border_fg))
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        )
        .padding(Padding::right(1));

    let results_list = build_results_list(
        results_block,
        entries,
        Some(selected_entries),
        match input_bar_position {
            InputPosition::Bottom => ListDirection::BottomToTop,
            InputPosition::Top => ListDirection::TopToBottom,
        },
        use_nerd_font_icons,
        &colorscheme.results,
        rect.width - 1, // right padding
    );

    f.render_stateful_widget(results_list, rect, relative_picker_state);
    Ok(())
}
