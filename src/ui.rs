use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders, List, ListDirection, Padding, Paragraph,
    },
    Frame,
};
use std::str::FromStr;
use syntect::highlighting::Color as SyntectColor;

use crate::app::{self, App};
use crate::syntax::convert_syn_region_to_span;

use devicons::{icon_for_file, File};

mod backend;
mod input;

pub use input::Input;

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

fn get_border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Rgb(90, 90, 110)).dim()
    }
}

// UI size
const UI_WIDTH_PERCENT: u16 = 100;
const UI_HEIGHT_PERCENT: u16 = 100;

// input colors
const DEFAULT_INPUT_FG: Color = Color::Rgb(200, 200, 200);
const DEFAULT_RESULTS_COUNT_FG: Color = Color::Rgb(170, 170, 170);

// results colors
const DEFAULT_RESULT_MATCH_COLOR: Color = Color::Rgb(255, 150, 150);
const DEFAULT_RESULT_LINE_FG: Color = Color::Rgb(150, 150, 150);

// preview colors
const DEFAULT_PREVIEW_GUTTER_FG: Color = Color::Rgb(70, 70, 70);
const DEFAULT_PREVIEW_GUTTER_SELECTED_FG: Color = Color::Rgb(255, 150, 150);

const FOUR_SPACES: &str = "    ";

