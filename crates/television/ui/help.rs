use super::layout::HelpBarLayout;
use crate::television::Television;
use crate::ui::logo::build_logo_paragraph;
use crate::ui::mode::mode_color;
use crate::ui::BORDER_COLOR;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Style};
use ratatui::widgets::{Block, BorderType, Borders, Padding};
use ratatui::Frame;

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

impl Television {
    pub(crate) fn draw_help_bar(
        &self,
        f: &mut Frame,
        layout: &Option<HelpBarLayout>,
    ) -> color_eyre::Result<()> {
        if let Some(help_bar) = layout {
            self.draw_metadata_block(f, help_bar.left);
            self.draw_keymaps_block(f, help_bar.middle)?;
            draw_logo_block(f, help_bar.right, mode_color(self.mode));
        }
        Ok(())
    }

    fn draw_metadata_block(&self, f: &mut Frame, area: Rect) {
        let metadata_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BORDER_COLOR))
            .padding(Padding::horizontal(1))
            .style(Style::default());

        let metadata_table = self.build_metadata_table().block(metadata_block);

        f.render_widget(metadata_table, area);
    }

    fn draw_keymaps_block(
        &self,
        f: &mut Frame,
        area: Rect,
    ) -> color_eyre::Result<()> {
        let keymaps_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BORDER_COLOR))
            .style(Style::default())
            .padding(Padding::horizontal(1));

        let keymaps_table = self.build_keymap_table()?.block(keymaps_block);

        f.render_widget(keymaps_table, area);
        Ok(())
    }
}
