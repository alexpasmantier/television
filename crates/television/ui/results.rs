use ratatui::prelude::{Color, Line, Span, Style, Stylize};
use ratatui::widgets::{Block, List, ListDirection};
use std::str::FromStr;
use crate::entry::Entry;
use crate::utils::strings::{next_char_boundary, slice_at_char_boundaries};

// Styles
const DEFAULT_RESULT_NAME_FG: Color = Color::Blue;
const DEFAULT_RESULT_PREVIEW_FG: Color = Color::Rgb(150, 150, 150);
const DEFAULT_RESULT_LINE_NUMBER_FG: Color = Color::Yellow;

pub fn build_results_list<'a, 'b>(
    results_block: Block<'b>,
    entries: &'a [Entry],
) -> List<'a>
where
    'b: 'a,
{
    List::new(entries.iter().map(|entry| {
        let mut spans = Vec::new();
        // optional icon
        if let Some(icon) = &entry.icon {
            spans.push(Span::styled(
                icon.to_string(),
                Style::default().fg(Color::from_str(icon.color).unwrap()),
            ));
            spans.push(Span::raw(" "));
        }
        // entry name
        if let Some(name_match_ranges) = &entry.name_match_ranges {
            let mut last_match_end = 0;
            for (start, end) in name_match_ranges
                .iter()
                .map(|(s, e)| (*s as usize, *e as usize))
            {
                spans.push(Span::styled(
                    slice_at_char_boundaries(
                        &entry.name,
                        last_match_end,
                        start,
                    ),
                    Style::default()
                        .fg(DEFAULT_RESULT_NAME_FG)
                        .bold()
                        .italic(),
                ));
                spans.push(Span::styled(
                    slice_at_char_boundaries(&entry.name, start, end),
                    Style::default().fg(Color::Red).bold().italic(),
                ));
                last_match_end = end;
            }
            spans.push(Span::styled(
                &entry.name[next_char_boundary(&entry.name, last_match_end)..],
                Style::default().fg(DEFAULT_RESULT_NAME_FG).bold().italic(),
            ));
        } else {
            spans.push(Span::styled(
                entry.display_name(),
                Style::default().fg(DEFAULT_RESULT_NAME_FG).bold().italic(),
            ));
        }
        // optional line number
        if let Some(line_number) = entry.line_number {
            spans.push(Span::styled(
                format!(":{line_number}"),
                Style::default().fg(DEFAULT_RESULT_LINE_NUMBER_FG),
            ));
        }
        // optional preview
        if let Some(preview) = &entry.value {
            spans.push(Span::raw(": "));

            if let Some(preview_match_ranges) = &entry.value_match_ranges {
                if !preview_match_ranges.is_empty() {
                    let mut last_match_end = 0;
                    for (start, end) in preview_match_ranges
                        .iter()
                        .map(|(s, e)| (*s as usize, *e as usize))
                    {
                        spans.push(Span::styled(
                            slice_at_char_boundaries(
                                preview,
                                last_match_end,
                                start,
                            ),
                            Style::default().fg(DEFAULT_RESULT_PREVIEW_FG),
                        ));
                        spans.push(Span::styled(
                            slice_at_char_boundaries(preview, start, end),
                            Style::default().fg(Color::Red),
                        ));
                        last_match_end = end;
                    }
                    spans.push(Span::styled(
                        &preview[next_char_boundary(
                            preview,
                            preview_match_ranges.last().unwrap().1 as usize,
                        )..],
                        Style::default().fg(DEFAULT_RESULT_PREVIEW_FG),
                    ));
                }
            } else {
                spans.push(Span::styled(
                    preview,
                    Style::default().fg(DEFAULT_RESULT_PREVIEW_FG),
                ));
            }
        }
        Line::from(spans)
    }))
    .direction(ListDirection::BottomToTop)
    .highlight_style(Style::default().bg(Color::Rgb(50, 50, 50)))
    .highlight_symbol("> ")
    .block(results_block)
}