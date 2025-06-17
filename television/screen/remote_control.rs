use crate::channels::remote_control::{CABLE_ICON, CableEntry};
use crate::config::Binding;
use crate::screen::colors::{
    Colorscheme, GeneralColorscheme, ResultsColorscheme,
};
use crate::screen::logo::build_remote_logo_paragraph;
use crate::screen::mode::mode_color;
use crate::screen::results::POINTER_SYMBOL;
use crate::television::Mode;
use crate::utils::indices::truncate_highlighted_string;
use crate::utils::input::Input;
use std::str::FromStr;

use anyhow::Result;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Line, Span, Style};
use ratatui::style::Stylize;
use ratatui::widgets::{
    Block, BorderType, Borders, List, ListDirection, ListState, Padding,
    Paragraph,
};
use unicode_width::UnicodeWidthStr;

#[allow(clippy::too_many_arguments)]
pub fn draw_remote_control(
    f: &mut Frame,
    rect: Rect,
    entries: &[CableEntry],
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
    entries: &[CableEntry],
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
        ListDirection::TopToBottom,
        use_nerd_font_icons,
        &colorscheme.results,
        area.width,
    );

    f.render_stateful_widget(channel_list, area, picker_state);
}

pub fn build_results_list<'a, 'b>(
    results_block: Block<'b>,
    entries: &'a [CableEntry],
    list_direction: ListDirection,
    use_icons: bool,
    colorscheme: &ResultsColorscheme,
    area_width: u16,
) -> List<'a>
where
    'b: 'a,
{
    List::new(entries.iter().map(|entry| {
        build_result_line(entry, use_icons, colorscheme, area_width)
    }))
    .direction(list_direction)
    .highlight_style(
        Style::default().bg(colorscheme.result_selected_bg).bold(),
    )
    .highlight_symbol(POINTER_SYMBOL)
    .block(results_block)
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

fn max_width(
    area_width: u16,
    use_icons: bool,
    channel_shortcut: Option<&Binding>,
) -> u16 {
    area_width
        .saturating_sub(2) // 2 for borders
        .saturating_sub(2 * u16::from(use_icons)) // 2 for the icon and space
        .saturating_sub(2) // 2 for the pointer symbol and space
        .saturating_sub(1) // 1 for the padding
        // reserve space for the optional shortcut
        .saturating_sub(if let Some(shortcut) = channel_shortcut {
            match shortcut {
                // > entry_1 Ctrl+S
                Binding::SingleKey(key) => {
                    // 1 for the key and 1 for the space before it
                    2 + u16::try_from(key.to_string().len()).unwrap_or(0)
                }
                // > entry_2 Ctrl+S Alt+T
                Binding::MultipleKeys(keys) => {
                    // each key's length + 1 for the space before it
                    keys.iter()
                        .map(|key| {
                            1 + u16::try_from(key.to_string().len())
                                .unwrap_or(0)
                        })
                        .sum::<u16>()
                }
            }
        } else {
            0
        })
}

fn build_result_line<'a>(
    entry: &'a CableEntry,
    use_icons: bool,
    colorscheme: &ResultsColorscheme,
    area_width: u16,
) -> Line<'a> {
    let mut spans = Vec::new();
    let name_max_width =
        max_width(area_width, use_icons, entry.shortcut.as_ref());
    // optional icon
    if use_icons {
        spans.push(Span::styled(
            CABLE_ICON.icon.to_string(),
            Style::default().fg(Color::from_str(CABLE_ICON.color).unwrap()),
        ));

        spans.push(Span::raw(" "));
    }
    // if the name is too long, we need to truncate it and add an ellipsis
    let mut channel_name = entry.channel_name.clone();
    let mut match_ranges = entry.match_ranges.clone().unwrap_or_default();
    if channel_name.width() > name_max_width as usize {
        (channel_name, match_ranges) = truncate_highlighted_string(
            &channel_name,
            &match_ranges,
            name_max_width,
        );
    }

    // build the spans for the entry name and match ranges
    let mut last_match_end = 0;
    let name_chars = channel_name.chars();
    let name_len = channel_name.as_str().width();
    for (start, end) in
        match_ranges.iter().map(|(s, e)| (*s as usize, *e as usize))
    {
        // from the end of the last match to the start of the current one
        spans.push(Span::styled(
            name_chars
                .clone()
                .skip(last_match_end)
                .take(start - last_match_end)
                .collect::<String>(),
            //channel_name[last_match_end..start].to_string(),
            Style::default().fg(colorscheme.result_name_fg),
        ));
        // the current match
        spans.push(Span::styled(
            name_chars
                .clone()
                .skip(start)
                .take(end - start)
                .collect::<String>(),
            Style::default().fg(colorscheme.match_foreground_color),
        ));
        last_match_end = end;
    }
    // we need to push a span for the remainder of the entry name
    // but only if there's something left
    if last_match_end < name_len {
        let remainder = name_chars.skip(last_match_end).collect::<String>();
        spans.push(Span::styled(
            remainder,
            Style::default().fg(colorscheme.result_name_fg),
        ));
    }
    // if the entry has a shortcut, add it
    if let Some(shortcut) = &entry.shortcut {
        spans.push(Span::raw(" "));
        match shortcut {
            Binding::SingleKey(key) => {
                spans.push(Span::styled(
                    key.to_string(),
                    Style::default().fg(colorscheme.match_foreground_color),
                ));
            }
            Binding::MultipleKeys(keys) => {
                for (i, key) in keys.iter().enumerate() {
                    if i > 0 {
                        spans.push(Span::raw(" "));
                    }
                    spans.push(Span::styled(
                        key.to_string(),
                        Style::default()
                            .fg(colorscheme.match_foreground_color),
                    ));
                }
            }
        }
    }
    Line::from(spans)
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
