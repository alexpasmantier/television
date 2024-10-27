use crate::channels::OnAir;
use crate::television::Television;
use crate::ui::get_border_style;
use crate::ui::logo::build_remote_logo_paragraph;
use crate::ui::mode::mode_color;
use crate::ui::results::{build_results_list, ResultsListColors};
use color_eyre::eyre::Result;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::Style;
use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, ListDirection, Padding, Paragraph,
};
use ratatui::Frame;

impl Television {
    pub fn draw_remote_control(
        &mut self,
        f: &mut Frame,
        area: &Rect,
    ) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Fill(1),
                    Constraint::Length(3),
                    Constraint::Length(20),
                ]
                    .as_ref(),
            )
            .split(*area);
        self.draw_rc_channels(f, &layout[0])?;
        self.draw_rc_input(f, &layout[1])?;
        draw_rc_logo(f, layout[2], mode_color(self.mode));
        Ok(())
    }

    fn draw_rc_channels(&mut self, f: &mut Frame, area: &Rect) -> Result<()> {
        let rc_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(get_border_style(false))
            .style(Style::default())
            .padding(Padding::right(1));

        let result_count = self.remote_control.result_count();
        if result_count > 0 && self.rc_picker.selected().is_none() {
            self.rc_picker.select(Some(0));
            self.rc_picker.relative_select(Some(0));
        }

        let entries = self.remote_control.results(
            area.height.saturating_sub(2).into(),
            u32::try_from(self.rc_picker.view_offset)?,
        );

        let channel_list =
            build_results_list(rc_block, &entries, ListDirection::TopToBottom, Some(ResultsListColors::default().result_name_fg(mode_color(self.mode))));

        f.render_stateful_widget(
            channel_list,
            *area,
            &mut self.rc_picker.state,
        );
        Ok(())
    }

    fn draw_rc_input(&mut self, f: &mut Frame, area: &Rect) -> Result<()> {
        let input_block = Block::default()
            .title_top(
                Line::from("Remote Control").alignment(Alignment::Center),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(get_border_style(false))
            .style(Style::default());

        let input_block_inner = input_block.inner(*area);

        f.render_widget(input_block, *area);

        // split input block into 2 parts: prompt symbol, input
        let inner_input_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                // prompt symbol
                Constraint::Length(2),
                // input field
                Constraint::Fill(1),
            ])
            .split(input_block_inner);

        let prompt_symbol_block = Block::default();
        let arrow = Paragraph::new(Span::styled(
            "> ",
            Style::default()
                .fg(crate::television::DEFAULT_INPUT_FG)
                .bold(),
        ))
            .block(prompt_symbol_block);
        f.render_widget(arrow, inner_input_chunks[0]);

        let interactive_input_block = Block::default();
        // keep 2 for borders and 1 for cursor
        let width = inner_input_chunks[1].width.max(3) - 3;
        let scroll = self.rc_picker.input.visual_scroll(width as usize);
        let input = Paragraph::new(self.rc_picker.input.value())
            .scroll((0, u16::try_from(scroll)?))
            .block(interactive_input_block)
            .style(
                Style::default()
                    .fg(crate::television::DEFAULT_INPUT_FG)
                    .bold()
                    .italic(),
            )
            .alignment(Alignment::Left);
        f.render_widget(input, inner_input_chunks[1]);

        // Make the cursor visible and ask tui-rs to put it at the
        // specified coordinates after rendering
        f.set_cursor_position((
            // Put cursor past the end of the input text
            inner_input_chunks[1].x
                + u16::try_from(
                self.rc_picker.input.visual_cursor().max(scroll) - scroll,
            )?,
            // Move one line down, from the border to the input line
            inner_input_chunks[1].y,
        ));
        Ok(())
    }
}

fn draw_rc_logo(f: &mut Frame, area: Rect, color: Color) {
    let logo_block = Block::default()
        .style(Style::default().fg(color));

    let logo_paragraph = build_remote_logo_paragraph()
        .alignment(Alignment::Center)
        .block(logo_block);

    f.render_widget(logo_paragraph, area);
}
