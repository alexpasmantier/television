use color_eyre::Result;
use ratatui::{
    layout::{
        Alignment, Constraint, Direction, Layout as RatatuiLayout, Rect,
    },
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, ListState, Paragraph},
    Frame,
};
use television_utils::input::Input;

use crate::{
    colors::Colorscheme,
    spinner::{Spinner, SpinnerState},
};

// TODO: refactor arguments (e.g. use a struct for the spinner+state, same
#[allow(clippy::too_many_arguments)]
pub fn draw_input_box(
    f: &mut Frame,
    rect: Rect,
    results_count: u32,
    total_count: u32,
    input_state: &mut Input,
    results_picker_state: &mut ListState,
    matcher_running: bool,
    spinner: &Spinner,
    spinner_state: &mut SpinnerState,
    colorscheme: &Colorscheme,
) -> Result<()> {
    let input_block = Block::default()
        .title_top(Line::from(" Pattern ").alignment(Alignment::Center))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colorscheme.general.border_fg))
        .style(Style::default().bg(colorscheme.general.background));

    let input_block_inner = input_block.inner(rect);
    if input_block_inner.area() == 0 {
        return Ok(());
    }

    f.render_widget(input_block, rect);

    // split input block into 4 parts: prompt symbol, input, result count, spinner
    let inner_input_chunks = RatatuiLayout::default()
        .direction(Direction::Horizontal)
        .constraints([
            // prompt symbol
            Constraint::Length(2),
            // input field
            Constraint::Fill(1),
            // result count
            Constraint::Length(
                3 * ((total_count as f32).log10().ceil() as u16 + 1) + 3,
            ),
            // spinner
            Constraint::Length(1),
        ])
        .split(input_block_inner);

    let arrow_block = Block::default();
    let arrow = Paragraph::new(Span::styled(
        "> ",
        Style::default().fg(colorscheme.input.input_fg).bold(),
    ))
    .block(arrow_block);
    f.render_widget(arrow, inner_input_chunks[0]);

    let interactive_input_block = Block::default();
    // keep 2 for borders and 1 for cursor
    let width = inner_input_chunks[1].width.max(3) - 3;
    let scroll = input_state.visual_scroll(width as usize);
    let input = Paragraph::new(input_state.value())
        .scroll((0, u16::try_from(scroll)?))
        .block(interactive_input_block)
        .style(
            Style::default()
                .fg(colorscheme.input.input_fg)
                .bold()
                .italic(),
        )
        .alignment(Alignment::Left);
    f.render_widget(input, inner_input_chunks[1]);

    if matcher_running {
        f.render_stateful_widget(
            spinner,
            inner_input_chunks[3],
            spinner_state,
        );
    }

    let result_count_block = Block::default();
    let result_count_paragraph = Paragraph::new(Span::styled(
        format!(
            " {} / {} ",
            if results_count == 0 {
                0
            } else {
                results_picker_state.selected().unwrap_or(0) + 1
            },
            results_count,
        ),
        Style::default()
            .fg(colorscheme.input.results_count_fg)
            .italic(),
    ))
    .block(result_count_block)
    .alignment(Alignment::Right);
    f.render_widget(result_count_paragraph, inner_input_chunks[2]);

    // Make the cursor visible and ask tui-rs to put it at the
    // specified coordinates after rendering
    f.set_cursor_position((
        // Put cursor past the end of the input text
        inner_input_chunks[1].x
            + u16::try_from(input_state.visual_cursor().max(scroll) - scroll)?,
        // Move one line down, from the border to the input line
        inner_input_chunks[1].y,
    ));
    Ok(())
}
