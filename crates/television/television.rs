use color_eyre::Result;
use futures::executor::block_on;
use ratatui::{
    layout::{
        Alignment, Constraint, Direction, Layout as RatatuiLayout, Rect,
    },
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders, ListState, Padding, Paragraph, Wrap,
    },
    Frame,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use tokio::sync::mpsc::UnboundedSender;

use crate::ui::input::Input;
use crate::ui::layout::{Dimensions, Layout};
use crate::ui::preview::DEFAULT_PREVIEW_TITLE_FG;
use crate::ui::results::build_results_list;
use crate::utils::strings::EMPTY_STRING;
use crate::{action::Action, config::Config};
use crate::{channels::tv_guide::TvGuide, ui::get_border_style};
use crate::{channels::OnAir, utils::strings::shrink_with_ellipsis};
use crate::{
    channels::TelevisionChannel, ui::input::actions::InputActionHandler,
};
use crate::{
    entry::{Entry, ENTRY_PLACEHOLDER},
    ui::spinner::Spinner,
};
use crate::{previewers::Previewer, ui::spinner::SpinnerState};

#[derive(PartialEq, Copy, Clone, Hash, Eq, Debug, Serialize, Deserialize)]
pub enum Mode {
    Channel,
    Guide,
    SendToChannel,
}

pub struct Television {
    action_tx: Option<UnboundedSender<Action>>,
    pub config: Config,
    channel: TelevisionChannel,
    guide: TelevisionChannel,
    current_pattern: String,
    pub mode: Mode,
    input: Input,
    picker_state: ListState,
    relative_picker_state: ListState,
    picker_view_offset: usize,
    results_area_height: u32,
    pub previewer: Previewer,
    pub preview_scroll: Option<u16>,
    pub preview_pane_height: u16,
    current_preview_total_lines: u16,
    /// A cache for meta paragraphs (i.e. previews like "Not Supported", etc.).
    ///
    /// The key is a tuple of the preview name and the dimensions of the
    /// preview pane. This is a little extra security to ensure meta previews
    /// are rendered correctly even when resizing the terminal while still
    /// benefiting from a cache mechanism.
    pub meta_paragraph_cache: HashMap<(String, u16, u16), Paragraph<'static>>,
    spinner: Spinner,
    spinner_state: SpinnerState,
}

impl Television {
    #[must_use]
    pub fn new(mut channel: TelevisionChannel) -> Self {
        channel.find(EMPTY_STRING);
        let guide = TelevisionChannel::TvGuide(TvGuide::new());

        let spinner = Spinner::default();
        let spinner_state = SpinnerState::from(&spinner);

        Self {
            action_tx: None,
            config: Config::default(),
            channel,
            guide,
            current_pattern: EMPTY_STRING.to_string(),
            mode: Mode::Channel,
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
            spinner,
            spinner_state,
        }
    }

    /// FIXME: this needs rework
    pub fn change_channel(&mut self, channel: TelevisionChannel) {
        self.reset_preview_scroll();
        self.reset_results_selection();
        self.current_pattern = EMPTY_STRING.to_string();
        self.input.reset();
        self.channel.shutdown();
        self.channel = channel;
    }

    fn find(&mut self, pattern: &str) {
        match self.mode {
            Mode::Channel => {
                self.channel.find(pattern);
            }
            Mode::Guide | Mode::SendToChannel => {
                self.guide.find(pattern);
            }
        }
    }

    #[must_use]
    pub fn get_selected_entry(&mut self) -> Option<Entry> {
        self.picker_state.selected().and_then(|i| match self.mode {
            Mode::Channel => {
                self.channel.get_result(u32::try_from(i).unwrap())
            }
            Mode::Guide | Mode::SendToChannel => {
                self.guide.get_result(u32::try_from(i).unwrap())
            }
        })
    }

