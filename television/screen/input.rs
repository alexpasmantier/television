use crate::{
    config::ui::{BorderType, DEFAULT_PROMPT, Padding},
    screen::{colors::Colorscheme, layout::InputPosition},
    utils::input::Input,
};
use anyhow::Result;
use ratatui::{
    Frame,
    layout::{
        Alignment, Constraint, Direction, Layout as RatatuiLayout, Rect,
    },
    style::{Color, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, ListState, Padding as RatatuiPadding, Paragraph,
        TitlePosition,
    },
};

const LOADING_CHAR: &str = "●";

#[allow(clippy::too_many_arguments)]
pub fn draw_input_box(
    f: &mut Frame,
    rect: Rect,
    results_count: u32,
    total_count: u32,
    input_state: &Input,
    results_picker_state: &ListState,
    matcher_running: bool,
    channel_name: &str,
    colorscheme: &Colorscheme,
    position: InputPosition,
    header: &Option<String>,
    padding: &Padding,
    border_type: &BorderType,
    prompt: Option<&String>,
    // minimal UI: dimmed compact count and undecorated query
    minimal: bool,
    // optional `· <hint>` suffix after the count in the given color (the
    // current channel or picker mode), shown when the status bar is hidden
    hint: Option<(&str, Color)>,
) -> Result<()> {
    // an empty header means "no header at all"
    let header = header.as_ref().map_or(channel_name, |v| v);
    let mut input_block = Block::default()
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        )
        .padding(RatatuiPadding::from(*padding));
    if !header.is_empty() {
        input_block = input_block
            .title_position(match position {
                InputPosition::Top => TitlePosition::Top,
                InputPosition::Bottom => TitlePosition::Bottom,
            })
            .title(
                Line::from(format!(" {} ", header))
                    .style(
                        Style::default().fg(colorscheme.mode.channel).bold(),
                    )
                    .centered(),
            );
    }
    if let Some(b) = border_type.to_ratatui_border_type() {
        input_block = input_block
            .borders(Borders::ALL)
            .border_type(b)
            .border_style(Style::default().fg(colorscheme.general.border_fg));
    }

    let input_block_inner = input_block.inner(rect);
    if input_block_inner.area() == 0 {
        return Ok(());
    }

    f.render_widget(input_block, rect);

    // split input block into 4 parts: prompt symbol, input, result count, indicator
    let indicator_len = if matcher_running { 2 } else { 0 };
    // an empty prompt means "no prompt at all"
    let prompt_len = prompt
        .as_ref()
        .map(|p| {
            if p.is_empty() {
                0
            } else {
                u16::try_from(p.chars().count() + 1)
                    .expect("Prompt length should fit in u16")
            }
        })
        .unwrap_or(2);
    let hint_len = hint.map_or(0, |(hint, _)| {
        u16::try_from(hint.chars().count() + 3)
            .expect("Hint length should fit in u16")
    });
    let constraints = [
        // prompt symbol + space
        Constraint::Length(prompt_len),
        // input field
        Constraint::Fill(1),
        // result count (+ optional mode hint)
        Constraint::Length(
            3 * (u16::try_from(total_count.max(1).ilog10()).unwrap() + 1)
                + 3
                + hint_len,
        ),
        // loading symbol
        Constraint::Length(indicator_len),
    ];

    let inner_input_chunks = RatatuiLayout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(input_block_inner);

    if prompt_len > 0 {
        let arrow_block = Block::default();
        let arrow = Paragraph::new(Span::styled(
            format!("{} ", prompt.unwrap_or(&DEFAULT_PROMPT.to_string())),
            Style::default().fg(colorscheme.input.input_fg).bold(),
        ))
        .block(arrow_block);
        f.render_widget(arrow, inner_input_chunks[0]);
    }

    let interactive_input_block = Block::default();
    // keep 2 for borders and 1 for cursor
    let width = inner_input_chunks[1].width.max(3) - 3;
    let scroll = input_state.visual_scroll(width as usize);
    // in minimal mode the query is left undecorated (terminal default
    // foreground, i.e. white on most dark terminals)
    let input_style = if minimal {
        Style::default().bold()
    } else {
        Style::default()
            .fg(colorscheme.input.input_fg)
            .bold()
            .italic()
    };
    let input = Paragraph::new(input_state.value())
        .scroll((0, u16::try_from(scroll)?))
        .block(interactive_input_block)
        .style(input_style)
        .alignment(Alignment::Left);
    f.render_widget(input, inner_input_chunks[1]);

    if matcher_running {
        f.render_widget(
            Span::styled(LOADING_CHAR, Style::default().fg(Color::Green)),
            inner_input_chunks[3],
        );
    }

    let result_count_block = Block::default();
    let selected_position = if results_count == 0 {
        0
    } else {
        results_picker_state.selected().unwrap_or(0) + 1
    };
    // minimal UI: a compact, dimmed `matches/total` count, with a mode hint
    // next to it (the channel name or the active picker) when it has to
    // stand in for the status bar's mode info
    let count_line = if minimal {
        let mut spans = vec![Span::styled(
            format!(" {}/{}", results_count, total_count),
            Style::default()
                .fg(colorscheme.general.dimmed_text_fg)
                .italic(),
        )];
        if let Some((hint, hint_color)) = hint {
            spans.push(Span::styled(
                format!(" · {}", hint),
                Style::default().fg(hint_color).italic(),
            ));
        }
        spans.push(Span::from(" "));
        Line::from(spans)
    } else {
        Line::from(Span::styled(
            format!(" {} / {} ", selected_position, results_count),
            Style::default()
                .fg(colorscheme.input.results_count_fg)
                .italic(),
        ))
    };
    let result_count_paragraph = Paragraph::new(count_line)
        .block(result_count_block)
        .alignment(Alignment::Right);
    f.render_widget(result_count_paragraph, inner_input_chunks[2]);

    // Make the cursor visible and ask tui-rs to put it at the
    // specified coordinates after rendering
    f.set_cursor_position((
        // Put cursor past the end of the input text
        inner_input_chunks[1].x.saturating_add(u16::try_from(
            input_state.visual_cursor().max(scroll) - scroll,
        )?),
        // Move one line down, from the border to the input line
        inner_input_chunks[1].y,
    ));
    Ok(())
}
