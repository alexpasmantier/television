use crate::config::KeyBindings;
use crate::input::convert_action_to_input_request;
use crate::picker::Picker;
use crate::{action::Action, config::Config};
use crate::{cable::load_cable_channels, keymap::Keymap};
use color_eyre::Result;
use copypasta::{ClipboardContext, ClipboardProvider};
use ratatui::{layout::Rect, style::Color, Frame};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use television_channels::channels::{
    remote_control::{load_builtin_channels, RemoteControl},
    OnAir, TelevisionChannel, UnitChannel,
};
use television_channels::entry::{Entry, ENTRY_PLACEHOLDER};
use television_previewers::previewers::Previewer;
use television_screen::cache::RenderedPreviewCache;
use television_screen::help::draw_help_bar;
use television_screen::input::draw_input_box;
use television_screen::keybindings::{
    build_keybindings_table, DisplayableAction, DisplayableKeybindings,
};
use television_screen::layout::{Dimensions, InputPosition, Layout};
use television_screen::mode::Mode;
use television_screen::preview::{
    draw_preview_content_block, draw_preview_title_block,
};
use television_screen::remote_control::draw_remote_control;
use television_screen::results::draw_results_list;
use television_screen::spinner::{Spinner, SpinnerState};
use television_utils::metadata::{AppMetadata, BuildMetadata};
use television_utils::strings::EMPTY_STRING;
use tokio::sync::mpsc::UnboundedSender;

pub struct Television {
    action_tx: Option<UnboundedSender<Action>>,
    pub config: Config,
    pub keymap: Keymap,
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
    pub icon_color_cache: HashMap<String, Color>,
    pub rendered_preview_cache: Arc<Mutex<RenderedPreviewCache<'static>>>,
    pub(crate) spinner: Spinner,
    pub(crate) spinner_state: SpinnerState,
    pub app_metadata: AppMetadata,
}

impl Television {
    #[must_use]
    pub fn new(mut channel: TelevisionChannel, config: Config) -> Self {
        let results_picker = match config.ui.input_bar_position {
            InputPosition::Bottom => Picker::default().inverted(),
            InputPosition::Top => Picker::default(),
        };
        let previewer = Previewer::new(Some(config.previewers.clone().into()));
        let keymap = Keymap::from(&config.keybindings);
        let builtin_channels = load_builtin_channels();
        let cable_channels = load_cable_channels().unwrap_or_default();
        let app_metadata = AppMetadata::new(
            env!("CARGO_PKG_VERSION").to_string(),
            BuildMetadata::new(
                env!("VERGEN_RUSTC_SEMVER").to_string(),
                env!("VERGEN_BUILD_DATE").to_string(),
                env!("VERGEN_CARGO_TARGET_TRIPLE").to_string(),
            ),
            std::env::current_dir()
                .expect("Could not get current directory")
                .to_string_lossy()
                .to_string(),
        );

        channel.find(EMPTY_STRING);
        let spinner = Spinner::default();
        Self {
            action_tx: None,
            config,
            keymap,
            channel,
            remote_control: TelevisionChannel::RemoteControl(
                RemoteControl::new(builtin_channels, Some(cable_channels)),
            ),
            mode: Mode::Channel,
            current_pattern: EMPTY_STRING.to_string(),
            results_picker,
            rc_picker: Picker::default(),
            results_area_height: 0,
            previewer,
            preview_scroll: None,
            preview_pane_height: 0,
            current_preview_total_lines: 0,
            icon_color_cache: HashMap::new(),
            rendered_preview_cache: Arc::new(Mutex::new(
                RenderedPreviewCache::default(),
            )),
            spinner,
            spinner_state: SpinnerState::from(&spinner),
            app_metadata,
        }
    }

    pub fn init_remote_control(&mut self) {
        let builtin_channels = load_builtin_channels();
        let cable_channels = load_cable_channels().unwrap_or_default();
        self.remote_control = TelevisionChannel::RemoteControl(
            RemoteControl::new(builtin_channels, Some(cable_channels)),
        );
    }

