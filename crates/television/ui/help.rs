use color_eyre::eyre::{OptionExt, Result};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use tracing::debug;

use crate::{
    action::Action,
    config::Config,
    event::Key,
    television::{Mode, Television},
};

const SEPARATOR: &str = "  ";
const ACTION_COLOR: Color = Color::DarkGray;
const KEY_COLOR: Color = Color::LightYellow;

impl Television {
    pub fn build_help_paragraph<'a>(
        &self,
        width: u16,
    ) -> Result<Paragraph<'a>> {
        let keymap = self
            .config
            .keybindings
            .get(&self.mode)
            .ok_or_eyre("No keybindings found for the current Mode")?;

        let mut help_spans = Vec::new();

        // Results navigation
        let prev: Vec<_> = keymap
            .iter()
            .filter(|(_key, action)| **action == Action::SelectPrevEntry)
            .map(|(key, _action)| format!("{key}"))
            .collect();

        let next: Vec<_> = keymap
            .iter()
            .filter(|(_key, action)| **action == Action::SelectNextEntry)
            .map(|(key, _action)| format!("{key}"))
            .collect();

        let results_spans = vec![
            Span::styled("↕ Results: [", Style::default().fg(ACTION_COLOR)),
            Span::styled(prev.join(", "), Style::default().fg(KEY_COLOR)),
            Span::styled(" | ", Style::default().fg(ACTION_COLOR)),
            Span::styled(next.join(", "), Style::default().fg(KEY_COLOR)),
            Span::styled("]", Style::default().fg(ACTION_COLOR)),
        ];

        help_spans.extend(results_spans);
        help_spans.push(Span::styled(SEPARATOR, Style::default()));

        if self.mode == Mode::Channel {
            // Preview navigation
            let up: Vec<_> = keymap
                .iter()
                .filter(|(_key, action)| {
                    **action == Action::ScrollPreviewHalfPageUp
                })
                .map(|(key, _action)| format!("{key}"))
                .collect();

            let down: Vec<_> = keymap
                .iter()
                .filter(|(_key, action)| {
                    **action == Action::ScrollPreviewHalfPageDown
                })
                .map(|(key, _action)| format!("{key}"))
                .collect();

            let preview_spans = vec![
                Span::styled(
                    "↕ Preview: [",
                    Style::default().fg(ACTION_COLOR),
                ),
                Span::styled(up.join(", "), Style::default().fg(KEY_COLOR)),
                Span::styled(" | ", Style::default().fg(ACTION_COLOR)),
                Span::styled(down.join(", "), Style::default().fg(KEY_COLOR)),
                Span::styled("]", Style::default().fg(ACTION_COLOR)),
            ];

            help_spans.extend(preview_spans);
            help_spans.push(Span::styled(SEPARATOR, Style::default()));

            // Channels
            let channels: Vec<_> = keymap
                .iter()
                .filter(|(_key, action)| {
                    **action == Action::ToChannelSelection
                })
                .map(|(key, _action)| format!("{key}"))
                .collect();

            let channels_spans = vec![
                Span::styled("Channels: [", Style::default().fg(ACTION_COLOR)),
                Span::styled(
                    channels.join(", "),
                    Style::default().fg(KEY_COLOR),
                ),
                Span::styled("]", Style::default().fg(ACTION_COLOR)),
            ];

            help_spans.extend(channels_spans);
            help_spans.push(Span::styled(SEPARATOR, Style::default()));
        }

        if self.mode == Mode::ChannelSelection {
            // Pipe into
            let channels: Vec<_> = keymap
                .iter()
                .filter(|(_key, action)| **action == Action::PipeInto)
                .map(|(key, _action)| format!("{key}"))
                .collect();

            let channels_spans = vec![
                Span::styled(
                    "Pipe into: [",
                    Style::default().fg(ACTION_COLOR),
                ),
                Span::styled(
                    channels.join(", "),
                    Style::default().fg(KEY_COLOR),
                ),
                Span::styled("]", Style::default().fg(ACTION_COLOR)),
            ];

            help_spans.extend(channels_spans);
            help_spans.push(Span::styled(SEPARATOR, Style::default()));

            // Select Channel
            let select: Vec<_> = keymap
                .iter()
                .filter(|(_key, action)| **action == Action::SelectEntry)
                .map(|(key, _action)| format!("{key}"))
                .collect();

            let select_spans = vec![
                Span::styled("Select: [", Style::default().fg(ACTION_COLOR)),
                Span::styled(
                    select.join(", "),
                    Style::default().fg(KEY_COLOR),
                ),
                Span::styled("]", Style::default().fg(ACTION_COLOR)),
            ];

            help_spans.extend(select_spans);
            help_spans.push(Span::styled(SEPARATOR, Style::default()));
        }

        // Quit
        let quit: Vec<_> = keymap
            .iter()
            .filter(|(_key, action)| **action == Action::Quit)
            .map(|(key, _action)| format!("{key}"))
            .collect();

        let quit_spans = vec![
            Span::styled("Quit: [", Style::default().fg(ACTION_COLOR)),
            Span::styled(quit.join(", "), Style::default().fg(KEY_COLOR)),
            Span::styled("]", Style::default().fg(ACTION_COLOR)),
        ];

        help_spans.extend(quit_spans);

        // arrange lines depending on the width
        let mut lines = Vec::new();
        let mut current_line = Line::default();
        let mut current_width = 0;

        for span in help_spans {
            let span_width = span.content.chars().count() as u16;
            if current_width + span_width > width {
                lines.push(current_line);
                current_line = Line::default();
                current_width = 0;
            }

            current_line.push_span(span);
            current_width += span_width;
        }

        lines.push(current_line);

        Ok(Paragraph::new(lines))
    }
}
