use crate::channels::entry::Entry;
use crate::screen::colors::{Colorscheme, ResultsColorscheme};
use crate::screen::layout::InputPosition;
use crate::utils::strings::{
    make_matched_string_printable, next_char_boundary,
    slice_at_char_boundaries,
};
use color_eyre::eyre::Result;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Color, Line, Span, Style};
use ratatui::style::Stylize;
use ratatui::widgets::{
    Block, BorderType, Borders, List, ListDirection, ListState, Padding,
};
use ratatui::Frame;
use rustc_hash::{FxHashMap, FxHashSet};
use std::str::FromStr;

const POINTER_SYMBOL: &str = "> ";
const SELECTED_SYMBOL: &str = "● ";
const DESELECTED_SYMBOL: &str = "  ";

pub fn build_results_list<'a, 'b>(
    results_block: Block<'b>,
    entries: &'a [Entry],
    selected_entries: Option<&FxHashSet<Entry>>,
    list_direction: ListDirection,
    use_icons: bool,
    icon_color_cache: &mut FxHashMap<String, Color>,
    colorscheme: &ResultsColorscheme,
) -> List<'a>
where
    'b: 'a,
{
    List::new(entries.iter().map(|entry| {
        let mut spans = Vec::new();
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
                if let Some(icon_color) = icon_color_cache.get(icon.color) {
                    spans.push(Span::styled(
                        icon.to_string(),
                        Style::default().fg(*icon_color),
                    ));
                } else {
                    let icon_color = Color::from_str(icon.color).unwrap();
                    icon_color_cache
                        .insert(icon.color.to_string(), icon_color);
                    spans.push(Span::styled(
                        icon.to_string(),
                        Style::default().fg(icon_color),
                    ));
                }

                spans.push(Span::raw(" "));
            }
        }
        // entry name
        let (entry_name, name_match_ranges) = make_matched_string_printable(
            &entry.name,
            entry.name_match_ranges.as_deref(),
        );
        let mut last_match_end = 0;
        for (start, end) in name_match_ranges
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
        // optional preview
        if let Some(preview) = &entry.value {
            spans.push(Span::raw(": "));

            let (preview, preview_match_ranges) =
                make_matched_string_printable(
                    preview,
                    entry.value_match_ranges.as_deref(),
                );
            let mut last_match_end = 0;
            for (start, end) in preview_match_ranges
                .iter()
                .map(|(s, e)| (*s as usize, *e as usize))
            {
                spans.push(Span::styled(
                    slice_at_char_boundaries(&preview, last_match_end, start)
                        .to_string(),
                    Style::default().fg(colorscheme.result_preview_fg),
                ));
                spans.push(Span::styled(
                    slice_at_char_boundaries(&preview, start, end).to_string(),
                    Style::default().fg(colorscheme.match_foreground_color),
                ));
                last_match_end = end;
            }
            let next_boundary = next_char_boundary(&preview, last_match_end);
            if next_boundary < preview.len() {
                spans.push(Span::styled(
                    preview[next_boundary..].to_string(),
                    Style::default().fg(colorscheme.result_preview_fg),
                ));
            }
        }
        Line::from(spans)
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
    icon_color_cache: &mut FxHashMap<String, Color>,
    colorscheme: &Colorscheme,
    help_keybinding: &str,
    preview_keybinding: &str,
) -> Result<()> {
    let results_block = Block::default()
        .title_top(Line::from(" Results ").alignment(Alignment::Center))
        .title_bottom(
            Line::from(format!(
                " help: <{help_keybinding}>  preview: <{preview_keybinding}> "
            ))
            .alignment(Alignment::Center),
        )
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
        icon_color_cache,
        &colorscheme.results,
    );

    f.render_stateful_widget(results_list, rect, relative_picker_state);
    Ok(())
}