    pub fn select_prev_entry(&mut self) {
        let result_count = match self.mode {
            Mode::Channel => self.channel.result_count(),
            Mode::Guide | Mode::SendToChannel => self.guide.total_count(),
        };
        if result_count == 0 {
            return;
        }
        let new_index = (self.picker_state.selected().unwrap_or(0) + 1)
            % result_count as usize;
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
        let result_count = match self.mode {
            Mode::Channel => self.channel.result_count(),
            Mode::Guide | Mode::SendToChannel => self.guide.total_count(),
        };
        if result_count == 0 {
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
            self.picker_view_offset = result_count
                .saturating_sub(self.results_area_height - 2)
                as usize;
            self.picker_state
                .select(Some((result_count as usize).saturating_sub(1)));
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

    fn reset_results_selection(&mut self) {
        self.picker_state.select(Some(0));
        self.relative_picker_state.select(Some(0));
        self.picker_view_offset = 0;
    }
}

// Styles
//  input
const DEFAULT_INPUT_FG: Color = Color::LightRed;
const DEFAULT_RESULTS_COUNT_FG: Color = Color::LightRed;

impl Television {
    /// Register an action handler that can send actions for processing if necessary.
    ///
    /// # Arguments
    /// * `tx` - An unbounded sender that can send actions.
    ///
    /// # Returns
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
    /// * `config` - Configuration settings.
    ///
    /// # Returns
    /// * `Result<()>` - An Ok result or an error.
    pub fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    /// Update the state of the component based on a received action.
    ///
    /// # Arguments
    /// * `action` - An action that may modify the state of the television.
    ///
    /// # Returns
    /// * `Result<Option<Action>>` - An action to be processed or none.
    pub async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            // handle input actions
            Action::AddInputChar(_)
            | Action::DeletePrevChar
            | Action::DeleteNextChar
            | Action::GoToInputEnd
            | Action::GoToInputStart
            | Action::GoToNextChar
            | Action::GoToPrevChar => {
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
                            self.reset_results_selection();
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
            Action::ToggleChannelSelection => {
                match self.mode {
                    Mode::Channel => {
                        self.reset_screen();
                        self.mode = Mode::Guide;
                    }
                    Mode::Guide => {
                        self.reset_screen();
                        self.mode = Mode::Channel;
                    }
                    Mode::SendToChannel => {}
                }
            }
            Action::SelectEntry => {
                if let Some(entry) = self.get_selected_entry() {
                    match self.mode {
                        Mode::Channel => self
                            .action_tx
                            .as_ref()
                            .unwrap()
                            .send(Action::SelectAndExit)?,
                        Mode::Guide => {
                            if let Ok(new_channel) =
                                TelevisionChannel::try_from(&entry)
                            {
                                self.mode = Mode::Channel;
                                self.change_channel(new_channel);
                            }
                        }
                        Mode::SendToChannel => {
                            // if let Ok(new_channel) =
                            //     UnitChannel::try_from(&entry)
                            // {
                            // }
                            self.reset_screen();
                            self.mode = Mode::Channel;
                            // TODO: spawn new channel with selected entries
                        }
                    }
                }
            }
            Action::SendToChannel => {
                self.mode = Mode::SendToChannel;
                // TODO: build new guide from current channel based on which are pipeable into
                self.guide = TelevisionChannel::TvGuide(TvGuide::new());
                self.reset_screen();
            }
            _ => {}
        }
        Ok(None)
    }

    fn reset_screen(&mut self) {
        self.reset_preview_scroll();
        self.reset_results_selection();
        self.current_pattern = EMPTY_STRING.to_string();
        self.input.reset();
        self.channel.find(EMPTY_STRING);
        self.guide.find(EMPTY_STRING);
    }

    /// Render the television on the screen.
    ///
    /// # Arguments
    /// * `f` - A frame used for rendering.
    /// * `area` - The area in which the television should be drawn.
    ///
    /// # Returns
    /// * `Result<()>` - An Ok result or an error.
    pub fn draw(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
        let dimensions = match self.mode {
            Mode::Channel => &Dimensions::default(),
            Mode::Guide | Mode::SendToChannel => &Dimensions::new(30, 70),
        };
        let layout = Layout::build(
            dimensions,
            area,
            match self.mode {
                Mode::Channel => true,
                Mode::Guide | Mode::SendToChannel => false,
            },
        );

        let help_block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default())
            .padding(Padding::uniform(1));

        let help_text = self
            .build_help_paragraph()?
            .style(Style::default().fg(Color::DarkGray).italic())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(help_block);

        f.render_widget(help_text, layout.help_bar);

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
            .border_style(get_border_style(false))
            .style(Style::default())
            .padding(Padding::right(1));

        let result_count = match self.mode {
            Mode::Channel => self.channel.result_count(),
            Mode::Guide | Mode::SendToChannel => self.guide.total_count(),
        };
        if result_count > 0 && self.picker_state.selected().is_none() {
            self.picker_state.select(Some(0));
            self.relative_picker_state.select(Some(0));
        }

        let entries = match self.mode {
            Mode::Channel => self.channel.results(
                layout.results.height.saturating_sub(2).into(),
                u32::try_from(self.picker_view_offset)?,
            ),
            Mode::Guide | Mode::SendToChannel => self.guide.results(
                layout.results.height.saturating_sub(2).into(),
                u32::try_from(self.picker_view_offset)?,
            ),
        };

        let results_list = build_results_list(results_block, &entries);

        f.render_stateful_widget(
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
            .border_style(get_border_style(false))
            .style(Style::default());

        let input_block_inner = input_block.inner(layout.input);

        f.render_widget(input_block, layout.input);

        // split input block into 3 parts: prompt symbol, input, result count
        let total_count = match self.mode {
            Mode::Channel => self.channel.total_count(),
            Mode::Guide | Mode::SendToChannel => self.guide.total_count(),
        };
        let inner_input_chunks = RatatuiLayout::default()
            .direction(Direction::Horizontal)
            .constraints([
                // prompt symbol
                Constraint::Length(2),
                // input field
                Constraint::Fill(1),
                // result count
                Constraint::Length(
                    3 * ((total_count as f32).log10().ceil() as u16 + 1) + 3,
                ),
                // spinner
                Constraint::Length(1),
            ])
            .split(input_block_inner);

        let arrow_block = Block::default();
        let arrow = Paragraph::new(Span::styled(
            "> ",
            Style::default().fg(DEFAULT_INPUT_FG).bold(),
        ))
            .block(arrow_block);
        f.render_widget(arrow, inner_input_chunks[0]);

        let interactive_input_block = Block::default();
        // keep 2 for borders and 1 for cursor
        let width = inner_input_chunks[1].width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);
        let input = Paragraph::new(self.input.value())
            .scroll((0, u16::try_from(scroll)?))
            .block(interactive_input_block)
            .style(Style::default().fg(DEFAULT_INPUT_FG).bold().italic())
            .alignment(Alignment::Left);
        f.render_widget(input, inner_input_chunks[1]);

        if match self.mode {
            Mode::Channel => self.channel.running(),
            Mode::Guide | Mode::SendToChannel => self.guide.running(),
        } {
            f.render_stateful_widget(
                self.spinner,
                inner_input_chunks[3],
                &mut self.spinner_state,
            );
        }

        let result_count_block = Block::default();
        let result_count_paragraph = Paragraph::new(Span::styled(
            format!(
                " {} / {} ",
                if result_count == 0 {
                    0
                } else {
                    self.picker_state.selected().unwrap_or(0) + 1
                },
                result_count,
            ),
            Style::default().fg(DEFAULT_RESULTS_COUNT_FG).italic(),
        ))
            .block(result_count_block)
            .alignment(Alignment::Right);
        f.render_widget(result_count_paragraph, inner_input_chunks[2]);

        // Make the cursor visible and ask tui-rs to put it at the
        // specified coordinates after rendering
        f.set_cursor_position((
            // Put cursor past the end of the input text
            inner_input_chunks[1].x
                + u16::try_from(
                self.input.visual_cursor().max(scroll) - scroll,
            )?,
            // Move one line down, from the border to the input line
            inner_input_chunks[1].y,
        ));

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
                        {
                            let mut icon_str = String::from(" ");
                            icon_str.push(icon.icon);
                            icon_str.push(' ');
                            icon_str
                        },
                        Style::default().fg(Color::from_str(icon.color)?),
                    ));
                }
                preview_title_spans.push(Span::styled(
                    shrink_with_ellipsis(
                        &preview.title,
                        preview_title_area.width.saturating_sub(4) as usize,
                    ),
                    Style::default().fg(DEFAULT_PREVIEW_TITLE_FG).bold(),
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
                f.render_widget(preview_title, preview_title_area);
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
                    .border_style(get_border_style(false))
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
                f.render_widget(preview_outer_block, preview_area);

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
                        .map(|l| u16::try_from(l).unwrap_or(0)),
                );
                f.render_widget(preview_block, inner);
                //}
            }
        }
        Ok(())
    }
}
