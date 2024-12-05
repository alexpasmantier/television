use crate::television::Television;
use crate::ui::layout::InputPosition;
use crate::ui::BORDER_COLOR;
use color_eyre::eyre::Result;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Color, Line, Span, Style};
use ratatui::widgets::{
    Block, BorderType, Borders, List, ListDirection, Padding,
};
use ratatui::Frame;
use std::collections::HashMap;
use std::str::FromStr;
use television_channels::channels::OnAir;
use television_channels::entry::Entry;
use television_utils::strings::{
    make_matched_string_printable, next_char_boundary,
    slice_at_char_boundaries,
};

// Styles
const DEFAULT_RESULT_NAME_FG: Color = Color::Blue;
const DEFAULT_RESULT_PREVIEW_FG: Color = Color::Rgb(150, 150, 150);
const DEFAULT_RESULT_LINE_NUMBER_FG: Color = Color::Yellow;
const DEFAULT_RESULT_SELECTED_BG: Color = Color::Rgb(50, 50, 50);

const DEFAULT_RESULTS_LIST_MATCH_FOREGROUND_COLOR: Color = Color::Red;

pub struct ResultsListColors {
    pub result_name_fg: Color,
    pub result_preview_fg: Color,
    pub result_line_number_fg: Color,
    pub result_selected_bg: Color,
}

impl Default for ResultsListColors {
    fn default() -> Self {
        Self {
            result_name_fg: DEFAULT_RESULT_NAME_FG,
            result_preview_fg: DEFAULT_RESULT_PREVIEW_FG,
            result_line_number_fg: DEFAULT_RESULT_LINE_NUMBER_FG,
            result_selected_bg: DEFAULT_RESULT_SELECTED_BG,
        }
    }
}

#[allow(dead_code)]
impl ResultsListColors {
    pub fn result_name_fg(mut self, color: Color) -> Self {
        self.result_name_fg = color;
        self
    }

    pub fn result_preview_fg(mut self, color: Color) -> Self {
        self.result_preview_fg = color;
        self
    }

    pub fn result_line_number_fg(mut self, color: Color) -> Self {
        self.result_line_number_fg = color;
        self
    }

    pub fn result_selected_bg(mut self, color: Color) -> Self {
        self.result_selected_bg = color;
        self
    }
}

pub fn build_results_list<'a, 'b>(
    results_block: Block<'b>,
    entries: &'a [Entry],
    list_direction: ListDirection,
    results_list_colors: Option<ResultsListColors>,
    use_icons: bool,
    icon_color_cache: &mut HashMap<String, Color>,
) -> List<'a>
where
    'b: 'a,
{
    let results_list_colors = results_list_colors.unwrap_or_default();
    List::new(entries.iter().map(|entry| {
        let mut spans = Vec::new();
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
                Style::default().fg(results_list_colors.result_name_fg),
            ));
            // the current match
            spans.push(Span::styled(
                slice_at_char_boundaries(&entry_name, start, end).to_string(),
                Style::default()
                    .fg(DEFAULT_RESULTS_LIST_MATCH_FOREGROUND_COLOR),
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
                Style::default().fg(results_list_colors.result_name_fg),
            ));
        }
        // optional line number
        if let Some(line_number) = entry.line_number {
            spans.push(Span::styled(
                format!(":{line_number}"),
                Style::default().fg(results_list_colors.result_line_number_fg),
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
                    Style::default().fg(results_list_colors.result_preview_fg),
                ));
                spans.push(Span::styled(
                    slice_at_char_boundaries(&preview, start, end).to_string(),
                    Style::default()
                        .fg(DEFAULT_RESULTS_LIST_MATCH_FOREGROUND_COLOR),
                ));
                last_match_end = end;
            }
            let next_boundary = next_char_boundary(&preview, last_match_end);
            if next_boundary < preview.len() {
                spans.push(Span::styled(
                    preview[next_boundary..].to_string(),
                    Style::default().fg(results_list_colors.result_preview_fg),
                ));
            }
        }
        Line::from(spans)
    }))
    .direction(list_direction)
    .highlight_style(
        Style::default().bg(results_list_colors.result_selected_bg),
    )
    .highlight_symbol("> ")
    .block(results_block)
}

impl Television {
    pub(crate) fn draw_results_list(
        &mut self,
        f: &mut Frame,
        rect: Rect,
    ) -> Result<()> {
        let results_block = Block::default()
            .title_top(Line::from(" Results ").alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BORDER_COLOR))
            .style(Style::default())
            .padding(Padding::right(1));

        let result_count = self.channel.result_count();
        if result_count > 0 && self.results_picker.selected().is_none() {
            self.results_picker.select(Some(0));
            self.results_picker.relative_select(Some(0));
        }

        let entries = self.channel.results(
            rect.height.saturating_sub(2).into(),
            u32::try_from(self.results_picker.offset())?,
        );

        let results_list = build_results_list(
            results_block,
            &entries,
            match self.config.ui.input_bar_position {
                InputPosition::Bottom => ListDirection::BottomToTop,
                InputPosition::Top => ListDirection::TopToBottom,
            },
            None,
            self.config.ui.use_nerd_font_icons,
            &mut self.icon_color_cache,
        );

        f.render_stateful_widget(
            results_list,
            rect,
            &mut self.results_picker.relative_state,
        );
        Ok(())
    }
}
