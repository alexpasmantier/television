use color_eyre::Result;
use futures::executor::block_on;
use ratatui::{
    layout::{
        Alignment, Constraint, Direction, Layout as RatatuiLayout, Rect,
    },
    style::{Color, Style},
    text::{Line, Span},
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders, ListState, Padding, Paragraph,
    },
    Frame,
};
use std::{collections::HashMap, str::FromStr};
use tokio::sync::mpsc::UnboundedSender;

use crate::channels::{CliTvChannel, TelevisionChannel};
use crate::entry::{Entry, ENTRY_PLACEHOLDER};
use crate::previewers::Previewer;
use crate::ui::get_border_style;
use crate::ui::input::actions::InputActionHandler;
use crate::ui::input::Input;
use crate::ui::layout::{Dimensions, Layout};
use crate::ui::preview::DEFAULT_PREVIEW_TITLE_FG;
use crate::ui::results::build_results_list;
use crate::utils::strings::EMPTY_STRING;
use crate::{action::Action, config::Config};

#[derive(PartialEq, Copy, Clone)]
enum Pane {
    Results,
    Preview,
    Input,
}

static PANES: [Pane; 3] = [Pane::Input, Pane::Results, Pane::Preview];

pub struct Television {
    action_tx: Option<UnboundedSender<Action>>,
    config: Config,
    channel: Box<dyn TelevisionChannel>,
    current_pattern: String,
    current_pane: Pane,
    input: Input,
    picker_state: ListState,
    relative_picker_state: ListState,
    picker_view_offset: usize,
    results_area_height: u32,
    previewer: Previewer,
    pub preview_scroll: Option<u16>,
    pub(crate) preview_pane_height: u16,
    current_preview_total_lines: u16,
    pub(crate) meta_paragraph_cache: HashMap<String, Paragraph<'static>>,
}

impl Television {
    #[must_use]
    pub fn new(cli_channel: CliTvChannel) -> Self {
        let mut tv_channel = cli_channel.to_channel();
        tv_channel.find(EMPTY_STRING);

        Self {
            action_tx: None,
            config: Config::default(),
            channel: tv_channel,
            current_pattern: EMPTY_STRING.to_string(),
            current_pane: Pane::Input,
            input: Input::new(EMPTY_STRING.to_string()),
            picker_state: ListState::default(),
            relative_picker_state: ListState::default(),
            picker_view_offset: 0,
            results_area_height: 0,
            previewer: Previewer::new(),
            preview_scroll: None,
            preview_pane_height: 0,
            current_preview_total_lines: 0,
            meta_paragraph_cache: HashMap::new(),
        }
    }

    fn find(&mut self, pattern: &str) {
        self.channel.find(pattern);
    }

    #[must_use]
    /// # Panics
    /// This method will panic if the index doesn't fit into an u32.
    pub fn get_selected_entry(&self) -> Option<Entry> {
        self.picker_state
            .selected()
            .and_then(|i| self.channel.get_result(u32::try_from(i).unwrap()))
    }

    pub fn select_prev_entry(&mut self) {
        if self.channel.result_count() == 0 {
            return;
        }
        let new_index = (self.picker_state.selected().unwrap_or(0) + 1)
            % self.channel.result_count() as usize;
        self.picker_state.select(Some(new_index));
        if new_index == 0 {
            self.picker_view_offset = 0;
            self.relative_picker_state.select(Some(0));
            return;
        }
        if self.relative_picker_state.selected().unwrap_or(0)
            == self.results_area_height as usize - 3
        {
            self.picker_view_offset += 1;
            self.relative_picker_state.select(Some(
                self.picker_state
                    .selected()
                    .unwrap_or(0)
                    .min(self.results_area_height as usize - 3),
            ));
        } else {
            self.relative_picker_state.select(Some(
                (self.relative_picker_state.selected().unwrap_or(0) + 1)
                    .min(self.picker_state.selected().unwrap_or(0)),
            ));
        }
    }

