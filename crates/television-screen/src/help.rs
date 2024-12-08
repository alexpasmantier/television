use super::layout::HelpBarLayout;
use crate::colors::BORDER_COLOR;
use crate::logo::build_logo_paragraph;
use crate::metadata::build_metadata_table;
use crate::mode::{mode_color, Mode};
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Style};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Table};
use ratatui::Frame;
use television_channels::channels::UnitChannel;
use television_utils::metadata::AppMetadata;

pub fn draw_logo_block(f: &mut Frame, area: Rect, color: Color) {
    let logo_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER_COLOR))
        .style(Style::default().fg(color))
        .padding(Padding::horizontal(1));

    let logo_paragraph = build_logo_paragraph().block(logo_block);

    f.render_widget(logo_paragraph, area);
}

fn draw_metadata_block(
    f: &mut Frame,
    area: Rect,
    mode: Mode,
    current_channel: UnitChannel,
    app_metadata: &AppMetadata,
) {
    let metadata_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Blue))
        .padding(Padding::horizontal(1))
        .style(Style::default());

    let metadata_table =
        build_metadata_table(mode, current_channel, app_metadata)
            .block(metadata_block);

    f.render_widget(metadata_table, area);
}

fn draw_keymaps_block(f: &mut Frame, area: Rect, keymap_table: Table) {
    let keymaps_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Blue))
        .style(Style::default())
        .padding(Padding::horizontal(1));

    let keymaps_table = keymap_table.block(keymaps_block);

    f.render_widget(keymaps_table, area);
}

pub fn draw_help_bar(
    f: &mut Frame,
    layout: &Option<HelpBarLayout>,
    current_channel: UnitChannel,
    keymap_table: Table,
    mode: Mode,
    app_metadata: &AppMetadata,
) {
    if let Some(help_bar) = layout {
        draw_metadata_block(
            f,
            help_bar.left,
            mode,
            current_channel,
            app_metadata,
        );
        draw_keymaps_block(f, help_bar.middle, keymap_table);
        draw_logo_block(f, help_bar.right, mode_color(mode));
    }
}
