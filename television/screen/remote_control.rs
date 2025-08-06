use crate::{
    channels::{prototypes::BinaryRequirement, remote_control::CableEntry},
    screen::{
        colors::{Colorscheme, GeneralColorscheme},
        logo::{REMOTE_LOGO_WIDTH_U16, build_remote_logo_paragraph},
        result_item,
    },
    utils::input::Input,
};
use anyhow::Result;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::{Color, Line, Span, Style},
    style::Stylize,
    widgets::{
        Block, BorderType, Borders, Clear, ListDirection, ListState, Padding,
        Paragraph, Wrap,
    },
};

#[allow(clippy::too_many_arguments)]
pub fn draw_remote_control(
    f: &mut Frame,
    rect: Rect,
    entries: &[CableEntry],
    picker_state: &mut ListState,
    input_state: &mut Input,
    colorscheme: &Colorscheme,
    show_channel_descriptions: bool,
) -> Result<()> {
    let layout = if show_channel_descriptions {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Fill(1),
                    Constraint::Fill(1),
                    Constraint::Length(REMOTE_LOGO_WIDTH_U16 + 2),
                ]
                .as_ref(),
            )
            .split(rect)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Fill(1),
                    Constraint::Length(REMOTE_LOGO_WIDTH_U16 + 2),
                ]
                .as_ref(),
            )
            .split(rect)
    };

    // Clear the popup area
    f.render_widget(Clear, rect);

    let selected_entry = entries.get(picker_state.selected().unwrap_or(0));

    draw_rc_logo(f, layout[layout.len() - 1], &colorscheme.general);
    draw_search_panel(
        f,
        layout[0],
        entries,
        picker_state,
        colorscheme,
        input_state,
    )?;

    if show_channel_descriptions {
        draw_information_panel(f, layout[1], selected_entry, colorscheme);
    }

    Ok(())
}

fn draw_information_panel(
    f: &mut Frame,
    rect: Rect,
    selected_entry: Option<&CableEntry>,
    colorscheme: &Colorscheme,
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(3)].as_ref())
        .split(rect);

    draw_description_block(f, layout[0], selected_entry, colorscheme);
    draw_requirements_block(f, layout[1], selected_entry, colorscheme);
}

fn draw_description_block(
    f: &mut Frame,
    area: Rect,
    selected_entry: Option<&CableEntry>,
    colorscheme: &Colorscheme,
) {
    let description_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colorscheme.general.border_fg))
        .title_top(Line::from(" Description ").alignment(Alignment::Center))
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        )
        .padding(Padding::right(1));

    let description = if let Some(entry) = selected_entry {
        entry
            .description
            .clone()
            .unwrap_or_else(|| "No description available.".to_string())
    } else {
        String::new()
    };

    let description_paragraph = Paragraph::new(description)
        .block(description_block)
        .style(Style::default().italic())
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(description_paragraph, area);
}

fn draw_requirements_block(
    f: &mut Frame,
    area: Rect,
    selected_entry: Option<&CableEntry>,
    colorscheme: &Colorscheme,
) {
    let mut requirements_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colorscheme.general.border_fg))
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        )
        .padding(Padding::right(1));

    if selected_entry.is_none() {
        // If no entry is selected, just render an empty block
        let title = Line::from(" Requirements ")
            .alignment(Alignment::Center)
            .italic();
        f.render_widget(requirements_block.title_top(title), area);
        return;
    }
    let selected_entry = selected_entry.unwrap();
    let spans = selected_entry.requirements.iter().fold(
        Vec::new(),
        |mut acc, requirement| {
            acc.push(Span::styled(
                format!("{} ", &requirement.bin_name),
                Style::default()
                    .fg(if requirement.is_met() {
                        Color::Green
                    } else {
                        Color::Red
                    })
                    .bold()
                    .italic(),
            ));
            acc
        },
    );

    requirements_block = requirements_block.title_top(
        Line::from({
            let mut title = vec![Span::from(" Requirements ")];
            if spans.is_empty()
                || selected_entry
                    .requirements
                    .iter()
                    .all(BinaryRequirement::is_met)
            {
                title.push(Span::styled(
                    "[OK] ",
                    Style::default().fg(Color::Green),
                ));
            } else {
                title.push(Span::styled(
                    "[MISSING] ",
                    Style::default().fg(Color::Red),
                ));
            }
            title
        })
        .style(Style::default().italic())
        .alignment(Alignment::Center),
    );

    let requirements_paragraph = Paragraph::new(Line::from(spans))
        .block(requirements_block)
        .alignment(Alignment::Center);

    f.render_widget(requirements_paragraph, area);
}

fn draw_search_panel(
    f: &mut Frame,
    area: Rect,
    entries: &[CableEntry],
    picker_state: &mut ListState,
    colorscheme: &Colorscheme,
    input: &mut Input,
) -> Result<()> {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(3)].as_ref())
        .split(area);

    draw_rc_channels(f, layout[0], entries, picker_state, colorscheme);
    draw_rc_input(f, layout[1], input, colorscheme)
}

fn draw_rc_channels(
    f: &mut Frame,
    area: Rect,
    entries: &[CableEntry],
    picker_state: &mut ListState,
    colorscheme: &Colorscheme,
) {
    let rc_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colorscheme.general.border_fg))
        .title_top(
            Line::from(" Channels ")
                .alignment(Alignment::Center)
                .italic(),
        )
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        )
        .padding(Padding::right(1));

    let channel_list = result_item::build_results_list(
        rc_block,
        entries,
        picker_state,
        ListDirection::TopToBottom,
        &colorscheme.results,
        area.width,
        |_| None,
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
        .title_top(
            Line::from(" Search ").alignment(Alignment::Center).italic(),
        )
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

fn draw_rc_logo(f: &mut Frame, area: Rect, colorscheme: &GeneralColorscheme) {
    let logo_block = Block::default()
        .style(Style::default().bg(colorscheme.background.unwrap_or_default()))
        .padding(Padding::horizontal(1));

    let logo_paragraph = build_remote_logo_paragraph()
        .alignment(Alignment::Center)
        .block(logo_block);

    f.render_widget(logo_paragraph, area);
}
