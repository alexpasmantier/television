use crate::channels::remote_control::RemoteControl;
use crate::channels::OnAir;
use crate::channels::UnitChannel;
use crate::picker::Picker;
use crate::ui::layout::{Dimensions, Layout};
use crate::utils::strings::EMPTY_STRING;
use crate::{action::Action, config::Config};
use crate::{
    channels::TelevisionChannel, ui::input::actions::InputActionHandler,
};
use crate::{
    entry::{Entry, ENTRY_PLACEHOLDER},
    ui::spinner::Spinner,
};
use crate::{previewers::Previewer, ui::spinner::SpinnerState};
use color_eyre::Result;
use futures::executor::block_on;
use ratatui::{layout::Rect, style::Color, widgets::Paragraph, Frame};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::Display;
use tokio::sync::mpsc::UnboundedSender;

#[derive(
    PartialEq, Copy, Clone, Hash, Eq, Debug, Serialize, Deserialize, Display,
)]
pub enum Mode {
    Channel,
    RemoteControl,
    SendToChannel,
}

pub struct Television {
    action_tx: Option<UnboundedSender<Action>>,
    pub config: Config,
    pub(crate) channel: TelevisionChannel,
    pub(crate) remote_control: TelevisionChannel,
    pub mode: Mode,
    current_pattern: String,
    pub(crate) results_picker: Picker,
    pub(crate) rc_picker: Picker,
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
    pub(crate) spinner: Spinner,
    pub(crate) spinner_state: SpinnerState,
}

impl Television {
    #[must_use]
    pub fn new(mut channel: TelevisionChannel) -> Self {
        channel.find(EMPTY_STRING);
        let spinner = Spinner::default();
        Self {
            action_tx: None,
            config: Config::default(),
            channel,
            remote_control: TelevisionChannel::RemoteControl(
                RemoteControl::new(),
            ),
            mode: Mode::Channel,
            current_pattern: EMPTY_STRING.to_string(),
            results_picker: Picker::default(),
            rc_picker: Picker::default().inverted(),
            results_area_height: 0,
            previewer: Previewer::new(),
            preview_scroll: None,
            preview_pane_height: 0,
            current_preview_total_lines: 0,
            meta_paragraph_cache: HashMap::new(),
            spinner,
            spinner_state: SpinnerState::from(&spinner),
        }
    }

    pub fn current_channel(&self) -> UnitChannel {
        UnitChannel::from(&self.channel)
    }

    /// FIXME: this needs rework
    pub fn change_channel(&mut self, channel: TelevisionChannel) {
        self.reset_preview_scroll();
        self.reset_picker_selection();
        self.reset_picker_input();
        self.current_pattern = EMPTY_STRING.to_string();
        self.channel.shutdown();
        self.channel = channel;
    }

    fn find(&mut self, pattern: &str) {
        match self.mode {
            Mode::Channel => {
                self.channel.find(pattern);
            }
            Mode::RemoteControl | Mode::SendToChannel => {
                self.remote_control.find(pattern);
            }
        }
    }

    #[must_use]
    pub fn get_selected_entry(&mut self, mode: Option<Mode>) -> Option<Entry> {
        match mode.unwrap_or(self.mode) {
            Mode::Channel => self.results_picker.selected().and_then(|i| {
                self.channel.get_result(u32::try_from(i).unwrap())
            }),
            Mode::RemoteControl | Mode::SendToChannel => {
                self.rc_picker.selected().and_then(|i| {
                    self.remote_control.get_result(u32::try_from(i).unwrap())
                })
            }
        }
    }

    pub fn select_prev_entry(&mut self) {
        let (result_count, picker) = match self.mode {
            Mode::Channel => {
                (self.channel.result_count(), &mut self.results_picker)
            }
            Mode::RemoteControl | Mode::SendToChannel => {
                (self.remote_control.total_count(), &mut self.rc_picker)
            }
        };
        if result_count == 0 {
            return;
        }
        picker.select_prev(
            result_count as usize,
            self.results_area_height as usize,
        );
    }

    pub fn select_next_entry(&mut self) {
        let (result_count, picker) = match self.mode {
            Mode::Channel => {
                (self.channel.result_count(), &mut self.results_picker)
            }
            Mode::RemoteControl | Mode::SendToChannel => {
                (self.remote_control.total_count(), &mut self.rc_picker)
            }
        };
        if result_count == 0 {
            return;
        }
        picker.select_next(
            result_count as usize,
            self.results_area_height as usize,
        );
    }