    pub fn current_channel(&self) -> UnitChannel {
        UnitChannel::from(&self.channel)
    }

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

    pub fn select_prev_entry(&mut self, step: u32) {
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
            step,
            result_count as usize,
            self.results_area_height as usize,
        );
    }

    pub fn select_next_entry(&mut self, step: u32) {
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
            step,
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
                self.rc_picker.reset_selection();
            }
        }
    }

    fn reset_picker_input(&mut self) {
        match self.mode {
            Mode::Channel => self.results_picker.reset_input(),
            Mode::RemoteControl | Mode::SendToChannel => {
                self.rc_picker.reset_input();
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
        self.action_tx = Some(tx);
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
                input
                    .handle(convert_action_to_input_request(&action).unwrap());
                match action {
                    Action::AddInputChar(_)
                    | Action::DeletePrevChar
                    | Action::DeleteNextChar => {
                        let new_pattern = input.value().to_string();
                        if new_pattern != self.current_pattern {
                            self.current_pattern.clone_from(&new_pattern);
                            self.find(&new_pattern);
                            self.reset_picker_selection();
                            self.reset_preview_scroll();
                        }
                    }
                    _ => {}
                }
            }
            Action::SelectNextEntry => {
                self.reset_preview_scroll();
                self.select_next_entry(1);
            }
            Action::SelectPrevEntry => {
                self.reset_preview_scroll();
                self.select_prev_entry(1);
            }
            Action::SelectNextPage => {
                self.reset_preview_scroll();
                self.select_next_entry(self.results_area_height);
            }
            Action::SelectPrevPage => {
                self.reset_preview_scroll();
                self.select_prev_entry(self.results_area_height);
            }
            Action::ScrollPreviewDown => self.scroll_preview_down(1),
            Action::ScrollPreviewUp => self.scroll_preview_up(1),
            Action::ScrollPreviewHalfPageDown => self.scroll_preview_down(20),
            Action::ScrollPreviewHalfPageUp => self.scroll_preview_up(20),
            Action::ToggleRemoteControl => match self.mode {
                Mode::Channel => {
                    self.mode = Mode::RemoteControl;
                    self.init_remote_control();
                }
                Mode::RemoteControl => {
                    // this resets the RC picker
                    self.reset_picker_input();
                    self.init_remote_control();
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
                            let new_channel = self
                                .remote_control
                                .zap(entry.name.as_str())?;
                            // this resets the RC picker
                            self.reset_picker_selection();
                            self.reset_picker_input();
                            self.remote_control.find(EMPTY_STRING);
                            self.mode = Mode::Channel;
                            self.change_channel(new_channel);
                        }
                        Mode::SendToChannel => {
                            let new_channel = self.channel.transition_to(
                                entry.name.as_str().try_into().unwrap(),
                            );
                            self.reset_picker_selection();
                            self.reset_picker_input();
                            self.remote_control.find(EMPTY_STRING);
                            self.mode = Mode::Channel;
                            self.change_channel(new_channel);
                        }
                    }
                }
            }
            Action::CopyEntryToClipboard => {
                if self.mode == Mode::Channel {
                    if let Some(entry) = self.get_selected_entry(None) {
                        let mut ctx = ClipboardContext::new().unwrap();
                        ctx.set_contents(entry.name).unwrap();
                    }
                }
            }
            Action::ToggleSendToChannel => match self.mode {
                Mode::Channel | Mode::RemoteControl => {
                    self.mode = Mode::SendToChannel;
                    self.remote_control = TelevisionChannel::RemoteControl(
                        RemoteControl::with_transitions_from(&self.channel),
                    );
                }
                Mode::SendToChannel => {
                    self.reset_picker_input();
                    self.remote_control.find(EMPTY_STRING);
                    self.reset_picker_selection();
                    self.mode = Mode::Channel;
                }
            },
            Action::ToggleHelp => {
                self.config.ui.show_help_bar = !self.config.ui.show_help_bar;
            }
            _ => {}
        }
        Ok(None)
    }

    /// Render the television on the screen.
    ///
    /// # Arguments
    /// * `f` - A frame used for rendering.
    /// * `area` - The area in which the television should be drawn.
    ///
    /// # Returns
    /// * `Result<()>` - An Ok result or an error.
    pub fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let layout = Layout::build(
            &Dimensions::from(self.config.ui.ui_scale),
            area,
            !matches!(self.mode, Mode::Channel),
            self.config.ui.show_help_bar,
            self.config.ui.input_bar_position,
        );

        // help bar (metadata, keymaps, logo)
        draw_help_bar(
            f,
            &layout.help_bar,
            self.current_channel(),
            build_keybindings_table(
                &self.config.keybindings.to_displayable(),
                self.mode,
            ),
            self.mode,
            &self.app_metadata,
        );

        self.results_area_height =
            u32::from(layout.results.height.saturating_sub(2)); // 2 for the borders
        self.preview_pane_height = layout.preview_window.height;

        // results list
        let result_count = self.channel.result_count();
        if result_count > 0 && self.results_picker.selected().is_none() {
            self.results_picker.select(Some(0));
            self.results_picker.relative_select(Some(0));
        }
        let entries = self.channel.results(
            self.results_area_height,
            u32::try_from(self.results_picker.offset())?,
        );
        draw_results_list(
            f,
            layout.results,
            &entries,
            &mut self.results_picker.relative_state,
            self.config.ui.input_bar_position,
            self.config.ui.use_nerd_font_icons,
            &mut self.icon_color_cache,
        )?;

        // input box
        draw_input_box(
            f,
            layout.input,
            result_count,
            self.channel.total_count(),
            &mut self.results_picker.input,
            &mut self.results_picker.state,
            self.channel.running(),
            &self.spinner,
            &mut self.spinner_state,
        )?;

        let selected_entry = self
            .get_selected_entry(Some(Mode::Channel))
            .unwrap_or(ENTRY_PLACEHOLDER);
        let preview = self.previewer.preview(&selected_entry);

        // preview title
        self.current_preview_total_lines = preview.total_lines();
        draw_preview_title_block(
            f,
            layout.preview_title,
            &preview,
            self.config.ui.use_nerd_font_icons,
        )?;

        // preview content
        // initialize preview scroll
        self.maybe_init_preview_scroll(
            selected_entry
                .line_number
                .map(|l| u16::try_from(l).unwrap_or(0)),
            layout.preview_window.height,
        );
        draw_preview_content_block(
            f,
            layout.preview_window,
            &selected_entry,
            &preview,
            &self.rendered_preview_cache,
            self.preview_scroll.unwrap_or(0),
        );

        // remote control
        if matches!(self.mode, Mode::RemoteControl | Mode::SendToChannel) {
            // NOTE: this should be done in the `update` method
            let result_count = self.remote_control.result_count();
            if result_count > 0 && self.rc_picker.selected().is_none() {
                self.rc_picker.select(Some(0));
                self.rc_picker.relative_select(Some(0));
            }
            let entries = self.remote_control.results(
                area.height.saturating_sub(2).into(),
                u32::try_from(self.rc_picker.offset())?,
            );
            draw_remote_control(
                f,
                layout.remote_control.unwrap(),
                &entries,
                self.config.ui.use_nerd_font_icons,
                &mut self.rc_picker.state,
                &mut self.rc_picker.input,
                &mut self.icon_color_cache,
            )?;
        }
        Ok(())
    }

    pub fn maybe_init_preview_scroll(
        &mut self,
        target_line: Option<u16>,
        height: u16,
    ) {
        if self.preview_scroll.is_none() && !self.channel.running() {
            self.preview_scroll =
                Some(target_line.unwrap_or(0).saturating_sub(height / 3));
        }
    }
}

