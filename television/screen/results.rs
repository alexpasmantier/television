use crate::channels::entry::Entry;
use crate::screen::colors::{Colorscheme, ResultsColorscheme};
use crate::screen::layout::InputPosition;
use crate::utils::indices::truncate_highlighted_string;
use crate::utils::strings::make_matched_string_printable;
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
use unicode_width::UnicodeWidthStr;

const POINTER_SYMBOL: &str = "> ";
const SELECTED_SYMBOL: &str = "● ";
const DESELECTED_SYMBOL: &str = "  ";

/// The max width for each part of the entry (name and value) depending on various factors.
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
            + 2 // borders
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
        (name_len, available_width - name_len)
    } else if value_len < available_width / 2 {
        (available_width - value_len, value_len)
    } else {
        (available_width / 2, available_width / 2)
    }
}

// TODO: could we not just iterate on chars here instead of using the indices?
// that would avoid quite some computation during the rendering and might fix multibyte char
// issues (nucleo's indices are actually char-based)
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
    let (mut entry_name, mut name_match_ranges) =
        make_matched_string_printable(
            &entry.name,
            entry.name_match_ranges.as_deref(),
        );
    // if the name is too long, we need to truncate it and add an ellipsis
    if entry_name.as_str().width() > name_max_width as usize {
        (entry_name, name_match_ranges) = truncate_highlighted_string(
            &entry_name,
            &name_match_ranges,
            name_max_width,
        );
    }

    let mut last_match_end = 0;
    let name_chars = entry_name.chars();
    let name_len = entry_name.as_str().width();
    for (start, end) in name_match_ranges
        .iter()
        .map(|(s, e)| (*s as usize, *e as usize))
    {
        // from the end of the last match to the start of the current one
        spans.push(Span::styled(
            name_chars
                .clone()
                .skip(last_match_end)
                .take(start - last_match_end)
                .collect::<String>(),
            //entry_name[last_match_end..start].to_string(),
            Style::default().fg(colorscheme.result_name_fg),
        ));
        // the current match
        spans.push(Span::styled(
            name_chars
                .clone()
                .skip(start)
                .take(end - start)
                .collect::<String>(),
            Style::default().fg(colorscheme.match_foreground_color),
        ));
        last_match_end = end;
    }
    // we need to push a span for the remainder of the entry name
    // but only if there's something left
    if last_match_end < name_len {
        let remainder = name_chars.skip(last_match_end).collect::<String>();
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
        if value.as_str().width() > value_max_width as usize {
            (value, value_match_ranges) = truncate_highlighted_string(
                &value,
                &value_match_ranges,
                value_max_width,
            );
        }

        let mut last_match_end = 0;
        let value_chars = value.chars();
        let value_len = value.chars().count();
        for (start, end) in value_match_ranges
            .iter()
            .map(|(s, e)| (*s as usize, *e as usize))
        {
            spans.push(Span::styled(
                value_chars
                    .clone()
                    .skip(last_match_end)
                    .take(start - last_match_end)
                    .collect::<String>(),
                Style::default().fg(colorscheme.result_preview_fg),
            ));
            spans.push(Span::styled(
                value_chars
                    .clone()
                    .skip(start)
                    .take(end - start)
                    .collect::<String>(),
                Style::default().fg(colorscheme.match_foreground_color),
            ));
            last_match_end = end;
        }
        if last_match_end < value_len {
            spans.push(Span::styled(
                value_chars.skip(last_match_end).collect::<String>(),
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
    no_help: bool,
) -> Result<()> {
    let mut toggle_hints = String::new();
    if !no_help {
        toggle_hints.push_str(&format!(" help: <{help_keybinding}> ",));
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_result_line() {
        let entry = Entry::new(String::from("something nice"))
            .with_name_match_indices(
                // something nice
                // 012345678901234
                //  om       ni
                &[1, 2, 10, 11],
            );
        let result_line = build_result_line(
            &entry,
            None,
            false,
            &ResultsColorscheme::default(),
            200,
        );

        let expected_line = Line::from(vec![
            Span::raw("s").fg(Color::Reset),
            Span::raw("om").fg(Color::Reset),
            Span::raw("ething ").fg(Color::Reset),
            Span::raw("ni").fg(Color::Reset),
            Span::raw("ce").fg(Color::Reset),
        ]);

        assert_eq!(result_line, expected_line);
    }

    #[test]
    fn test_build_result_line_multibyte_chars() {
        let entry =
            // See https://github.com/alexpasmantier/television/issues/439
            Entry::new(String::from("ジェイムス下地 - REDLINE Original Soundtrack - 06 - ROBOWORLD TV.mp3"))
                .with_name_match_indices(&[27, 28, 29, 30, 31]);
        let result_line = build_result_line(
            &entry,
            None,
            false,
            &ResultsColorscheme::default(),
            // 16 + (borders + (pointer & space))
            16 + 2 + 2,
        );

        let expected_line = Line::from(vec![
            Span::raw("…Original ").fg(Color::Reset),
            Span::raw("Sound").fg(Color::Reset),
            Span::raw("…").fg(Color::Reset),
        ]);

        assert_eq!(result_line, expected_line);
    }
}
