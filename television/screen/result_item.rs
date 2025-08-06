use crate::{
    event::Key,
    screen::{
        colors::ResultsColorscheme,
        constants::{DESELECTED_SYMBOL, POINTER_SYMBOL, SELECTED_SYMBOL},
    },
    utils::{
        indices::truncate_highlighted_string,
        strings::make_result_item_printable,
    },
};
use ansi_to_tui::IntoText;
use anyhow::Result;
use devicons::FileIcon;
use ratatui::{
    prelude::{Color, Line, Span, Style},
    style::Stylize,
    widgets::{Block, List, ListDirection, ListState},
};
use unicode_width::UnicodeWidthStr;

/// Trait implemented by any item that can be displayed in the results or remote-control list.
pub trait ResultItem {
    /// Returns the raw string representation of the item.
    fn raw(&self) -> &str;

    /// Returns an optional icon to display in front of the item.
    fn icon(&self) -> Option<&FileIcon> {
        None
    }

    /// The main text representing the item.
    fn display(&self) -> &str;

    /// The output string that will be used when the item is selected.
    fn output(&self) -> Result<String>;

    /// Highlight match ranges (char based indices) within `display()`.
    ///
    /// These are contiguous ranges of character indices that should be highlighted.
    fn match_ranges(&self) -> Option<&[(u32, u32)]> {
        None
    }

    /// Optional shortcut binding shown after the name (remote-control entries).
    fn shortcut(&self) -> Option<&Key> {
        None
    }

    /// Whether the item uses ANSI escape codes for styling.
    fn ansi(&self) -> bool {
        false
    }
}

/// Build a single `Line` for a [`ResultItem`].
#[allow(clippy::too_many_arguments)]
#[allow(clippy::cast_possible_truncation)]
pub fn build_result_line<'a, T: ResultItem + ?Sized>(
    item: &'a T,
    selection_fg: Color,
    result_fg: Color,
    match_fg: Color,
    area_width: u16,
    // Some(true)=selected ●, Some(false)=unselected, None=no prefix
    prefix: Option<bool>,
) -> Line<'a> {
    // PERF: Pre-allocate spans vector with estimated capacity
    let mut spans = Vec::<Span<'a>>::with_capacity(16);

    // Optional selection prefix
    if let Some(selected) = prefix {
        if selected {
            spans.push(Span::styled(
                SELECTED_SYMBOL,
                Style::default().fg(selection_fg),
            ));
        } else {
            spans.push(Span::raw(DESELECTED_SYMBOL));
        }
    }

    let selection_prefix_width: u16 = if prefix.is_some() { 2 } else { 0 };

    let shortcut_extra: u16 = item
        .shortcut()
        .map(|k| 2 + k.to_string().len() as u16) // space + key
        .unwrap_or(0);

    let item_max_width = area_width
        .saturating_sub(2) // pointer + space (kept for caller)
        .saturating_sub(2) // borders
        .saturating_sub(selection_prefix_width)
        .saturating_sub(shortcut_extra);

    if item.ansi() {
        spans.extend(build_entry_spans_ansi(
            item,
            item_max_width,
            result_fg,
            match_fg,
        ));
    } else {
        spans.extend(build_entry_spans(
            item,
            item_max_width,
            result_fg,
            match_fg,
        ));
    }

    // Show shortcut if present.
    if let Some(key) = item.shortcut() {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            key.to_string(),
            Style::default().fg(match_fg),
        ));
    }

    Line::from(spans)
}

