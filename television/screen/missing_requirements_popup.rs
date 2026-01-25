use crate::{
    screen::colors::Colorscheme, television::MissingRequirementsPopup,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
};

const MIN_POPUP_WIDTH: u16 = 30;
const MIN_POPUP_HEIGHT: u16 = 6;

/// Draws a centered popup dialog showing missing binary requirements.
///
/// This popup is displayed when a user attempts to switch to a channel
/// that has unmet binary requirements from the Remote Control.
pub fn draw_missing_requirements_popup(
    f: &mut Frame<'_>,
    area: Rect,
    popup: &MissingRequirementsPopup,
    colorscheme: &Colorscheme,
) {
    let (popup_width, popup_height) =
        calculate_popup_size(popup, area.width, area.height);
    let popup_area = centered_rect(popup_width, popup_height, area);

    if popup_area.width < MIN_POPUP_WIDTH
        || popup_area.height < MIN_POPUP_HEIGHT
    {
        return;
    }

    let content = generate_popup_content(popup, colorscheme);

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colorscheme.input.results_count_fg))
        .title_top(
            Line::from(Span::styled(
                " Missing Requirements ",
                Style::default()
                    .fg(colorscheme.input.results_count_fg)
                    .bold(),
            ))
            .alignment(Alignment::Center),
        )
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        )
        .padding(Padding::horizontal(1));

    let paragraph = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Left);

    f.render_widget(paragraph, popup_area);
}

fn generate_popup_content(
    popup: &MissingRequirementsPopup,
    colorscheme: &Colorscheme,
) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("Cannot switch to "),
            Span::styled(
                popup.channel_name.clone(),
                Style::default().fg(colorscheme.mode.channel).bold(),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Missing binaries:",
            Style::default().fg(colorscheme.help.metadata_field_name_fg),
        )),
    ];

    for req in &popup.missing_requirements {
        lines.push(Line::from(vec![
            Span::styled(
                "  - ",
                Style::default().fg(colorscheme.input.results_count_fg),
            ),
            Span::styled(
                req.clone(),
                Style::default()
                    .fg(colorscheme.input.results_count_fg)
                    .bold(),
            ),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press Enter or Esc to dismiss",
        Style::default().fg(colorscheme.general.border_fg).italic(),
    )));

    lines
}

#[allow(clippy::cast_possible_truncation)]
fn calculate_popup_size(
    popup: &MissingRequirementsPopup,
    max_width: u16,
    max_height: u16,
) -> (u16, u16) {
    let channel_line_width =
        "Cannot switch to ".len() + popup.channel_name.len();
    let max_req_width = popup
        .missing_requirements
        .iter()
        .map(|r| r.len() + 4)
        .max()
        .unwrap_or(0);
    let dismiss_line_width = "Press Enter or Esc to dismiss".len();

    let content_width = channel_line_width
        .max(max_req_width)
        .max(dismiss_line_width)
        .max("Missing binaries:".len());

    let required_width = (content_width + 6).min(max_width as usize) as u16;
    let required_width = required_width.max(MIN_POPUP_WIDTH);

    let content_lines = 6 + popup.missing_requirements.len();
    let required_height = (content_lines + 2).min(max_height as usize) as u16;
    let required_height = required_height.max(MIN_POPUP_HEIGHT);

    (required_width, required_height)
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical_layout = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height),
        Constraint::Fill(1),
    ])
    .split(area);

    let horizontal_layout = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(width),
        Constraint::Fill(1),
    ])
    .split(vertical_layout[1]);

    horizontal_layout[1]
}
