use crate::config::Binding;
use crate::screen::colors::ResultsColorscheme;
use crate::screen::constants::POINTER_SYMBOL;
use devicons::FileIcon;
use ratatui::prelude::{Color, Line, Span, Style};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, List, ListDirection};
use std::str::FromStr;
use unicode_width::UnicodeWidthStr;

/// Trait implemented by any item that can be displayed in the results or remote-control list.
pub trait ResultItem {
    /// Returns an optional icon to display in front of the item.
    fn icon(&self) -> Option<&FileIcon> {
        None
    }

    /// The main text representing the item.
    fn display(&self) -> &str;

    /// Highlight match ranges (char based indices) within `display()`.
    fn match_ranges(&self) -> Option<&[(u32, u32)]> {
        None
    }

    /// Optional shortcut binding shown after the name (remote-control entries).
    fn shortcut(&self) -> Option<&Binding> {
        None
    }

    /// Optional line number (file search results).
    fn line_number(&self) -> Option<u32> {
        None
    }
}

/// Build a single `Line` for a [`ResultItem`].
#[allow(clippy::too_many_arguments)]
#[allow(clippy::cast_possible_truncation)]
pub fn build_result_line<'a, T: ResultItem + ?Sized>(
    item: &'a T,
    use_icons: bool,
    colorscheme: &ResultsColorscheme,
    area_width: u16,
    prefix: Option<bool>, // Some(true)=selected ●, Some(false)=unselected, None=no prefix
) -> Line<'a> {
    let mut spans = Vec::<Span<'a>>::new();

    // Optional selection prefix
    if let Some(selected) = prefix {
        if selected {
            spans.push(Span::styled(
                crate::screen::constants::SELECTED_SYMBOL,
                Style::default().fg(colorscheme.result_selected_fg),
            ));
        } else {
            spans.push(Span::raw(crate::screen::constants::DESELECTED_SYMBOL));
        }
    }

    let selection_prefix_width: u16 = if prefix.is_some() { 2 } else { 0 };

    let shortcut_extra: u16 = item
        .shortcut()
        .map(|b| match b {
            Binding::SingleKey(k) => 2 + k.to_string().len() as u16, // space + key
            Binding::MultipleKeys(keys) => keys
                .iter()
                .map(|k| 1 + k.to_string().len() as u16) // space + key
                .sum(),
        })
        .unwrap_or(0);

    let name_max_width = area_width
        .saturating_sub(2) // pointer + space (kept for caller)
        .saturating_sub(2) // borders
        .saturating_sub(2 * u16::from(use_icons))
        .saturating_sub(selection_prefix_width)
        .saturating_sub(shortcut_extra);

    // Icon.
    if use_icons {
        if let Some(icon) = item.icon() {
            spans.push(Span::styled(
                icon.to_string(),
                Style::default().fg(Color::from_str(icon.color).unwrap()),
            ));
            spans.push(Span::raw(" "));
        }
    }

    let (mut entry_name, mut match_ranges) =
        crate::utils::strings::make_matched_string_printable(
            item.display(),
            item.match_ranges(),
        );

    // Truncate if too long.
    if UnicodeWidthStr::width(entry_name.as_str()) > name_max_width as usize {
        let (name, ranges) =
            crate::utils::indices::truncate_highlighted_string(
                &entry_name,
                &match_ranges,
                name_max_width,
            );
        entry_name = name;
        match_ranges = ranges;
    }

    // Build highlighted spans.
    let mut last_end = 0;
    let chars = entry_name.chars();
    let name_len = UnicodeWidthStr::width(entry_name.as_str());

    for (s, e) in match_ranges.iter().map(|(s, e)| (*s as usize, *e as usize))
    {
        spans.push(Span::styled(
            chars
                .clone()
                .skip(last_end)
                .take(s - last_end)
                .collect::<String>(),
            Style::default().fg(colorscheme.result_name_fg),
        ));
        spans.push(Span::styled(
            chars.clone().skip(s).take(e - s).collect::<String>(),
            Style::default().fg(colorscheme.match_foreground_color),
        ));
        last_end = e;
    }
    if last_end < name_len {
        spans.push(Span::styled(
            chars.skip(last_end).collect::<String>(),
            Style::default().fg(colorscheme.result_name_fg),
        ));
    }

    // Show shortcut if present.
    if let Some(binding) = item.shortcut() {
        spans.push(Span::raw(" "));
        match binding {
            Binding::SingleKey(k) => spans.push(Span::styled(
                k.to_string(),
                Style::default().fg(colorscheme.match_foreground_color),
            )),
            Binding::MultipleKeys(keys) => {
                for (i, k) in keys.iter().enumerate() {
                    if i > 0 {
                        spans.push(Span::raw(" "));
                    }
                    spans.push(Span::styled(
                        k.to_string(),
                        Style::default()
                            .fg(colorscheme.match_foreground_color),
                    ));
                }
            }
        }
    }

    // Optional line number (only for Entry).
    if let Some(line) = item.line_number() {
        spans.push(Span::styled(
            format!(":{line}"),
            Style::default().fg(colorscheme.result_line_number_fg),
        ));
    }

    Line::from(spans)
}

/// Build a `List` widget from a slice of [`ResultItem`]s.
#[allow(clippy::too_many_arguments)]
pub fn build_results_list<'a, 'b, T, F>(
    block: Block<'b>,
    entries: &'a [T],
    list_direction: ListDirection,
    use_icons: bool,
    colorscheme: &ResultsColorscheme,
    area_width: u16,
    mut prefix_fn: F,
) -> List<'a>
where
    'b: 'a,
    T: ResultItem,
    F: FnMut(&T) -> Option<bool>,
{
    use ratatui::widgets::List;
    List::new(entries.iter().map(|e| {
        let prefix = prefix_fn(e);
        build_result_line(e, use_icons, colorscheme, area_width, prefix)
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
    use crate::screen::colors::ResultsColorscheme;
    use ratatui::prelude::{Color, Span};
    use ratatui::text::Line;

    #[test]
    fn test_build_result_line_simple() {
        let entry = Entry::new("something nice".to_string())
            .with_match_indices(&[1, 2, 10, 11]);
        let line = build_result_line(
            &entry,
            false,
            &ResultsColorscheme::default(),
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
            false,
            &ResultsColorscheme::default(),
            20, // small width
            None,
        );

        // We expect the resulting string to contain the ellipsis char
        let rendered: String =
            line.spans.iter().map(|s| s.content.clone()).collect();
        assert!(rendered.contains('…'));
    }
}
