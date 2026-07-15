use crate::{
    channels::entry::Entry,
    config::ui::{BorderType, Padding},
    event::Key,
    screen::{
        colors::Colorscheme, constants::POINTER_SYMBOL, layout::InputPosition,
        result_item,
    },
};
use anyhow::Result;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    prelude::{Span, Style},
    text::Line,
    widgets::{Block, Borders, ListState, Padding as RatatuiPadding},
};
use rustc_hash::FxHashSet;

#[allow(clippy::too_many_arguments)]
pub fn draw_results_list(
    f: &mut Frame,
    rect: Rect,
    entries: &[Entry],
    selected_entries: &FxHashSet<Entry>,
    relative_picker_state: &mut ListState,
    input_bar_position: InputPosition,
    colorscheme: &Colorscheme,
    results_panel_padding: &Padding,
    results_panel_border_type: &BorderType,
    source_index: usize,
    source_count: usize,
    current_source_name: Option<&str>,
    cycle_key: Option<Key>,
) -> Result<()> {
    let borderless = *results_panel_border_type == BorderType::None;
    // Borderless results are rendered without any title line to keep the
    // display minimal (color-only).
    let title = if borderless {
        None
    } else if source_count > 1 {
        let mut spans = match current_source_name {
            Some(name) => {
                vec![Span::from(" "), Span::from(name), Span::from(" ")]
            }
            None => vec![Span::from(" Results ")],
        };
        let dots: String = (0..source_count)
            .map(|i| if i == source_index { "●" } else { "○" })
            .collect::<Vec<_>>()
            .join(" ");
        spans.push(Span::styled(
            format!("⟨ {} ⟩", dots),
            Style::default().fg(colorscheme.input.results_count_fg),
        ));
        if let Some(key) = cycle_key {
            spans.push(Span::styled(
                format!(" {}", key),
                Style::default().fg(colorscheme.general.border_fg),
            ));
        }
        spans.push(Span::from(" "));
        Some(Line::from(spans).alignment(Alignment::Center))
    } else {
        Some(Line::from(" Results ").alignment(Alignment::Center))
    };

    let mut results_block = Block::default()
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        )
        .padding(RatatuiPadding::from(*results_panel_padding));
    if let Some(title) = title {
        results_block = results_block.title_top(title);
    }
    if let Some(border_type) =
        results_panel_border_type.to_ratatui_border_type()
    {
        results_block = results_block
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(colorscheme.general.border_fg));
    }

    let list_direction = match input_bar_position {
        InputPosition::Bottom => ratatui::widgets::ListDirection::BottomToTop,
        InputPosition::Top => ratatui::widgets::ListDirection::TopToBottom,
    };

    let has_multi_select = !selected_entries.is_empty();

    let results_list = result_item::build_results_list(
        results_block,
        entries,
        relative_picker_state,
        list_direction,
        &colorscheme.results,
        rect.width - 1, // right padding
        if borderless { "" } else { POINTER_SYMBOL },
        |entry| {
            if has_multi_select {
                Some(selected_entries.contains(entry))
            } else {
                None
            }
        },
    );

    f.render_stateful_widget(results_list, rect, relative_picker_state);
    Ok(())
}

/// Draw a minimal-mode picker list (remote control / actions picker
/// takeover): borderless, color-only selection, dimmed description and
/// shortcut columns.
pub fn draw_minimal_picker_list<T: result_item::ResultItem>(
    f: &mut Frame,
    rect: Rect,
    entries: &[T],
    relative_picker_state: &mut ListState,
    input_bar_position: InputPosition,
    colorscheme: &Colorscheme,
    padding: &Padding,
) -> Result<()> {
    let block = Block::default()
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        )
        .padding(RatatuiPadding::from(*padding));

    let list_direction = match input_bar_position {
        InputPosition::Bottom => ratatui::widgets::ListDirection::BottomToTop,
        InputPosition::Top => ratatui::widgets::ListDirection::TopToBottom,
    };

    let list = result_item::build_minimal_picker_list(
        block,
        entries,
        &colorscheme.results,
        colorscheme.general.dimmed_text_fg,
        rect.width.saturating_sub(1),
        list_direction,
    );

    f.render_stateful_widget(list, rect, relative_picker_state);
    Ok(())
}