fn build_entry_spans<T: ResultItem + ?Sized>(
    item: &T,
    max_width: u16,
    result_fg: Color,
    match_fg: Color,
) -> Vec<Span> {
    let mut spans = Vec::with_capacity(16);

    let (mut entry_name, mut match_ranges) = make_result_item_printable(item);

    // Truncate if too long.
    if UnicodeWidthStr::width(entry_name.as_str()) > max_width as usize {
        let (name, ranges) =
            truncate_highlighted_string(&entry_name, &match_ranges, max_width);
        entry_name = name;
        match_ranges = ranges;
    }

    if match_ranges.is_empty() {
        spans.push(Span::styled(entry_name, Style::default().fg(result_fg)));
        return spans;
    }

    let chars: Vec<char> = entry_name.chars().collect();
    let mut idx = 0;
    for &(start, end) in &match_ranges {
        let start = start as usize;
        let end = end as usize;
        if idx < start {
            let text: String = chars[idx..start].iter().collect();
            if !text.is_empty() {
                spans.push(Span::styled(text, Style::default().fg(result_fg)));
            }
        }
        if start < end {
            let text: String = chars[start..end].iter().collect();
            if !text.is_empty() {
                spans.push(Span::styled(text, Style::default().fg(match_fg)));
            }
        }
        idx = end;
    }
    if idx < chars.len() {
        let text: String = chars[idx..].iter().collect();
        if !text.is_empty() {
            spans.push(Span::styled(text, Style::default().fg(result_fg)));
        }
    }
    spans
}

/// Builds a vector of [`Span`]s for a [`ResultItem`] that may contain ANSI escape codes.
///
/// # Algorithm
///
/// - 1/ The function parses the raw string of the item into styled spans using `ansi_to_tui`.
///
/// - 2/ If the parsed result contains only a single span with the default style (i.e., no ANSI codes),
///   it falls back to the simpler `build_entry_spans` function.
///
/// - 3/ Otherwise, it iterates over the parsed spans and overlays the match highlight ranges onto them.
///    - It tracks the current character position across all spans.
///    - For each span, it walks through its characters, checking if the current position falls within
///      any of the match highlight ranges.
///    - If a highlight range is encountered, it splits the span into sub-spans:
///        - Unhighlighted text before the match uses the original style.
///        - Highlighted text within the match uses the original style but overrides the foreground color with `match_fg`.
///        - Remaining text after the match continues with the original style.
///    - The algorithm advances through both the spans and the highlight ranges, ensuring that highlights
///      are applied correctly even if they cross span boundaries.
/// - 4/ The result is a vector of spans that preserves the original ANSI styling, but overlays match highlights
///   as specified by the input ranges.
///
/// # Parameters
/// - `item`: The result item to render.
/// - `max_width`: The maximum width for the rendered line (currently not used for truncation in this function).
/// - `result_fg`: The default foreground color for non-highlighted text.
/// - `match_fg`: The foreground color to use for highlighted (matched) text.
///
/// # Returns
/// A vector of [`Span`]s, each with appropriate styling and highlighting.
///
/// # Notes
/// - This function is designed to work with items that use ANSI escape codes for styling.
/// - It ensures that match highlights do not disrupt the underlying ANSI styles, except for the foreground color.
/// - If no ANSI codes are present, it delegates to the simpler span builder for efficiency.
fn build_entry_spans_ansi<T: ResultItem + ?Sized>(
    item: &T,
    max_width: u16,
    result_fg: Color,
    match_fg: Color,
) -> Vec<Span> {
    let text = item.raw();
    let match_ranges = item.match_ranges().unwrap_or(&[]);
    let parsed = text.into_text().unwrap();
    let spans = &parsed.lines[0].spans;

    // If there are no ANSI codes, fall back to the simple span builder
    if spans.len() == 1 && spans[0].style == Style::default() {
        return build_entry_spans(item, max_width, result_fg, match_fg);
    }

    // hypothesis: ~ 2 to 3 highlighted clusters + in the worst case scenario
    // each cluster splits its containing span into 3 parts -> + 6 spans so we
    // should be fine pre-allocating `spans.len() + 8`
    let mut highlighted_spans: Vec<Span> = Vec::with_capacity(spans.len() + 8);
    let mut hl_ranges = match_ranges
        .iter()
        .map(|(start, end)| (*start as usize, *end as usize))
        .peekable();
    let mut char_pos = 0;

    for span in spans {
        let span_len = span.content.chars().count();
        let mut cursor = 0;
        while cursor < span_len {
            if let Some(&(hl_start, hl_end)) = hl_ranges.peek() {
                if char_pos + cursor >= hl_end {
                    hl_ranges.next();
                    continue;
                }
                let highlight_start = hl_start.saturating_sub(char_pos);
                let highlight_end = hl_end.saturating_sub(char_pos);

                // note that the following will also just push the entire span if
                // the highlight range comes anywhere after the current span
                if cursor < highlight_start {
                    let s: String = span
                        .content
                        .chars()
                        .skip(cursor)
                        .take(highlight_start - cursor)
                        .collect();
                    if !s.is_empty() {
                        highlighted_spans.push(Span::styled(s, span.style));
                    }
                    cursor = highlight_start;
                } else {
                    let s: String = span
                        .content
                        .chars()
                        .skip(cursor)
                        .take(highlight_end - cursor)
                        .collect();
                    if !s.is_empty() {
                        highlighted_spans
                            .push(Span::styled(s, span.style.fg(match_fg)));
                    }
                    cursor = highlight_end;
                }
            } else {
                let s: String = span.content.chars().skip(cursor).collect();
                if !s.is_empty() {
                    highlighted_spans.push(Span::styled(s, span.style));
                }
                break;
            }
        }
        char_pos += span_len;
    }
    highlighted_spans
}

