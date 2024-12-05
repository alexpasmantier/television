use std::fmt::Display;

use crate::mode::{mode_color, Mode};
use ratatui::{
    layout::Constraint,
    style::{Color, Style},
    text::Span,
    widgets::{Cell, Row, Table},
};
use television_channels::channels::UnitChannel;

const METADATA_FIELD_NAME_COLOR: Color = Color::DarkGray;
const METADATA_FIELD_VALUE_COLOR: Color = Color::Gray;

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
) -> Table<'a> {
    let version_row = Row::new(vec![
        Cell::from(Span::styled(
            "version: ",
            Style::default().fg(METADATA_FIELD_NAME_COLOR),
        )),
        Cell::from(Span::styled(
            env!("CARGO_PKG_VERSION"),
            Style::default().fg(METADATA_FIELD_VALUE_COLOR),
        )),
    ]);

    let target_triple_row = Row::new(vec![
        Cell::from(Span::styled(
            "target triple: ",
            Style::default().fg(METADATA_FIELD_NAME_COLOR),
        )),
        Cell::from(Span::styled(
            env!("VERGEN_CARGO_TARGET_TRIPLE"),
            Style::default().fg(METADATA_FIELD_VALUE_COLOR),
        )),
    ]);

    let build_row = Row::new(vec![
        Cell::from(Span::styled(
            "build: ",
            Style::default().fg(METADATA_FIELD_NAME_COLOR),
        )),
        Cell::from(Span::styled(
            env!("VERGEN_RUSTC_SEMVER"),
            Style::default().fg(METADATA_FIELD_VALUE_COLOR),
        )),
        Cell::from(Span::styled(
            " (",
            Style::default().fg(METADATA_FIELD_NAME_COLOR),
        )),
        Cell::from(Span::styled(
            env!("VERGEN_BUILD_DATE"),
            Style::default().fg(METADATA_FIELD_VALUE_COLOR),
        )),
        Cell::from(Span::styled(
            ")",
            Style::default().fg(METADATA_FIELD_NAME_COLOR),
        )),
    ]);

    let current_dir_row = Row::new(vec![
        Cell::from(Span::styled(
            "current directory: ",
            Style::default().fg(METADATA_FIELD_NAME_COLOR),
        )),
        Cell::from(Span::styled(
            std::env::current_dir()
                .expect("Could not get current directory")
                .display()
                .to_string(),
            Style::default().fg(METADATA_FIELD_VALUE_COLOR),
        )),
    ]);

    let current_channel_row = Row::new(vec![
        Cell::from(Span::styled(
            "current channel: ",
            Style::default().fg(METADATA_FIELD_NAME_COLOR),
        )),
        Cell::from(Span::styled(
            current_channel.to_string(),
            Style::default().fg(METADATA_FIELD_VALUE_COLOR),
        )),
    ]);

    let current_mode_row = Row::new(vec![
        Cell::from(Span::styled(
            "current mode: ",
            Style::default().fg(METADATA_FIELD_NAME_COLOR),
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
