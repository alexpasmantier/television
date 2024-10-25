use ratatui::{
    layout::Constraint,
    style::{Color, Style},
    text::Span,
    widgets::{Cell, Row, Table},
};

use crate::television::Television;

// television 0.1.6
// target triple: aarch64-apple-darwin
// build: 1.82.0 (2024-10-24)
// current_channel: git_repos
// current_mode: channel

impl Television {
    pub fn build_metadata_table<'a>(&self) -> Table<'a> {
        let version_row = Row::new(vec![
            Cell::from(Span::styled(
                "version: ",
                Style::default().fg(Color::DarkGray),
            )),
            Cell::from(Span::styled(
                env!("CARGO_PKG_VERSION"),
                Style::default().fg(Color::LightYellow),
            )),
        ]);

        let target_triple_row = Row::new(vec![
            Cell::from(Span::styled(
                "target triple: ",
                Style::default().fg(Color::DarkGray),
            )),
            Cell::from(Span::styled(
                env!("VERGEN_CARGO_TARGET_TRIPLE"),
                Style::default().fg(Color::LightYellow),
            )),
        ]);

        let build_row = Row::new(vec![
            Cell::from(Span::styled(
                "build: ",
                Style::default().fg(Color::DarkGray),
            )),
            Cell::from(Span::styled(
                env!("VERGEN_RUSTC_SEMVER"),
                Style::default().fg(Color::LightYellow),
            )),
            Cell::from(Span::styled(
                " (",
                Style::default().fg(Color::DarkGray),
            )),
            Cell::from(Span::styled(
                env!("VERGEN_BUILD_DATE"),
                Style::default().fg(Color::LightYellow),
            )),
            Cell::from(Span::styled(
                ")",
                Style::default().fg(Color::DarkGray),
            )),
        ]);

        let current_dir_row = Row::new(vec![
            Cell::from(Span::styled(
                "current directory: ",
                Style::default().fg(Color::DarkGray),
            )),
            Cell::from(Span::styled(
                std::env::current_dir()
                    .expect("Could not get current directory")
                    .display()
                    .to_string(),
                Style::default().fg(Color::LightYellow),
            )),
        ]);

        let current_channel_row = Row::new(vec![
            Cell::from(Span::styled(
                "current channel: ",
                Style::default().fg(Color::DarkGray),
            )),
            Cell::from(Span::styled(
                self.current_channel().to_string(),
                Style::default().fg(Color::LightYellow),
            )),
        ]);

        let current_mode_row = Row::new(vec![
            Cell::from(Span::styled(
                "current mode: ",
                Style::default().fg(Color::DarkGray),
            )),
            Cell::from(Span::styled(
                self.mode.to_string(),
                Style::default().fg(Color::LightYellow),
            )),
        ]);

        let widths = vec![Constraint::Fill(1), Constraint::Fill(2)];

        Table::new(
            vec![
                version_row,
                target_triple_row,
                build_row,
                current_dir_row,
                current_channel_row,
                current_mode_row,
            ],
            widths,
        )
    }
}
