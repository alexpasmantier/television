use crate::{
    event::Key,
    screen::{
        colors::ResultsColorscheme,
        constants::{DESELECTED_SYMBOL, SELECTED_SYMBOL},
    },
    utils::{
        indices::truncate_highlighted_string,
        strings::{
            ReplaceNonPrintableConfig, make_result_item_printable,
            replace_non_printable_bulk,
        },
    },
};
use anyhow::Result;
use ratatui::{
    prelude::{Color, Line, Span, Style},
    text::Text,
    widgets::{Block, List, ListDirection, ListState},
};
use unicode_width::UnicodeWidthStr;

/// Trait implemented by any item that can be displayed in the results or remote-control list.
pub trait ResultItem {
    /// Returns the raw string representation of the item.
    fn raw(&self) -> &str;

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

    /// Optional description shown next to the name (minimal-mode pickers).
    fn description(&self) -> Option<&str> {
        None
    }

    /// The styling of the item, as the style it takes from a character
    /// offset within `display()` until the next run.
    fn styles(&self) -> Option<&[(u32, Style)]> {
        None
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

    match item.styles() {
        Some(styles) if !styles.is_empty() => {
            spans.extend(build_entry_spans_styled(item, styles, match_fg));
        }
        _ => {
            spans.extend(build_entry_spans(
                item,
                item_max_width,
                result_fg,
                match_fg,
            ));
        }
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

/// Build the rows of a [`ResultItem`] that is drawn `height` rows tall.
///
/// A height of 1 is exactly [`build_result_line`]. Above that, the item's
/// text is split on `\n` and each line gets a row of its own, so the result
/// always has exactly `height` lines: an item with fewer lines is padded with
/// blank ones, and whatever lies beyond the last row is joined onto it.
///
/// The selection prefix is drawn on the first row only; the rows below it are
/// indented by the prefix's width so that the lines stay aligned.
#[allow(clippy::too_many_arguments)]
pub fn build_result_text<'a, T: ResultItem + ?Sized>(
    item: &'a T,
    selection_fg: Color,
    result_fg: Color,
    match_fg: Color,
    area_width: u16,
    prefix: Option<bool>,
    height: u16,
) -> Text<'a> {
    if height <= 1 {
        return Text::from(build_result_line(
            item,
            selection_fg,
            result_fg,
            match_fg,
            area_width,
            prefix,
        ));
    }
    let mut lines: Vec<Line<'a>> = split_entry_lines(item, height)
        .into_iter()
        .enumerate()
        .map(|(i, line)| {
            let line_prefix = if i == 0 {
                prefix
            } else {
                prefix.map(|_| false)
            };
            // `EntryLine` owns its text, so the spans are made `'static`
            // rather than borrowing from a temporary
            let built = build_result_line(
                &line,
                selection_fg,
                result_fg,
                match_fg,
                area_width,
                line_prefix,
            );
            Line::from(
                built
                    .spans
                    .into_iter()
                    .map(|span| {
                        Span::styled(span.content.into_owned(), span.style)
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    lines.resize(usize::from(height), Line::default());
    Text::from(lines)
}

/// One line of a multi-line [`ResultItem`], with its match ranges and style
/// runs rebased onto the line.
struct EntryLine {
    text: String,
    match_ranges: Option<Vec<(u32, u32)>>,
    styles: Option<Vec<(u32, Style)>>,
}

impl ResultItem for EntryLine {
    fn raw(&self) -> &str {
        &self.text
    }

    fn display(&self) -> &str {
        &self.text
    }

    fn output(&self) -> Result<String> {
        Ok(self.text.clone())
    }

    fn match_ranges(&self) -> Option<&[(u32, u32)]> {
        self.match_ranges.as_deref()
    }

    fn styles(&self) -> Option<&[(u32, Style)]> {
        self.styles.as_deref()
    }
}

/// Split an item's display text into at most `height` lines.
///
/// Match ranges and style runs are character offsets into the whole text, so
/// each line keeps the part of them that falls inside it, shifted to start at
/// the line's first character. A style run that began on an earlier line
/// carries over to the start of this one.
///
/// The last line keeps any remaining `\n`s; they are removed when it is made
/// printable, which joins the overflow onto it.
fn split_entry_lines<T: ResultItem + ?Sized>(
    item: &T,
    height: u16,
) -> Vec<EntryLine> {
    let display = item.display();
    let max_lines = usize::from(height.max(1));

    // (start, end) character offsets of each line, `end` excluding the `\n`
    let mut bounds: Vec<(u32, u32)> = Vec::with_capacity(max_lines);
    let mut start = 0u32;
    let mut count = 0u32;
    for c in display.chars() {
        if c == '\n' && bounds.len() + 1 < max_lines {
            bounds.push((start, count));
            start = count + 1;
        }
        count += 1;
    }
    bounds.push((start, count));

    let styles = item.styles().filter(|s| !s.is_empty());
    bounds
        .into_iter()
        .map(|(start, end)| {
            let text = display
                .chars()
                .skip(start as usize)
                .take((end - start) as usize)
                .collect();
            let match_ranges = item.match_ranges().map(|ranges| {
                ranges
                    .iter()
                    .filter(|&&(s, e)| s < end && e > start)
                    .map(|&(s, e)| (s.max(start) - start, e.min(end) - start))
                    .collect()
            });
            let styles = styles.map(|runs| {
                let mut line_runs: Vec<(u32, Style)> = Vec::new();
                // the run in effect where this line begins
                if let Some(&(_, style)) =
                    runs.iter().rev().find(|&&(at, _)| at <= start)
                {
                    line_runs.push((0, style));
                }
                line_runs.extend(
                    runs.iter()
                        .filter(|&&(at, _)| at > start && at < end)
                        .map(|&(at, style)| (at - start, style)),
                );
                // keep the item on the styled path even if no run reaches
                // this line, so it is not repainted in the plain colours
                if line_runs.is_empty() {
                    line_runs.push((0, Style::default()));
                }
                line_runs
            });
            EntryLine {
                text,
                match_ranges,
                styles,
            }
        })
        .collect()
}

fn build_entry_spans<T: ResultItem + ?Sized>(
    item: &'_ T,
    max_width: u16,
    result_fg: Color,
    match_fg: Color,
) -> Vec<Span<'_>> {
    let mut spans = Vec::with_capacity(16);

    let (mut entry_name, mut match_ranges) = make_result_item_printable(item);

    // Truncate if too long.
    if UnicodeWidthStr::width(entry_name.as_ref()) > max_width as usize {
        let (name, ranges) =
            truncate_highlighted_string(&entry_name, &match_ranges, max_width);
        entry_name = std::borrow::Cow::Owned(name);
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
            let text: String =
                chars.iter().skip(idx).take(start - idx).collect();
            if !text.is_empty() {
                spans.push(Span::styled(text, Style::default().fg(result_fg)));
            }
        }
        if start < end {
            let text: String =
                chars.iter().skip(start).take(end - start).collect();
            if !text.is_empty() {
                spans.push(Span::styled(text, Style::default().fg(match_fg)));
            }
        }
        idx = end;
    }
    if idx < chars.len() {
        let text: String = chars.iter().skip(idx).collect();
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
/// The item's style runs and its match ranges are two independent partitions
/// of the same text, so this walks the characters once, tracking the current
/// run and the current match range, and starts a new span wherever either
/// changes. Matched characters keep the style of the run they fall in, with
/// the foreground overridden to `match_fg`, so highlights never disrupt the
/// item's own styling beyond that.
///
/// # Parameters
/// - `item`: The result item to render.
/// - `styles`: The item's style runs (see [`ResultItem::styles`]).
/// - `match_fg`: The foreground color to use for highlighted (matched) text.
///
/// # Returns
/// A vector of [`Span`]s, each with appropriate styling and highlighting.
fn build_entry_spans_styled<'a, T: ResultItem + ?Sized>(
    item: &'a T,
    styles: &[(u32, Style)],
    match_fg: Color,
) -> Vec<Span<'a>> {
    let display = item.display();
    let match_ranges = item.match_ranges().unwrap_or(&[]);

    // hypothesis: ~ 2 to 3 highlighted clusters + in the worst case scenario
    // each cluster splits its containing run into 3 parts -> + 6 spans so we
    // should be fine pre-allocating `styles.len() + 8`
    let mut spans: Vec<Span> = Vec::with_capacity(styles.len() + 8);
    let mut segment = String::new();
    let mut segment_style: Option<Style> = None;
    let mut run = 0;
    let mut range = 0;

    for (pos, c) in display.chars().enumerate() {
        let pos = u32::try_from(pos).unwrap_or(u32::MAX);
        while run + 1 < styles.len() && styles[run + 1].0 <= pos {
            run += 1;
        }
        // the first run may start after the first character
        let base = if styles[run].0 > pos {
            Style::default()
        } else {
            styles[run].1
        };

        while range < match_ranges.len() && match_ranges[range].1 <= pos {
            range += 1;
        }
        let highlighted =
            range < match_ranges.len() && match_ranges[range].0 <= pos;
        let style = if highlighted { base.fg(match_fg) } else { base };

        if segment_style != Some(style) {
            push_segment(&mut spans, &mut segment, segment_style);
            segment_style = Some(style);
        }
        segment.push(c);
    }
    push_segment(&mut spans, &mut segment, segment_style);

    spans
}

/// Flush the characters accumulated so far as a span, leaving `segment` empty.
fn push_segment(
    spans: &mut Vec<Span<'_>>,
    segment: &mut String,
    style: Option<Style>,
) {
    if segment.is_empty() {
        return;
    }
    let printable = replace_non_printable_bulk(
        segment,
        &ReplaceNonPrintableConfig::default(),
    )
    .0
    .into_owned();
    spans.push(Span::styled(printable, style.unwrap_or_default()));
    segment.clear();
}

/// Build a `List` widget from a slice of [`ResultItem`]s.
///
/// An empty `highlight_symbol` means "color-only" selection: rows stay flush
/// and the selected row keeps its normal text colors, carried by the
/// background wash alone.
#[allow(clippy::too_many_arguments)]
pub fn build_results_list<'a, 'b, T, F>(
    block: Block<'b>,
    entries: &'a [T],
    relative_picker_state: &ListState,
    list_direction: ListDirection,
    colorscheme: &ResultsColorscheme,
    area_width: u16,
    highlight_symbol: &'a str,
    entry_height: u16,
    mut prefix_fn: F,
) -> List<'a>
where
    'b: 'a,
    T: ResultItem,
    F: FnMut(&T) -> Option<bool>,
{
    let color_only = highlight_symbol.is_empty();
    List::new(entries.iter().enumerate().map(|(i, e)| {
        let prefix = prefix_fn(e);
        let result_fg =
            if !color_only && relative_picker_state.selected() == Some(i) {
                colorscheme.result_selected_fg
            } else {
                colorscheme.result_fg
            };
        build_result_text(
            e,
            colorscheme.result_selected_fg,
            result_fg,
            colorscheme.match_foreground_color,
            area_width,
            prefix,
            entry_height,
        )
    }))
    .direction(list_direction)
    .highlight_style(
        Style::default().bg(colorscheme.result_selected_bg).bold(),
    )
    .highlight_symbol(highlight_symbol)
    .block(block)
}

/// Build a `List` for the minimal-mode pickers (remote control, actions):
/// entry name with match highlights, followed by a dimmed description
/// column and a dimmed shortcut key. Selection is color-only (background
/// wash, no symbol).
pub fn build_minimal_picker_list<'a, 'b, T>(
    block: Block<'b>,
    entries: &'a [T],
    colorscheme: &ResultsColorscheme,
    dimmed_fg: Color,
    area_width: u16,
    list_direction: ListDirection,
    show_descriptions: bool,
) -> List<'a>
where
    'b: 'a,
    T: ResultItem,
{
    let name_col = entries
        .iter()
        .map(|e| UnicodeWidthStr::width(e.display()))
        .max()
        .unwrap_or(0);
    let desc_col = if show_descriptions {
        entries
            .iter()
            .map(|e| e.description().map_or(0, UnicodeWidthStr::width))
            .max()
            .unwrap_or(0)
    } else {
        0
    };

    List::new(entries.iter().map(|e| {
        let mut spans = build_entry_spans(
            e,
            area_width,
            colorscheme.result_fg,
            colorscheme.match_foreground_color,
        );
        if desc_col > 0 {
            let padding = name_col
                .saturating_sub(UnicodeWidthStr::width(e.display()))
                + 2;
            spans.push(Span::styled(
                format!(
                    "{}{:<desc_col$}",
                    " ".repeat(padding),
                    e.description().unwrap_or_default(),
                ),
                Style::default().fg(dimmed_fg),
            ));
        }
        if let Some(key) = e.shortcut() {
            spans.push(Span::styled(
                format!("  {}", key),
                Style::default().fg(dimmed_fg),
            ));
        }
        Line::from(spans)
    }))
    .direction(list_direction)
    .highlight_style(
        Style::default().bg(colorscheme.result_selected_bg).bold(),
    )
    .highlight_symbol("")
    .block(block)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channels::entry::Entry;
    use ratatui::prelude::{Color, Span};
    use ratatui::style::Stylize;
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

    /// The style runs `utils::ansi` produces for
    /// `"\x1b[31mRed\x1b[0m and \x1b[32mGreen\x1b[0m"`.
    /// The text of each row, with the spans of a row joined.
    fn row_texts(text: &Text) -> Vec<String> {
        text.lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect())
            .collect()
    }

    #[test]
    fn test_build_result_text_height_one_is_one_line() {
        let entry = Entry::new("subject\ncrumb".to_string());
        let text = build_result_text(
            &entry,
            Color::Reset,
            Color::Reset,
            Color::Reset,
            200,
            None,
            1,
        );
        // the single-line path is unchanged: the newline is removed
        assert_eq!(row_texts(&text), vec!["subjectcrumb"]);
    }

    #[test]
    fn test_build_result_text_splits_lines_and_matches() {
        // matches on "sub" in the first line and "cr" in the second
        let entry = Entry::new("subject\ncrumb".to_string())
            .with_match_indices(&[0, 1, 2, 8, 9]);
        let text = build_result_text(
            &entry,
            Color::Reset,
            Color::Reset,
            Color::Yellow,
            200,
            None,
            2,
        );
        assert_eq!(row_texts(&text), vec!["subject", "crumb"]);
        assert_eq!(text.lines[0].spans[0], Span::raw("sub").fg(Color::Yellow));
        assert_eq!(text.lines[1].spans[0], Span::raw("cr").fg(Color::Yellow));
        assert_eq!(text.lines[1].spans[1], Span::raw("umb").fg(Color::Reset));
    }

    #[test]
    fn test_build_result_text_match_spanning_the_newline() {
        // one range covering "t\nc" is cut in two at the line break
        let entry = Entry::new("subject\ncrumb".to_string())
            .with_match_indices(&[6, 7, 8]);
        let text = build_result_text(
            &entry,
            Color::Reset,
            Color::Reset,
            Color::Yellow,
            200,
            None,
            2,
        );
        assert_eq!(
            text.lines[0].spans.last().unwrap(),
            &Span::raw("t").fg(Color::Yellow)
        );
        assert_eq!(text.lines[1].spans[0], Span::raw("c").fg(Color::Yellow));
    }

    #[test]
    fn test_build_result_text_pads_short_entries() {
        let entry = Entry::new("only one line".to_string());
        let text = build_result_text(
            &entry,
            Color::Reset,
            Color::Reset,
            Color::Reset,
            200,
            None,
            3,
        );
        assert_eq!(row_texts(&text), vec!["only one line", "", ""]);
    }

    #[test]
    fn test_build_result_text_joins_overflow_onto_last_row() {
        let entry = Entry::new("a\nb\nc\nd".to_string());
        let text = build_result_text(
            &entry,
            Color::Reset,
            Color::Reset,
            Color::Reset,
            200,
            None,
            2,
        );
        assert_eq!(row_texts(&text), vec!["a", "bcd"]);
    }

    #[test]
    fn test_build_result_text_carries_style_across_lines() {
        // red from the start of "Red\nand", then green from "Green"
        let styles = vec![
            (0, Style::default().fg(Color::Red)),
            (8, Style::default().fg(Color::Green)),
        ];
        let entry = Entry::new("Red\nand Green".to_string())
            .with_match_indices(&[4])
            .with_styles(styles);
        let text = build_result_text(
            &entry,
            Color::Reset,
            Color::Reset,
            Color::Yellow,
            200,
            None,
            2,
        );
        assert_eq!(text.lines[0].spans, vec![Span::raw("Red").fg(Color::Red)]);
        assert_eq!(
            text.lines[1].spans,
            vec![
                Span::raw("a").fg(Color::Yellow),
                Span::raw("nd ").fg(Color::Red),
                Span::raw("Green").fg(Color::Green),
            ]
        );
    }

    #[test]
    fn test_build_result_text_styled_line_without_runs() {
        // the only run starts on the second line; the first stays unstyled
        // rather than falling back to the plain colours
        let styles = vec![(4, Style::default().fg(Color::Green))];
        let entry = Entry::new("abc\ndef".to_string()).with_styles(styles);
        let text = build_result_text(
            &entry,
            Color::Reset,
            Color::Blue,
            Color::Yellow,
            200,
            None,
            2,
        );
        assert_eq!(text.lines[0].spans, vec![Span::raw("abc")]);
        assert_eq!(
            text.lines[1].spans,
            vec![Span::raw("def").fg(Color::Green)]
        );
    }

    #[test]
    fn test_build_result_text_prefix_only_on_first_row() {
        let entry = Entry::new("subject\ncrumb".to_string());
        let text = build_result_text(
            &entry,
            Color::Reset,
            Color::Reset,
            Color::Reset,
            200,
            Some(true),
            2,
        );
        assert_eq!(
            row_texts(&text),
            vec![
                format!("{SELECTED_SYMBOL}subject"),
                format!("{DESELECTED_SYMBOL}crumb"),
            ]
        );
    }

    fn red_and_green() -> Vec<(u32, Style)> {
        vec![
            (0, Style::default().fg(Color::Red)),
            (3, Style::default()),
            (8, Style::default().fg(Color::Green)),
        ]
    }

    #[test]
    fn test_build_entry_spans_styled_overlays_matches() {
        let styles = red_and_green();
        let entry = Entry::new("Red and Green".to_string())
            .with_match_indices(&[1, 4, 5])
            .with_styles(styles.clone());
        let spans = build_entry_spans_styled(&entry, &styles, Color::Yellow);

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
        assert_eq!(spans[3], Span::raw(" "));
        assert_eq!(spans[4], Span::raw("an").fg(Color::Yellow));
        assert_eq!(spans[5], Span::raw("d "));
        assert_eq!(spans[6], Span::raw("Green").fg(Color::Green));
    }

    #[test]
    fn test_build_entry_spans_styled_replaces_non_printable() {
        let styles = vec![
            (0, Style::default().fg(Color::Red)),
            (3, Style::default()),
            (4, Style::default().fg(Color::Green)),
        ];
        let entry = Entry::new("Red\tGreen".to_string())
            .with_match_indices(&[1, 4, 5])
            .with_styles(styles.clone());
        let spans = build_entry_spans_styled(&entry, &styles, Color::Yellow);

        assert_eq!(
            spans.len(),
            6,
            "Expected 6 spans but got {:?}",
            spans
                .iter()
                .map(|s| (s.content.clone(), s.style.fg))
                .collect::<Vec<_>>()
        );
        assert_eq!(spans[0], Span::raw("R").fg(Color::Red));
        assert_eq!(spans[1], Span::raw("e").fg(Color::Yellow));
        assert_eq!(spans[2], Span::raw("d").fg(Color::Red));
        // tabs are expanded like everywhere else in the results list
        assert_eq!(spans[3], Span::raw("    "));
        assert_eq!(spans[4], Span::raw("Gr").fg(Color::Yellow));
        assert_eq!(spans[5], Span::raw("een").fg(Color::Green));
    }

    #[test]
    fn test_build_entry_spans_styled_corner_cases() {
        // a run covering everything, fully highlighted
        let styles = vec![(0, Style::default().fg(Color::Green))];
        let entry = Entry::new("A".to_string())
            .with_match_indices(&[0])
            .with_styles(styles.clone());
        let spans = build_entry_spans_styled(&entry, &styles, Color::Red);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0], Span::raw("A").fg(Color::Red));

        // no text at all
        let entry = Entry::new(String::new()).with_match_indices(&[]);
        assert!(
            build_entry_spans_styled(&entry, &styles, Color::Red).is_empty()
        );

        // a run starting after the first character leaves it unstyled
        let styles = vec![(1, Style::default().fg(Color::Green))];
        let entry = Entry::new("ab".to_string()).with_match_indices(&[]);
        let spans = build_entry_spans_styled(&entry, &styles, Color::Red);
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0], Span::raw("a"));
        assert_eq!(spans[1], Span::raw("b").fg(Color::Green));
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
