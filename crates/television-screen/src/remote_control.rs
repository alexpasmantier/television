use std::collections::HashMap;

use crate::logo::build_remote_logo_paragraph;
use crate::mode::REMOTE_CONTROL_COLOR;
use crate::results::build_results_list;
use television_channels::entry::Entry;
use television_utils::input::Input;

use crate::colors::{ResultsColorscheme, BORDER_COLOR, DEFAULT_INPUT_FG};
use color_eyre::eyre::Result;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::Style;
use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, ListDirection, ListState, Padding, Paragraph,
};
use ratatui::Frame;

pub fn draw_remote_control(
    f: &mut Frame,
    rect: Rect,
    entries: &[Entry],
    use_nerd_font_icons: bool,
    picker_state: &mut ListState,
    input_state: &mut Input,
    icon_color_cache: &mut HashMap<String, Color>,
) -> Result<()> {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(3),
                Constraint::Length(3),
                Constraint::Length(20),
            ]
            .as_ref(),
        )
        .split(rect);
    draw_rc_channels(
        f,
        layout[0],
        entries,
        use_nerd_font_icons,
        picker_state,
        icon_color_cache,
    );
    draw_rc_input(f, layout[1], input_state)?;
    draw_rc_logo(f, layout[2]);
    Ok(())
}

fn draw_rc_channels(
    f: &mut Frame,
    area: Rect,
    entries: &[Entry],
    use_nerd_font_icons: bool,
    picker_state: &mut ListState,
    icon_color_cache: &mut HashMap<String, Color>,
    results_colorscheme: &ResultsColorscheme,
) {
    let rc_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER_COLOR))
        .style(Style::default())
        .padding(Padding::right(1));

    let channel_list = build_results_list(
        rc_block,
        entries,
        ListDirection::TopToBottom,
        use_nerd_font_icons,
        icon_color_cache,
        &results_colorscheme,
    );

    f.render_stateful_widget(channel_list, area, picker_state);
}

fn draw_rc_input(f: &mut Frame, area: Rect, input: &mut Input) -> Result<()> {
    let input_block = Block::default()
        .title_top(Line::from("Remote Control").alignment(Alignment::Center))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER_COLOR))
        .style(Style::default());

    let input_block_inner = input_block.inner(area);

    f.render_widget(input_block, area);

    // split input block into 2 parts: prompt symbol, input
    let inner_input_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            // prompt symbol
            Constraint::Length(2),
            // input field
            Constraint::Fill(1),
        ])
        .split(input_block_inner);

    let prompt_symbol_block = Block::default();
    let arrow = Paragraph::new(Span::styled(
        "> ",
        Style::default().fg(DEFAULT_INPUT_FG).bold(),
    ))
    .block(prompt_symbol_block);
    f.render_widget(arrow, inner_input_chunks[0]);

    let interactive_input_block = Block::default();
    // keep 2 for borders and 1 for cursor
    let width = inner_input_chunks[1].width.max(3) - 3;
    let scroll = input.visual_scroll(width as usize);
    let input_paragraph = Paragraph::new(input.value())
        .scroll((0, u16::try_from(scroll)?))
        .block(interactive_input_block)
        .style(Style::default().fg(DEFAULT_INPUT_FG).bold().italic())
        .alignment(Alignment::Left);
    f.render_widget(input_paragraph, inner_input_chunks[1]);

    // Make the cursor visible and ask tui-rs to put it at the
    // specified coordinates after rendering
    f.set_cursor_position((
        // Put cursor past the end of the input text
        inner_input_chunks[1].x
            + u16::try_from(input.visual_cursor().max(scroll) - scroll)?,
        // Move one line down, from the border to the input line
        inner_input_chunks[1].y,
    ));
    Ok(())
}
fn draw_rc_logo(f: &mut Frame, area: Rect) {
    let logo_block =
        Block::default().style(Style::default().fg(REMOTE_CONTROL_COLOR));

    let logo_paragraph = build_remote_logo_paragraph()
        .alignment(Alignment::Center)
        .block(logo_block);

    f.render_widget(logo_paragraph, area);
}
