use ratatui::{
    Frame,
    layout::{
        Alignment, Constraint, Direction, Layout as RatatuiLayout, Rect,
    },
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    config::ui::UiConfig, screen::colors::Colorscheme, television::Mode,
};

/// Draw the status bar at the bottom of the screen
#[allow(clippy::too_many_arguments)]
pub fn draw_status_bar(
    f: &mut Frame<'_>,
    area: Rect,
    selected_count: usize,
    current_channel: &str,
    version: &str,
    mode: Mode,
    colorscheme: &Colorscheme,
    ui_config: &UiConfig,
) {
    // Split the status bar into left and right sections
    let chunks = RatatuiLayout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1), // Left section (mode + channel + results)
            Constraint::Fill(1), // Right section (version)
        ])
        .split(area);

    // Create left section with mode bubble and channel info
    let mut left_spans = Vec::new();
    left_spans.push(Span::raw(" ")); // Initial spacing

    // Add mode bubble with separators
    let (mode_text, mode_style, mode_bg) = match mode {
        Mode::Channel => (
            "CHANNEL",
            Style::default()
                .fg(colorscheme.mode.channel_fg)
                .bg(colorscheme.mode.channel)
                .add_modifier(Modifier::BOLD),
            colorscheme.mode.channel,
        ),
        Mode::RemoteControl => (
            "REMOTE",
            Style::default()
                .fg(colorscheme.mode.remote_control_fg)
                .bg(colorscheme.mode.remote_control)
                .add_modifier(Modifier::BOLD),
            colorscheme.mode.remote_control,
        ),
    };

    let separator_style = Style::default()
        .fg(mode_bg)
        .bg(ratatui::style::Color::Reset);

    // Add mode separator start
    if !ui_config.status_separator_open.is_empty() {
        left_spans.push(Span::styled(
            ui_config.status_separator_open.clone(),
            separator_style,
        ));
    }

    // Add mode text
    left_spans.push(Span::styled(format!(" {} ", mode_text), mode_style));

    // Add mode separator end
    if !ui_config.status_separator_close.is_empty() {
        left_spans.push(Span::styled(
            ui_config.status_separator_close.clone(),
            separator_style,
        ));
    }

    // Add channel info only in Channel mode
    if mode == Mode::Channel {
        // Channel name
        left_spans.push(Span::styled(
            format!(" {}", current_channel),
            Style::default()
                .fg(colorscheme.results.result_name_fg)
                .add_modifier(Modifier::BOLD),
        ));

        // Selected count (if any)
        if selected_count > 0 {
            left_spans.push(Span::styled(
                " • ",
                Style::default().fg(colorscheme.general.border_fg),
            ));
            left_spans.push(Span::styled(
                format!("{} selected", selected_count),
                Style::default()
                    .fg(colorscheme.results.result_name_fg)
                    .add_modifier(Modifier::ITALIC),
            ));
        }
    }

    // Render left section
    let left_paragraph =
        Paragraph::new(Line::from(left_spans)).alignment(Alignment::Left);
    f.render_widget(left_paragraph, chunks[0]);

    // Create and render right section (version)
    let right_spans = vec![Span::styled(
        format!("v{} ", version),
        Style::default()
            .fg(colorscheme.results.result_name_fg)
            .add_modifier(Modifier::ITALIC),
    )];

    let right_paragraph =
        Paragraph::new(Line::from(right_spans)).alignment(Alignment::Right);
    f.render_widget(right_paragraph, chunks[1]);
}
