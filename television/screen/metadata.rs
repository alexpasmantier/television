use std::fmt::Display;

use crate::screen::{colors::Colorscheme, mode::mode_color};
use crate::television::Mode;
use crate::utils::metadata::AppMetadata;
use ratatui::{
    layout::Constraint,
    style::Style,
    text::Span,
    widgets::{Cell, Row, Table},
};

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Channel => write!(f, "Channel"),
            Mode::RemoteControl => write!(f, "Remote Control"),
        }
    }
}

pub fn build_metadata_table<'a>(
    mode: Mode,
    current_channel_name: &'a str,
    app_metadata: &'a AppMetadata,
    colorscheme: &'a Colorscheme,
) -> Table<'a> {
    let version_row = Row::new(vec![
        Cell::from(Span::styled(
            "version: ",
            Style::default().fg(colorscheme.help.metadata_field_name_fg),
        )),
        Cell::from(Span::styled(
            &app_metadata.version,
            Style::default().fg(colorscheme.help.metadata_field_value_fg),
        )),
    ]);

    let current_dir_row = Row::new(vec![
        Cell::from(Span::styled(
            "current directory: ",
            Style::default().fg(colorscheme.help.metadata_field_name_fg),
        )),
        Cell::from(Span::styled(
            std::env::current_dir()
                .expect("Could not get current directory")
                .display()
                .to_string(),
            Style::default().fg(colorscheme.help.metadata_field_value_fg),
        )),
    ]);

    let current_channel_row = Row::new(vec![
        Cell::from(Span::styled(
            "current channel: ",
            Style::default().fg(colorscheme.help.metadata_field_name_fg),
        )),
        Cell::from(Span::styled(
            current_channel_name,
            Style::default().fg(colorscheme.help.metadata_field_value_fg),
        )),
    ]);

    let current_mode_row = Row::new(vec![
        Cell::from(Span::styled(
            "current mode: ",
            Style::default().fg(colorscheme.help.metadata_field_name_fg),
        )),
        Cell::from(Span::styled(
            mode.to_string(),
            Style::default().fg(mode_color(mode, &colorscheme.mode)),
        )),
    ]);

    let widths = vec![Constraint::Fill(1), Constraint::Fill(2)];

    Table::new(
        vec![
            version_row,
            current_dir_row,
            current_channel_row,
            current_mode_row,
        ],
        widths,
    )
}