/// Build a `List` widget from a slice of [`ResultItem`]s.
#[allow(clippy::too_many_arguments)]
pub fn build_results_list<'a, 'b, T, F>(
    block: Block<'b>,
    entries: &'a [T],
    relative_picker_state: &ListState,
    list_direction: ListDirection,
    colorscheme: &ResultsColorscheme,
    area_width: u16,
    mut prefix_fn: F,
) -> List<'a>
where
    'b: 'a,
    T: ResultItem,
    F: FnMut(&T) -> Option<bool>,
{
    List::new(entries.iter().enumerate().map(|(i, e)| {
        let prefix = prefix_fn(e);
        let result_fg = if relative_picker_state.selected() == Some(i) {
            colorscheme.result_selected_fg
        } else {
            colorscheme.result_fg
        };
        build_result_line(
            e,
            colorscheme.result_selected_fg,
            result_fg,
            colorscheme.match_foreground_color,
            area_width,
            prefix,
        )
    }))
    .direction(list_direction)
    .highlight_style(
        Style::default().bg(colorscheme.result_selected_bg).bold(),
    )
    .highlight_symbol(POINTER_SYMBOL)
    .block(block)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channels::entry::Entry;
    use ratatui::prelude::{Color, Span};
    use ratatui::text::Line;

    #[test]
    fn test_build_result_line_simple() {
        let entry = Entry::new("something nice".to_string())
            .with_match_indices(&[1, 2, 10, 11]);
        let line = build_result_line(
            &entry,
            Color::Reset,
            Color::Reset,
            Color::Reset,
            200,
            None,
        );

        let expected = Line::from(vec![
            Span::raw("s").fg(Color::Reset),
            Span::raw("om").fg(Color::Reset),
            Span::raw("ething ").fg(Color::Reset),
            Span::raw("ni").fg(Color::Reset),
            Span::raw("ce").fg(Color::Reset),
        ]);

        assert_eq!(line, expected);
    }

    #[test]
    fn test_build_result_line_truncate_multibyte() {
        let entry = Entry::new("ジェイムス下地 - REDLINE Original Soundtrack - 06 - ROBOWORLD TV.mp3".to_string())
            .with_match_indices(&[27, 28, 29, 30, 31]);
        // area width tuned so that text should be truncated with ellipsis
        let line = build_result_line(
            &entry,
            Color::Reset,
            Color::Reset,
            Color::Reset,
            20, // small width
            None,
        );

        // We expect the resulting string to contain the ellipsis char
        let rendered: String =
            line.spans.iter().map(|s| s.content.clone()).collect();
        assert!(rendered.contains('…'));
    }

    #[test]
    fn test_build_entry_spans_ansi_no_ansi() {
        let entry = Entry::new("A simple string".to_string())
            .with_match_indices(&[3, 4, 5]);
        let spans =
            build_entry_spans_ansi(&entry, 200, Color::Blue, Color::Red);
        let blue_fg = Style::default().fg(Color::Blue);

        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0], Span::styled("A s", blue_fg));
        assert_eq!(
            spans[1],
            Span::styled("imp", Style::default().fg(Color::Red))
        );
        assert_eq!(spans[2], Span::styled("le string", blue_fg));
    }

    #[test]
    fn test_build_entry_spans_ansi_no_ansi_corner_cases() {
        let entry = Entry::new("A".to_string()).with_match_indices(&[0]);
        let spans =
            build_entry_spans_ansi(&entry, 200, Color::Reset, Color::Red);

        assert_eq!(spans.len(), 1);
        assert_eq!(
            spans[0],
            Span::styled("A", Style::default().fg(Color::Red))
        );

        let entry = Entry::new(String::new()).with_match_indices(&[]);
        let spans =
            build_entry_spans_ansi(&entry, 200, Color::Reset, Color::Red);

        assert!(spans.is_empty());

        let entry = Entry::new("A".to_string()).with_match_indices(&[]);
        let spans =
            build_entry_spans_ansi(&entry, 200, Color::Reset, Color::Red);

        assert_eq!(spans.len(), 1);
        assert_eq!(
            spans[0],
            Span::styled("A", Style::default().fg(Color::Reset))
        );
    }

    #[test]
    fn test_build_entry_spans_ansi_with_ansi() {
        let entry = Entry::new(
            "\x1b[31mRed\x1b[0m and \x1b[32mGreen\x1b[0m".to_string(),
        )
        .with_match_indices(&[1, 4, 5]);
        let spans =
            build_entry_spans_ansi(&entry, 200, Color::Blue, Color::Yellow);

        assert_eq!(
            spans.len(),
            7,
            "Expected 7 spans but got {:?}",
            spans
                .iter()
                .map(|s| (s.content.clone(), s.style.fg))
                .collect::<Vec<_>>()
        );
        assert_eq!(spans[0], Span::raw("R").fg(Color::Red));
        assert_eq!(spans[1], Span::raw("e").fg(Color::Yellow));
        assert_eq!(spans[2], Span::raw("d").fg(Color::Red));
        assert_eq!(spans[3].content, Span::raw(" ").content);
        assert_eq!(spans[3].style, Style::reset());
        assert_eq!(spans[4], Span::raw("an").reset().fg(Color::Yellow));
        assert_eq!(spans[5], Span::raw("d ").reset());
        assert_eq!(spans[6], Span::raw("Green").reset().fg(Color::Green));
    }

    #[test]
    fn test_build_entry_spans_full_string_highlight() {
        let entry = Entry::new("highlight me".to_string())
            .with_match_indices(&(0..12).collect::<Vec<_>>());
        let spans = build_entry_spans(&entry, 200, Color::Blue, Color::Red);

        // All chars should be highlighted
        assert_eq!(spans.len(), 1);
        assert_eq!(
            spans[0],
            Span::styled("highlight me", Style::default().fg(Color::Red))
        );
    }

    #[test]
    fn test_build_entry_spans_match_at_boundaries() {
        let entry =
            Entry::new("boundary".to_string()).with_match_indices(&[0, 7]);
        let spans = build_entry_spans(&entry, 200, Color::Blue, Color::Red);

        assert_eq!(
            spans[0],
            Span::styled("b", Style::default().fg(Color::Red))
        );
        assert_eq!(
            spans[1],
            Span::styled("oundar", Style::default().fg(Color::Blue))
        );
        assert_eq!(
            spans[2],
            Span::styled("y", Style::default().fg(Color::Red))
        );
    }

    #[test]
    fn test_build_entry_spans_unicode_boundaries() {
        let entry = Entry::new("a😀b".to_string()).with_match_indices(&[1]); // highlight the emoji only
        let spans = build_entry_spans(&entry, 200, Color::Blue, Color::Red);

        assert_eq!(
            spans[0],
            Span::styled("a", Style::default().fg(Color::Blue))
        );
        assert_eq!(
            spans[1],
            Span::styled("😀", Style::default().fg(Color::Red))
        );
        assert_eq!(
            spans[2],
            Span::styled("b", Style::default().fg(Color::Blue))
        );
    }
}
