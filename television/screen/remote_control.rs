use crate::channels::entry::Entry;
use crate::screen::colors::{Colorscheme, GeneralColorscheme};
use crate::screen::logo::build_remote_logo_paragraph;
use crate::screen::mode::mode_color;
use crate::screen::results::build_results_list;
use crate::television::Mode;
use crate::utils::input::Input;

use anyhow::Result;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::Style;
use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, ListDirection, ListState, Padding, Paragraph,
};

#[allow(clippy::too_many_arguments)]
pub fn draw_remote_control(
    f: &mut Frame,
    rect: Rect,
    entries: &[Entry],
    use_nerd_font_icons: bool,
    picker_state: &mut ListState,
    input_state: &mut Input,
    mode: &Mode,
    colorscheme: &Colorscheme,
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
        colorscheme,
    );
    draw_rc_input(f, layout[1], input_state, colorscheme)?;
    draw_rc_logo(
        f,
        layout[2],
        mode_color(*mode, &colorscheme.mode),
        &colorscheme.general,
    );
    Ok(())
}

fn draw_rc_channels(
    f: &mut Frame,
    area: Rect,
    entries: &[Entry],
    use_nerd_font_icons: bool,
    picker_state: &mut ListState,
    colorscheme: &Colorscheme,
) {
    let rc_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colorscheme.general.border_fg))
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        )
        .padding(Padding::right(1));

    let channel_list = build_results_list(
        rc_block,
        entries,
        None,
        ListDirection::TopToBottom,
        use_nerd_font_icons,
        &colorscheme.results,
        area.width,
    );

    f.render_stateful_widget(channel_list, area, picker_state);
}

fn draw_rc_input(
    f: &mut Frame,
    area: Rect,
    input: &mut Input,
    colorscheme: &Colorscheme,
) -> Result<()> {
    let input_block = Block::default()
        .title_top(Line::from("Remote Control").alignment(Alignment::Center))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colorscheme.general.border_fg))
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        );

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
        Style::default().fg(colorscheme.input.input_fg).bold(),
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
        .style(
            Style::default()
                .fg(colorscheme.input.input_fg)
                .bold()
                .italic(),
        )
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
fn draw_rc_logo(
    f: &mut Frame,
    area: Rect,
    mode_color: Color,
    colorscheme: &GeneralColorscheme,
) {
    let logo_block = Block::default().style(
        Style::default()
            .fg(mode_color)
            .bg(colorscheme.background.unwrap_or_default()),
    );

    let logo_paragraph = build_remote_logo_paragraph()
        .alignment(Alignment::Center)
        .block(logo_block);

    f.render_widget(logo_paragraph, area);
}
