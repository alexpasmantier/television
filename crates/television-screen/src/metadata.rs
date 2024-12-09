use std::fmt::Display;

use crate::{
    colors::HelpColorscheme,
    mode::{mode_color, Mode},
};
use ratatui::{
    layout::Constraint,
    style::Style,
    text::Span,
    widgets::{Cell, Row, Table},
};
use television_channels::channels::UnitChannel;
use television_utils::metadata::AppMetadata;

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Channel => write!(f, "Channel"),
            Mode::RemoteControl => write!(f, "Remote Control"),
            Mode::SendToChannel => write!(f, "Send to Channel"),
        }
    }
}

pub fn build_metadata_table<'a>(
    mode: Mode,
    current_channel: UnitChannel,
    app_metadata: &'a AppMetadata,
    help_colorscheme: &'a HelpColorscheme,
) -> Table<'a> {
    let version_row = Row::new(vec![
        Cell::from(Span::styled(
            "version: ",
            Style::default().fg(help_colorscheme.metadata_field_name_fg),
        )),
        Cell::from(Span::styled(
            &app_metadata.version,
            Style::default().fg(help_colorscheme.metadata_field_value_fg),
        )),
    ]);

    let target_triple_row = Row::new(vec![
        Cell::from(Span::styled(
            "target triple: ",
            Style::default().fg(help_colorscheme.metadata_field_name_fg),
        )),
        Cell::from(Span::styled(
            &app_metadata.build.target_triple,
            Style::default().fg(help_colorscheme.metadata_field_value_fg),
        )),
    ]);

    let build_row = Row::new(vec![
        Cell::from(Span::styled(
            "build: ",
            Style::default().fg(help_colorscheme.metadata_field_name_fg),
        )),
        Cell::from(Span::styled(
            &app_metadata.build.rustc_version,
            Style::default().fg(help_colorscheme.metadata_field_value_fg),
        )),
        Cell::from(Span::styled(
            " (",
            Style::default().fg(help_colorscheme.metadata_field_name_fg),
        )),
        Cell::from(Span::styled(
            &app_metadata.build.build_date,
            Style::default().fg(help_colorscheme.metadata_field_value_fg),
        )),
        Cell::from(Span::styled(
            ")",
            Style::default().fg(help_colorscheme.metadata_field_name_fg),
        )),
    ]);

    let current_dir_row = Row::new(vec![
        Cell::from(Span::styled(
            "current directory: ",
            Style::default().fg(help_colorscheme.metadata_field_name_fg),
        )),
        Cell::from(Span::styled(
            std::env::current_dir()
                .expect("Could not get current directory")
                .display()
                .to_string(),
            Style::default().fg(help_colorscheme.metadata_field_value_fg),
        )),
    ]);

    let current_channel_row = Row::new(vec![
        Cell::from(Span::styled(
            "current channel: ",
            Style::default().fg(help_colorscheme.metadata_field_name_fg),
        )),
        Cell::from(Span::styled(
            current_channel.to_string(),
            Style::default().fg(help_colorscheme.metadata_field_value_fg),
        )),
    ]);

    let current_mode_row = Row::new(vec![
        Cell::from(Span::styled(
            "current mode: ",
            Style::default().fg(help_colorscheme.metadata_field_name_fg),
        )),
        Cell::from(Span::styled(
            mode.to_string(),
            Style::default().fg(mode_color(mode)),
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