    pub fn select_next_entry(&mut self) {
        if self.channel.result_count() == 0 {
            return;
        }
        let selected = self.picker_state.selected().unwrap_or(0);
        let relative_selected =
            self.relative_picker_state.selected().unwrap_or(0);
        if selected > 0 {
            self.picker_state.select(Some(selected - 1));
            self.relative_picker_state
                .select(Some(relative_selected.saturating_sub(1)));
            if relative_selected == 0 {
                self.picker_view_offset =
                    self.picker_view_offset.saturating_sub(1);
            }
        } else {
            self.picker_view_offset = self
                .channel
                .result_count()
                .saturating_sub(self.results_area_height - 2)
                as usize;
            self.picker_state.select(Some(
                (self.channel.result_count() as usize).saturating_sub(1),
            ));
            self.relative_picker_state
                .select(Some(self.results_area_height as usize - 3));
        }
    }

    fn reset_preview_scroll(&mut self) {
        self.preview_scroll = None;
    }

    pub fn scroll_preview_down(&mut self, offset: u16) {
        if self.preview_scroll.is_none() {
            self.preview_scroll = Some(0);
        }
        if let Some(scroll) = self.preview_scroll {
            self.preview_scroll = Some(
                (scroll + offset).min(
                    self.current_preview_total_lines
                        .saturating_sub(2 * self.preview_pane_height / 3),
                ),
            );
        }
    }

    pub fn scroll_preview_up(&mut self, offset: u16) {
        if let Some(scroll) = self.preview_scroll {
            self.preview_scroll = Some(scroll.saturating_sub(offset));
        }
    }

    fn get_current_pane_index(&self) -> usize {
        PANES
            .iter()
            .position(|pane| *pane == self.current_pane)
            .unwrap()
    }

    pub fn next_pane(&mut self) {
        let current_index = self.get_current_pane_index();
        let next_index = (current_index + 1) % PANES.len();
        self.current_pane = PANES[next_index];
    }

    pub fn previous_pane(&mut self) {
        let current_index = self.get_current_pane_index();
        let previous_index = if current_index == 0 {
            PANES.len() - 1
        } else {
            current_index - 1
        };
        self.current_pane = PANES[previous_index];
    }

    /// ┌───────────────────┐┌─────────────┐
    /// │ Results           ││ Preview     │
    /// │                   ││             │
    /// │                   ││             │
    /// │                   ││             │
    /// └───────────────────┘│             │
    /// ┌───────────────────┐│             │
    /// │ Search          x ││             │
    /// └───────────────────┘└─────────────┘
    pub fn move_to_pane_on_top(&mut self) {
        if self.current_pane == Pane::Input {
            self.current_pane = Pane::Results;
        }
    }

    /// ┌───────────────────┐┌─────────────┐
    /// │ Results         x ││ Preview     │
    /// │                   ││             │
    /// │                   ││             │
    /// │                   ││             │
    /// └───────────────────┘│             │
    /// ┌───────────────────┐│             │
    /// │ Search            ││             │
    /// └───────────────────┘└─────────────┘
    pub fn move_to_pane_below(&mut self) {
        if self.current_pane == Pane::Results {
            self.current_pane = Pane::Input;
        }
    }

    /// ┌───────────────────┐┌─────────────┐
    /// │ Results         x ││ Preview     │
    /// │                   ││             │
    /// │                   ││             │
    /// │                   ││             │
    /// └───────────────────┘│             │
    /// ┌───────────────────┐│             │
    /// │ Search          x ││             │
    /// └───────────────────┘└─────────────┘
    pub fn move_to_pane_right(&mut self) {
        match self.current_pane {
            Pane::Results | Pane::Input => {
                self.current_pane = Pane::Preview;
            }
            Pane::Preview => {}
        }
    }

    /// ┌───────────────────┐┌─────────────┐
    /// │ Results           ││ Preview   x │
    /// │                   ││             │
    /// │                   ││             │
    /// │                   ││             │
    /// └───────────────────┘│             │
    /// ┌───────────────────┐│             │
    /// │ Search            ││             │
    /// └───────────────────┘└─────────────┘
    pub fn move_to_pane_left(&mut self) {
        if self.current_pane == Pane::Preview {
            self.current_pane = Pane::Results;
        }
    }

    #[must_use]
    pub fn is_input_focused(&self) -> bool {
        Pane::Input == self.current_pane
    }
}

// Styles
//  input
const DEFAULT_INPUT_FG: Color = Color::Rgb(200, 200, 200);
const DEFAULT_RESULTS_COUNT_FG: Color = Color::Rgb(150, 150, 150);

