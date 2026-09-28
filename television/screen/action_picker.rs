use crate::{
    channels::action_picker::ActionEntry,
    config::layers::MergedConfig,
    config::ui::BorderType,
    screen::{
        colors::Colorscheme,
        input::draw_input_box,
        layout::{InputPosition, preview_pane_block},
        results::{draw_picker_list, picker_list_padding},
    },
    utils::input::Input,
};
use anyhow::Result;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::ListState,
};

#[allow(clippy::too_many_arguments)]
pub fn draw_actions_pane(
    f: &mut Frame,
    rect: Rect,
    entries: &[ActionEntry],
    relative_picker_state: &mut ListState,
    picker_state: &ListState,
    input_state: &Input,
    results_count: u32,
    total_count: u32,
    config: &MergedConfig,
    colorscheme: &Colorscheme,
) -> Result<()> {
    // the action pane reuses the preview pane
    let pane_block = preview_pane_block(config, colorscheme);
    let inner = pane_block.inner(rect);
    f.render_widget(pane_block, rect);
    if inner.area() == 0 {
        return Ok(());
    }

    // same vertical arrangement as the main picker: input line (with its
    // padding acting as the gap) and the list below/above it
    let input_height =
        1 + config.input_bar_padding.top + config.input_bar_padding.bottom;
    let constraints = match config.input_bar_position {
        InputPosition::Top => {
            [Constraint::Length(input_height), Constraint::Fill(1)]
        }
        InputPosition::Bottom => {
            [Constraint::Fill(1), Constraint::Length(input_height)]
        }
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);
    let (input_rect, list_rect) = match config.input_bar_position {
        InputPosition::Top => (chunks[0], chunks[1]),
        InputPosition::Bottom => (chunks[1], chunks[0]),
    };

    draw_input_box(
        f,
        input_rect,
        config,
        colorscheme,
        input_state,
        picker_state,
        results_count,
        total_count,
        false,
        "actions",
        // with no status bar, the picker hint stands in for the mode
        config
            .status_bar_hidden
            .then_some(("actions", colorscheme.mode.action_picker)),
        None,
    )?;
    let list_padding = picker_list_padding(
        config.results_panel_padding,
        config.preview_panel_border_type != BorderType::None,
    );
    draw_picker_list(
        f,
        list_rect,
        entries,
        relative_picker_state,
        config.input_bar_position,
        colorscheme,
        &list_padding,
        &BorderType::None,
        None,
        true,
    )?;

    Ok(())
}
