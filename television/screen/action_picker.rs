use crate::{
    channels::action_picker::ActionEntry,
    screen::{colors::Colorscheme, result_item},
    utils::input::Input,
};
use anyhow::Result;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::{Line, Span, Style},
    style::Stylize,
    widgets::{
        Block, BorderType, Borders, Clear, ListDirection, ListState, Padding,
        Paragraph, Wrap,
    },
};

/// Minimum width required to show the description panel alongside the action list.
const MIN_WIDTH_FOR_DESCRIPTION_PANEL: u16 = 60;

#[allow(clippy::too_many_arguments)]
pub fn draw_action_picker(
    f: &mut Frame,
    rect: Rect,
    entries: &[ActionEntry],
    picker_state: &mut ListState,
    input_state: &mut Input,
    colorscheme: &Colorscheme,
) -> Result<()> {
    let mut constraints = vec![Constraint::Fill(1)];
    if rect.width > MIN_WIDTH_FOR_DESCRIPTION_PANEL {
        constraints.push(Constraint::Fill(1));
    }

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(rect);

    f.render_widget(Clear, rect);

    let selected_entry = entries.get(picker_state.selected().unwrap_or(0));

    draw_search_panel(
        f,
        layout[0],
        entries,
        picker_state,
        colorscheme,
        input_state,
    )?;

    if layout.len() > 1 {
        draw_detail_panel(f, layout[1], selected_entry, colorscheme);
    }

    Ok(())
}

fn draw_detail_panel(
    f: &mut Frame,
    rect: Rect,
    selected_entry: Option<&ActionEntry>,
    colorscheme: &Colorscheme,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colorscheme.general.border_fg))
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        )
        .padding(Padding::horizontal(1));

    let content: Vec<Line> = if let Some(entry) = selected_entry {
        let mut lines = Vec::new();

        // Description section
        let description = entry
            .description
            .as_deref()
            .unwrap_or("No description available.");
        lines.push(Line::from(Span::styled(
            "Description:",
            Style::default().bold(),
        )));
        lines.push(Line::from(Span::styled(
            description,
            Style::default().italic(),
        )));

        // Blank line separator
        lines.push(Line::from(""));

        // Command section
        lines.push(Line::from(Span::styled(
            "Command:",
            Style::default().bold(),
        )));
        if entry.commands.is_empty() {
            lines.push(Line::from("No command defined."));
        } else {
            for cmd in &entry.commands {
                lines.push(Line::from(Span::styled(
                    cmd.as_str(),
                    Style::default().fg(colorscheme.preview.title_fg),
                )));
            }
        }

        lines
    } else {
        Vec::new()
    };

    let paragraph = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, rect);
}

fn draw_search_panel(
    f: &mut Frame,
    area: Rect,
    entries: &[ActionEntry],
    picker_state: &mut ListState,
    colorscheme: &Colorscheme,
    input: &mut Input,
) -> Result<()> {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(3)].as_ref())
        .split(area);

    draw_action_list(f, layout[0], entries, picker_state, colorscheme);
    draw_input(f, layout[1], input, colorscheme)
}

fn draw_action_list(
    f: &mut Frame,
    area: Rect,
    entries: &[ActionEntry],
    picker_state: &mut ListState,
    colorscheme: &Colorscheme,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colorscheme.general.border_fg))
        .title_top(
            Line::from(" Actions ")
                .alignment(Alignment::Center)
                .italic(),
        )
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        )
        .padding(Padding::right(1));

    let action_list = result_item::build_results_list(
        block,
        entries,
        picker_state,
        ListDirection::TopToBottom,
        &colorscheme.results,
        area.width,
        |_| None,
    );

    f.render_stateful_widget(action_list, area, picker_state);
}

fn draw_input(
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

    let inner_input_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(2), Constraint::Fill(1)])
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

    f.set_cursor_position((
        inner_input_chunks[1].x
            + u16::try_from(input.visual_cursor().max(scroll) - scroll)?,
        inner_input_chunks[1].y,
    ));
    Ok(())
}