    fn reset_preview_scroll(&mut self) {
        self.preview_scroll = None;
    }

    fn reset_picker_selection(&mut self) {
        match self.mode {
            Mode::Channel => self.results_picker.reset_selection(),
            Mode::RemoteControl | Mode::SendToChannel => {
                self.rc_picker.reset_selection()
            }
        }
    }

    fn reset_picker_input(&mut self) {
        match self.mode {
            Mode::Channel => self.results_picker.reset_input(),
            Mode::RemoteControl | Mode::SendToChannel => {
                self.rc_picker.reset_input()
            }
        }
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
}

// Styles
//  input
pub(crate) const DEFAULT_INPUT_FG: Color = Color::LightRed;
pub(crate) const DEFAULT_RESULTS_COUNT_FG: Color = Color::LightRed;

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
                let input = match self.mode {
                    Mode::Channel => &mut self.results_picker.input,
                    Mode::RemoteControl | Mode::SendToChannel => {
                        &mut self.rc_picker.input
                    }
                };
                input.handle_action(&action);
                match action {
                    Action::AddInputChar(_)
                    | Action::DeletePrevChar
                    | Action::DeleteNextChar => {
                        let new_pattern = input.value().to_string();
                        if new_pattern != self.current_pattern {
                            self.current_pattern.clone_from(&new_pattern);
                            self.find(&new_pattern);
                            self.reset_preview_scroll();
                            self.reset_picker_selection();
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
            Action::ToggleRemoteControl => match self.mode {
                Mode::Channel => {
                    self.mode = Mode::RemoteControl;
                }
                Mode::RemoteControl => {
                    // this resets the RC picker
                    self.reset_picker_input();
                    self.remote_control.find(EMPTY_STRING);
                    self.reset_picker_selection();
                    self.mode = Mode::Channel;
                }
                Mode::SendToChannel => {}
            },
            Action::SelectEntry => {
                if let Some(entry) = self.get_selected_entry(None) {
                    match self.mode {
                        Mode::Channel => self
                            .action_tx
                            .as_ref()
                            .unwrap()
                            .send(Action::SelectAndExit)?,
                        Mode::RemoteControl => {
                            if let Ok(new_channel) =
                                TelevisionChannel::try_from(&entry)
                            {
                                // this resets the RC picker
                                self.reset_picker_selection();
                                self.reset_picker_input();
                                self.remote_control.find(EMPTY_STRING);
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
                self.remote_control =
                    TelevisionChannel::RemoteControl(RemoteControl::new());
                self.reset_screen();
            }
            _ => {}
        }
        Ok(None)
    }

    fn reset_screen(&mut self) {
        self.reset_preview_scroll();
        self.reset_picker_selection();
        self.reset_picker_input();
        self.current_pattern = EMPTY_STRING.to_string();
        self.channel.find(EMPTY_STRING);
        self.remote_control.find(EMPTY_STRING);
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
        let layout = Layout::build(
            &Dimensions::default(),
            area,
            !matches!(self.mode, Mode::Channel),
        );

        // help bar (metadata, keymaps, logo)
        self.draw_help_bar(f, &layout)?;

        self.results_area_height = u32::from(layout.results.height);
        self.preview_pane_height = layout.preview_window.height;

        // top left block: results
        self.draw_results_list(f, &layout)?;

        // bottom left block: input
        self.draw_input_box(f, &layout)?;

        let selected_entry = self
            .get_selected_entry(Some(Mode::Channel))
            .unwrap_or(ENTRY_PLACEHOLDER);
        let preview = block_on(self.previewer.preview(&selected_entry));

        // top right block: preview title
        self.current_preview_total_lines = preview.total_lines();
        self.draw_preview_title_block(f, &layout, &selected_entry, &preview)?;

        // bottom right block: preview content
        self.draw_preview_content_block(
            f,
            &layout,
            &selected_entry,
            &preview,
        )?;

        // remote control
        if matches!(self.mode, Mode::RemoteControl) {
            self.draw_remote_control(f, &layout.remote_control.unwrap())?;
        }
        Ok(())
    }
}
