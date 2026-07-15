use crate::{
    config::{layers::MergedConfig, ui::DEFAULT_PROMPT},
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

/// Columns always reserved for the query, however long the count line gets.
const MIN_QUERY_WIDTH: u16 = 12;

/// Multi-source indicator rendered next to the result count in minimal
/// mode: the current source name and one dot per source.
pub struct SourceIndicator<'a> {
    pub name: Option<&'a str>,
    pub index: usize,
    pub count: usize,
}

#[allow(clippy::too_many_arguments)]
pub fn draw_input_box(
    f: &mut Frame,
    rect: Rect,
    config: &MergedConfig,
    colorscheme: &Colorscheme,
    input_state: &Input,
    results_picker_state: &ListState,
    results_count: u32,
    total_count: u32,
    matcher_running: bool,
    // shown as the header when the config doesn't set one
    header_fallback: &str,
    // optional `· <hint>` suffix after the count in the given color (the
    // current channel or picker mode), shown when the status bar is hidden
    hint: Option<(&str, Color)>,
    // multi-source channels: current source name + dots after the count
    sources: Option<&SourceIndicator>,
) -> Result<()> {
    let position = config.input_bar_position;
    let padding = &config.input_bar_padding;
    let border_type = &config.input_bar_border_type;
    let prompt = config.input_bar_prompt.as_ref();
    // minimal UI: dimmed compact count and undecorated query
    let minimal = config.input_bar_minimal;
    // an empty header means "no header at all"
    let header = config
        .input_bar_header
        .as_deref()
        .unwrap_or(header_fallback);
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
    // columns left for the count once the query field got its minimum
    let count_width_budget = input_block_inner
        .width
        .saturating_sub(prompt_len + indicator_len + MIN_QUERY_WIDTH)
        as usize;
    // minimal UI: a compact, dimmed `matches/total` count, with an optional
    // multi-source indicator and a mode hint (the channel name or the
    // active picker) when it has to stand in for the status bar's mode info
    let count_line = if minimal {
        let dimmed = Style::default()
            .fg(colorscheme.general.dimmed_text_fg)
            .italic();
        let build = |with_sources: bool, with_hint: bool| {
            let mut spans = vec![Span::styled(
                format!(" {}/{}", results_count, total_count),
                dimmed,
            )];
            if with_sources
                && let Some(sources) = sources
                && sources.count > 1
            {
                // dots first, then the name: the name buffers the dots from
                // the loading indicator at the end of the row
                let source_style =
                    Style::default().fg(colorscheme.input.source_indicator_fg);
                spans.push(Span::styled(" · ", dimmed));
                for i in 0..sources.count {
                    spans.push(Span::styled(
                        if i == sources.index { "● " } else { "○ " },
                        source_style,
                    ));
                }
                if let Some(name) = sources.name {
                    spans.push(Span::styled(name.to_string(), source_style));
                }
            }
            if with_hint && let Some((hint, hint_color)) = hint {
                spans.push(Span::styled(
                    format!(" · {}", hint),
                    Style::default().fg(hint_color).italic(),
                ));
            }
            spans.push(Span::from(" "));
            Line::from(spans)
        };
        // when the row gets narrow, drop whole segments instead of clipping
        // mid-word: the hint first, then the source indicator, then the
        // count itself
        [(true, true), (true, false), (false, false)]
            .iter()
            .map(|&(with_sources, with_hint)| build(with_sources, with_hint))
            .find(|line| line.width() <= count_width_budget)
            .unwrap_or_default()
    } else {
        let selected_position = if results_count == 0 {
            0
        } else {
            results_picker_state.selected().unwrap_or(0) + 1
        };
        let line = Line::from(Span::styled(
            format!(" {} / {} ", selected_position, results_count),
            Style::default()
                .fg(colorscheme.input.results_count_fg)
                .italic(),
        ));
        if line.width() <= count_width_budget {
            line
        } else {
            Line::default()
        }
    };
    let constraints = [
        // prompt symbol + space
        Constraint::Length(prompt_len),
        // input field
        Constraint::Fill(1),
        // result count (+ optional indicators), already sized to fit the
        // budget so it never starves the query field
        Constraint::Length(
            u16::try_from(count_line.width())
                .expect("Count line width should fit in u16"),
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