impl Television {
    /// Register an action handler that can send actions for processing if necessary.
    ///
    /// # Arguments
    ///
    /// * `tx` - An unbounded sender that can send actions.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    pub fn register_action_handler(
        &mut self,
        tx: UnboundedSender<Action>,
    ) -> Result<()> {
        self.action_tx = Some(tx.clone());
        Ok(())
    }

    /// Register a configuration handler that provides configuration settings if necessary.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration settings.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    pub fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    /// Update the state of the component based on a received action.
    ///
    /// # Arguments
    ///
    /// * `action` - An action that may modify the state of the television.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    pub async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::GoToPaneUp => {
                self.move_to_pane_on_top();
            }
            Action::GoToPaneDown => {
                self.move_to_pane_below();
            }
            Action::GoToPaneLeft => {
                self.move_to_pane_left();
            }
            Action::GoToPaneRight => {
                self.move_to_pane_right();
            }
            Action::GoToNextPane => {
                self.next_pane();
            }
            Action::GoToPrevPane => {
                self.previous_pane();
            }
            // handle input actions
            Action::AddInputChar(_)
            | Action::DeletePrevChar
            | Action::DeleteNextChar
            | Action::GoToInputEnd
            | Action::GoToInputStart
            | Action::GoToNextChar
            | Action::GoToPrevChar
                if self.is_input_focused() =>
            {
                self.input.handle_action(&action);
                match action {
                    Action::AddInputChar(_)
                    | Action::DeletePrevChar
                    | Action::DeleteNextChar => {
                        let new_pattern = self.input.value().to_string();
                        if new_pattern != self.current_pattern {
                            self.current_pattern.clone_from(&new_pattern);
                            self.find(&new_pattern);
                            self.reset_preview_scroll();
                            self.picker_state.select(Some(0));
                            self.relative_picker_state.select(Some(0));
                            self.picker_view_offset = 0;
                        }
                    }
                    _ => {}
                }
            }
            Action::SelectNextEntry => {
                self.select_next_entry();
                self.reset_preview_scroll();
            }
            Action::SelectPrevEntry => {
                self.select_prev_entry();
                self.reset_preview_scroll();
            }
            Action::ScrollPreviewDown => self.scroll_preview_down(1),
            Action::ScrollPreviewUp => self.scroll_preview_up(1),
            Action::ScrollPreviewHalfPageDown => self.scroll_preview_down(20),
            Action::ScrollPreviewHalfPageUp => self.scroll_preview_up(20),
            _ => {}
        }
        Ok(None)
    }

    /// Render the television on the screen.
    ///
    /// # Arguments
    ///
    /// * `f` - A frame used for rendering.
    /// * `area` - The area in which the television should be drawn.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let layout = Layout::all_panes_centered(Dimensions::default(), area);
        //let layout =
        //    Layout::results_only_centered(Dimensions::new(40, 60), area);

        self.results_area_height = u32::from(layout.results.height);
        if let Some(preview_window) = layout.preview_window {
            self.preview_pane_height = preview_window.height;
        }

        // top left block: results
        let results_block = Block::default()
            .title(
                Title::from(" Results ")
                    .position(Position::Top)
                    .alignment(Alignment::Center),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(get_border_style(Pane::Results == self.current_pane))
            .style(Style::default())
            .padding(Padding::right(1));

        if self.channel.result_count() > 0
            && self.picker_state.selected().is_none()
        {
            self.picker_state.select(Some(0));
            self.relative_picker_state.select(Some(0));
        }

        let entries = self.channel.results(
            (layout.results.height - 2).into(),
            u32::try_from(self.picker_view_offset)?,
        );
        let results_list = build_results_list(results_block, &entries);

        frame.render_stateful_widget(
            results_list,
            layout.results,
            &mut self.relative_picker_state,
        );

        // bottom left block: input
        let input_block = Block::default()
            .title(
                Title::from(" Pattern ")
                    .position(Position::Top)
                    .alignment(Alignment::Center),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(get_border_style(Pane::Input == self.current_pane))
            .style(Style::default());

        let input_block_inner = input_block.inner(layout.input);

        frame.render_widget(input_block, layout.input);

        let inner_input_chunks = RatatuiLayout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(
                    3 * ((self.channel.total_count() as f32).log10().ceil()
                        as u16
                        + 1)
                        + 3,
                ),
            ])
            .split(input_block_inner);

        let arrow_block = Block::default();
        let arrow = Paragraph::new(Span::styled("> ", Style::default()))
            .block(arrow_block);
        frame.render_widget(arrow, inner_input_chunks[0]);

        let interactive_input_block = Block::default();
        // keep 2 for borders and 1 for cursor
        let width = inner_input_chunks[1].width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);
        let input = Paragraph::new(self.input.value())
            .scroll((0, u16::try_from(scroll)?))
            .block(interactive_input_block)
            .style(Style::default().fg(DEFAULT_INPUT_FG))
            .alignment(Alignment::Left);
        frame.render_widget(input, inner_input_chunks[1]);

        let result_count_block = Block::default();
        let result_count = Paragraph::new(Span::styled(
            format!(
                " {} / {} ",
                if self.channel.result_count() == 0 {
                    0
                } else {
                    self.picker_state.selected().unwrap_or(0) + 1
                },
                self.channel.result_count(),
            ),
            Style::default().fg(DEFAULT_RESULTS_COUNT_FG),
        ))
        .block(result_count_block)
        .alignment(Alignment::Right);
        frame.render_widget(result_count, inner_input_chunks[2]);

        if let Pane::Input = self.current_pane {
            // Make the cursor visible and ask tui-rs to put it at the
            // specified coordinates after rendering
            frame.set_cursor_position((
                // Put cursor past the end of the input text
                inner_input_chunks[1].x
                    + u16::try_from(
                        self.input.visual_cursor().max(scroll) - scroll,
                    )?,
                // Move one line down, from the border to the input line
                inner_input_chunks[1].y,
            ));
        }

        if layout.preview_title.is_some() || layout.preview_window.is_some() {
            let selected_entry =
                self.get_selected_entry().unwrap_or(ENTRY_PLACEHOLDER);
            let preview = block_on(self.previewer.preview(&selected_entry));

            if let Some(preview_title_area) = layout.preview_title {
                // top right block: preview title
                self.current_preview_total_lines = preview.total_lines();

                let mut preview_title_spans = Vec::new();
                if let Some(icon) = &selected_entry.icon {
                    preview_title_spans.push(Span::styled(
                        icon.to_string(),
                        Style::default().fg(Color::from_str(icon.color)?),
                    ));
                    preview_title_spans.push(Span::raw(" "));
                }
                preview_title_spans.push(Span::styled(
                    preview.title.clone(),
                    Style::default().fg(DEFAULT_PREVIEW_TITLE_FG),
                ));
                let preview_title =
                    Paragraph::new(Line::from(preview_title_spans))
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .border_type(BorderType::Rounded)
                                .border_style(get_border_style(false)),
                        )
                        .alignment(Alignment::Left);
                frame.render_widget(preview_title, preview_title_area);
            }

            if let Some(preview_area) = layout.preview_window {
                // file preview
                let preview_outer_block = Block::default()
                    .title(
                        Title::from(" Preview ")
                            .position(Position::Top)
                            .alignment(Alignment::Center),
                    )
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(get_border_style(
                        Pane::Preview == self.current_pane,
                    ))
                    .style(Style::default())
                    .padding(Padding::right(1));

                let preview_inner_block = Block::default()
                    .style(Style::default())
                    .padding(Padding {
                        top: 0,
                        right: 1,
                        bottom: 0,
                        left: 1,
                    });
                let inner = preview_outer_block.inner(preview_area);
                frame.render_widget(preview_outer_block, preview_area);

                //if let PreviewContent::Image(img) = &preview.content {
                //    let image_component = StatefulImage::new(None);
                //    frame.render_stateful_widget(
                //        image_component,
                //        inner,
                //        &mut img.clone(),
                //    );
                //} else {
                let preview_block = self.build_preview_paragraph(
                    preview_inner_block,
                    inner,
                    &preview,
                    selected_entry
                        .line_number
                        // FIXME: this actually might panic in some edge cases
                        .map(|l| u16::try_from(l).unwrap()),
                );
                frame.render_widget(preview_block, inner);
                //}
            }
        }
        Ok(())
    }
}