pub fn ui(frame: &mut Frame, app: &mut App) {
    let main_block = centered_rect(UI_WIDTH_PERCENT, UI_HEIGHT_PERCENT, frame.area());

    // split the main block into two vertical chunks
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_block);

    // left block: results + input field
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10)])
        .split(chunks[1]);

    // top left block: results
    let results_block = Block::default()
        .title(
            Title::from(" Results ")
                .position(Position::Top)
                .alignment(Alignment::Center),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(get_border_style(
            app::CurrentBlock::Results == app.current_block,
        ))
        .style(Style::default())
        .padding(Padding::right(1));

    let results_list = List::new(app.results_list.results.iter().map(|r| {
        let mut last_match_end = 0;
        let content_line_spans = r.matches.iter().map(|m| {
            let line_text = &r.line;
            let start = m.start;
            let end = m.end;
            let mut spans = vec![];
            if start > last_match_end {
                let chunk_without_tabs =
                    line_text[last_match_end..start].replace("\t", FOUR_SPACES);
                spans.push(
                    Span::raw(chunk_without_tabs).style(
                        Style::default().fg(app
                            .ratatui_theme_settings
                            .foreground
                            .unwrap_or(DEFAULT_RESULT_LINE_FG)),
                    ),
                );
            }
            spans.push(Span::styled(
                line_text[start..end].replace("\t", FOUR_SPACES),
                Style::default().fg(DEFAULT_RESULT_MATCH_COLOR),
            ));
            last_match_end = end;
            spans
        });
        let mut content_spans = content_line_spans.flatten().collect::<Vec<Span>>();
        if last_match_end < r.line.len() {
            content_spans.push(
                Span::raw(&r.line[last_match_end..]).style(
                    Style::default().fg(app
                        .ratatui_theme_settings
                        .foreground
                        .unwrap_or(DEFAULT_RESULT_LINE_FG)),
                ),
            );
        }
        let file_icon = icon_for_file(&File::new(&r.path), None);
        let mut line_spans = vec![
            Span::styled(
                format!("{}{}", file_icon.icon, ' '),
                Style::default()
                    .fg(Color::from_str(file_icon.color).expect(
                        format!("Error parsing hexadecimal color {}", file_icon.color).as_str(),
                    ))
                    .bold(),
            ),
            Span::styled(r.path.to_string_lossy(), Style::default().blue()),
            Span::styled(
                format!(":{}", r.line_number),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(": "),
        ];
        line_spans.extend(content_spans);
        Line::from(line_spans)
    }))
    .highlight_style(
        Style::default().bg(app
            .ratatui_theme_settings
            .inactive_selection
            .unwrap_or(Color::Rgb(50, 50, 50))),
    )
    .highlight_symbol("> ")
    //.repeat_highlight_symbol(true)
    .direction(ListDirection::BottomToTop)
    .block(results_block);

    frame.render_stateful_widget(
        results_list,
        left_chunks[0],
        &mut app.results_list.state.clone(),
    );

    // bottom left block: input field
    let botleft_block = Block::default()
        .title(
            Title::from(" Pattern ")
                .position(Position::Top)
                .alignment(Alignment::Center),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(get_border_style(
            app::CurrentBlock::Search == app.current_block,
        ))
        .style(Style::default());

    let botleft_inner = botleft_block.inner(left_chunks[1]);

    frame.render_widget(botleft_block, left_chunks[1]);

    let total_search_results = app.results_queue.len() + app.results_list.results.len();
    let bottom_left_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2 * ((total_search_results as f32).log10().ceil() as u16 + 1) + 3),
        ])
        .split(botleft_inner);

    let arrow_block = Block::default();
    let arrow = Paragraph::new(Span::styled("> ", Style::default())).block(arrow_block);
    frame.render_widget(arrow, bottom_left_chunks[0]);

    let input_block = Block::default();
    let width = bottom_left_chunks[1].width.max(3) - 3; // keep 2 for borders and 1 for cursor
    let scroll = app.input.visual_scroll(width as usize);
    let input = Paragraph::new(app.input.value())
        .scroll((0, scroll as u16))
        .block(input_block)
        .style(
            Style::default().fg(app
                .ratatui_theme_settings
                .foreground
                .unwrap_or(DEFAULT_INPUT_FG)),
        )
        .alignment(Alignment::Left);
    frame.render_widget(input, bottom_left_chunks[1]);

    if let Some(selected) = app.results_list.state.selected() {
        let result_count_block = Block::default();
        let result_count = Paragraph::new(Span::styled(
            format!(" {} / {} ", selected + 1, total_search_results,),
            Style::default().fg(DEFAULT_RESULTS_COUNT_FG),
        ))
        .block(result_count_block)
        .alignment(Alignment::Right);
        frame.render_widget(result_count, bottom_left_chunks[2]);
    }

    if let app::CurrentBlock::Search = app.current_block {
        // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
        frame.set_cursor_position((
            // Put cursor past the end of the input text
            bottom_left_chunks[1].x + ((app.input.visual_cursor()).max(scroll) - scroll) as u16,
            // Move one line down, from the border to the input line
            bottom_left_chunks[1].y,
        ))
    }

    // current file name
    let result_title: String;
    if let Some(file_name) = &app.preview_state.file_name {
        result_title = file_name.to_string();
    } else {
        result_title = "No results".to_string();
    }
    let preview_file_path = Paragraph::new(result_title)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(get_border_style(false)),
        )
        .style(Style::default().fg(Color::Blue))
        .alignment(Alignment::Left);

    // file preview
    let preview_outer_block = Block::default()
        .title(
            Title::from(" Preview ")
                .position(Position::Top)
                .alignment(Alignment::Center),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(get_border_style(
            app::CurrentBlock::Preview == app.current_block,
        ))
        .style(Style::default())
        .padding(Padding::right(1));

    let preview_inner_block = Block::default().style(Style::default()).padding(Padding {
        top: 0,
        right: 1,
        bottom: 0,
        left: 1,
    });
    let inner = preview_outer_block.inner(right_chunks[1]);
    app.preview_pane_height = inner.height;
    frame.render_widget(preview_outer_block, right_chunks[1]);

    if app.preview_state.file_name.is_some() {
        let result = app.selected_result.as_ref().unwrap();

        let preview_lines: Vec<Line> = app
            .preview_state
            .highlighted_lines
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let line_number_with_style = Span::styled(
                    format!("{:5} ", i + 1),
                    Style::default().fg(if i == result.line_number - 1 {
                        DEFAULT_PREVIEW_GUTTER_SELECTED_FG
                    } else {
                        app.ratatui_theme_settings
                            .gutter_foreground
                            .unwrap_or(DEFAULT_PREVIEW_GUTTER_FG)
                    }),
                );
                Line::from_iter(
                    std::iter::once(line_number_with_style)
                        .chain(std::iter::once(Span::styled(
                            " │ ",
                            Style::default()
                                .fg(app
                                    .ratatui_theme_settings
                                    .gutter_foreground
                                    .unwrap_or(DEFAULT_PREVIEW_GUTTER_FG))
                                .dim(),
                        )))
                        .chain(l.iter().cloned().map(|sr| {
                            convert_syn_region_to_span(
                                (sr.0, sr.1.replace("\t", FOUR_SPACES)),
                                if i == result.line_number - 1 {
                                    Some(SyntectColor {
                                        r: 50,
                                        g: 50,
                                        b: 50,
                                        a: 255,
                                    })
                                } else {
                                    None
                                },
                            )
                        })),
                )
            })
            .collect();
        let preview_text = Text::from(preview_lines);

        let preview_paragraph = Paragraph::new(preview_text)
            .block(preview_inner_block)
            .alignment(Alignment::Left)
            .scroll(app.preview_state.scroll);
        frame.render_widget(preview_paragraph, inner);
    }

    frame.render_widget(preview_file_path, right_chunks[0]);
}
