use crate::{
    channels::entry::Entry,
    config::ui::{BorderType, Padding},
    screen::{colors::Colorscheme, layout::InputPosition, result_item},
};
use anyhow::Result;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    prelude::Style,
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
) -> Result<()> {
    let mut results_block = Block::default()
        .title_top(Line::from(" Results ").alignment(Alignment::Center))
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        )
        .padding(RatatuiPadding::from(*results_panel_padding));
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