impl KeyBindings {
    pub fn to_displayable(&self) -> HashMap<Mode, DisplayableKeybindings> {
        // channel mode keybindings
        let channel_bindings: HashMap<DisplayableAction, Vec<String>> =
            HashMap::from_iter(vec![
                (
                    DisplayableAction::ResultsNavigation,
                    serialized_keys_for_actions(
                        self,
                        &[
                            Action::SelectPrevEntry,
                            Action::SelectNextEntry,
                            Action::SelectPrevPage,
                            Action::SelectNextPage,
                        ],
                    ),
                ),
                (
                    DisplayableAction::PreviewNavigation,
                    serialized_keys_for_actions(
                        self,
                        &[
                            Action::ScrollPreviewHalfPageUp,
                            Action::ScrollPreviewHalfPageDown,
                        ],
                    ),
                ),
                (
                    DisplayableAction::SelectEntry,
                    serialized_keys_for_actions(self, &[Action::SelectEntry]),
                ),
                (
                    DisplayableAction::CopyEntryToClipboard,
                    serialized_keys_for_actions(
                        self,
                        &[Action::CopyEntryToClipboard],
                    ),
                ),
                (
                    DisplayableAction::SendToChannel,
                    serialized_keys_for_actions(
                        self,
                        &[Action::ToggleSendToChannel],
                    ),
                ),
                (
                    DisplayableAction::ToggleRemoteControl,
                    serialized_keys_for_actions(
                        self,
                        &[Action::ToggleRemoteControl],
                    ),
                ),
                (
                    DisplayableAction::Quit,
                    serialized_keys_for_actions(self, &[Action::Quit]),
                ),
            ]);

        // remote control mode keybindings
        let remote_control_bindings: HashMap<DisplayableAction, Vec<String>> =
            HashMap::from_iter(vec![
                (
                    DisplayableAction::ResultsNavigation,
                    serialized_keys_for_actions(
                        self,
                        &[Action::SelectPrevEntry, Action::SelectNextEntry],
                    ),
                ),
                (
                    DisplayableAction::SelectEntry,
                    serialized_keys_for_actions(self, &[Action::SelectEntry]),
                ),
                (
                    DisplayableAction::ToggleRemoteControl,
                    serialized_keys_for_actions(
                        self,
                        &[Action::ToggleRemoteControl],
                    ),
                ),
            ]);

        // send to channel mode keybindings
        let send_to_channel_bindings: HashMap<DisplayableAction, Vec<String>> =
            HashMap::from_iter(vec![
                (
                    DisplayableAction::ResultsNavigation,
                    serialized_keys_for_actions(
                        self,
                        &[Action::SelectPrevEntry, Action::SelectNextEntry],
                    ),
                ),
                (
                    DisplayableAction::SelectEntry,
                    serialized_keys_for_actions(self, &[Action::SelectEntry]),
                ),
                (
                    DisplayableAction::Cancel,
                    serialized_keys_for_actions(
                        self,
                        &[Action::ToggleSendToChannel],
                    ),
                ),
            ]);

        HashMap::from_iter(vec![
            (Mode::Channel, DisplayableKeybindings::new(channel_bindings)),
            (
                Mode::RemoteControl,
                DisplayableKeybindings::new(remote_control_bindings),
            ),
            (
                Mode::SendToChannel,
                DisplayableKeybindings::new(send_to_channel_bindings),
            ),
        ])
    }
}

fn serialized_keys_for_actions(
    keybindings: &KeyBindings,
    actions: &[Action],
) -> Vec<String> {
    actions
        .iter()
        .map(|a| {
            keybindings
                .get(&Mode::Channel)
                .unwrap()
                .get(a)
                .unwrap()
                .clone()
                .to_string()
        })
        .collect()
}
