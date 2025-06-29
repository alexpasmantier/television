use crate::{
    channels::entry::Entry,
    screen::{colors::Colorscheme, layout::InputPosition, result_item},
};
use anyhow::Result;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    prelude::Style,
    text::Line,
    widgets::{Block, BorderType, Borders, ListState, Padding},
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
    use_nerd_font_icons: bool,
    colorscheme: &Colorscheme,
) -> Result<()> {
    let results_block = Block::default()
        .title_top(Line::from(" Results ").alignment(Alignment::Center))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colorscheme.general.border_fg))
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        )
        .padding(Padding::right(1));

    let list_direction = match input_bar_position {
        InputPosition::Bottom => ratatui::widgets::ListDirection::BottomToTop,
        InputPosition::Top => ratatui::widgets::ListDirection::TopToBottom,
    };

    let has_multi_select = !selected_entries.is_empty();

    let results_list = result_item::build_results_list(
        results_block,
        entries,
        list_direction,
        use_nerd_font_icons,
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
