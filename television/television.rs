use crate::action::Action;
use crate::channels::entry::{Entry, PreviewType, ENTRY_PLACEHOLDER};
use crate::channels::{
    remote_control::{load_builtin_channels, RemoteControl},
    OnAir, TelevisionChannel, UnitChannel,
};
use crate::config::{Config, KeyBindings, Theme};
use crate::input::convert_action_to_input_request;
use crate::picker::Picker;
use crate::preview::Previewer;
use crate::screen::cache::RenderedPreviewCache;
use crate::screen::colors::Colorscheme;
use crate::screen::help::draw_help_bar;
use crate::screen::input::draw_input_box;
use crate::screen::keybindings::{
    build_keybindings_table, DisplayableAction, DisplayableKeybindings,
};
use crate::screen::layout::{Dimensions, InputPosition, Layout};
use crate::screen::mode::Mode;
use crate::screen::preview::draw_preview_content_block;
use crate::screen::remote_control::draw_remote_control;
use crate::screen::results::draw_results_list;
use crate::screen::spinner::{Spinner, SpinnerState};
use crate::utils::metadata::AppMetadata;
use crate::utils::strings::EMPTY_STRING;
use crate::{cable::load_cable_channels, keymap::Keymap};
use anyhow::Result;
use copypasta::{ClipboardContext, ClipboardProvider};
use ratatui::{layout::Rect, style::Color, Frame};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;

pub struct Television {
    action_tx: Option<UnboundedSender<Action>>,
    pub config: Config,
    pub keymap: Keymap,
    pub(crate) channel: TelevisionChannel,
    pub(crate) remote_control: TelevisionChannel,
    pub mode: Mode,
    pub current_pattern: String,
    pub(crate) results_picker: Picker,
    pub(crate) rc_picker: Picker,
    results_area_height: u32,
    pub previewer: Previewer,
    pub preview_scroll: Option<u16>,
    pub preview_pane_height: u16,
    current_preview_total_lines: u16,
    pub icon_color_cache: FxHashMap<String, Color>,
    pub rendered_preview_cache: Arc<Mutex<RenderedPreviewCache<'static>>>,
    pub(crate) spinner: Spinner,
    pub(crate) spinner_state: SpinnerState,
    pub app_metadata: AppMetadata,
    pub colorscheme: Colorscheme,
}

impl Television {
    #[must_use]
    pub fn new(
        mut channel: TelevisionChannel,
        config: Config,
        input: Option<String>,
    ) -> Self {
        let mut results_picker = Picker::new(input.clone());
        if config.ui.input_bar_position == InputPosition::Bottom {
            results_picker = results_picker.inverted();
        }
        let previewer = Previewer::new(Some(config.previewers.clone().into()));
        let keymap = Keymap::from(&config.keybindings);
        let cable_channels = load_cable_channels().unwrap_or_default();
        let builtin_channels = load_builtin_channels(Some(
            &cable_channels.keys().collect::<Vec<_>>(),
        ));

        let app_metadata = AppMetadata::new(
            env!("CARGO_PKG_VERSION").to_string(),
            std::env::current_dir()
                .expect("Could not get current directory")
                .to_string_lossy()
                .to_string(),
        );
        let colorscheme = (&Theme::from_name(&config.ui.theme)).into();

        channel.find(&input.unwrap_or(EMPTY_STRING.to_string()));
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
            icon_color_cache: FxHashMap::default(),
            rendered_preview_cache: Arc::new(Mutex::new(
                RenderedPreviewCache::default(),
            )),
            spinner,
            spinner_state: SpinnerState::from(&spinner),
            app_metadata,
            colorscheme,
        }
    }

    pub fn init_remote_control(&mut self) {
        let cable_channels = load_cable_channels().unwrap_or_default();
        let builtin_channels = load_builtin_channels(Some(
            &cable_channels.keys().collect::<Vec<_>>(),
        ));
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
            Mode::Channel => {
                if let Some(i) = self.results_picker.selected() {
                    return self.channel.get_result(i.try_into().unwrap());
                }
                None
            }
            Mode::RemoteControl | Mode::SendToChannel => {
                if let Some(i) = self.rc_picker.selected() {
                    return self
                        .remote_control
                        .get_result(i.try_into().unwrap());
                }
                None
            }
        }
    }

    #[must_use]
    pub fn get_selected_entries(
        &mut self,
        mode: Option<Mode>,
    ) -> Option<FxHashSet<Entry>> {
        if self.channel.selected_entries().is_empty()
            || matches!(mode, Some(Mode::RemoteControl))
        {
            return self.get_selected_entry(mode).map(|e| {
                let mut set = HashSet::with_hasher(FxBuildHasher);
                set.insert(e);
                set
            });
        }
        Some(self.channel.selected_entries().clone())
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

    fn should_render(&self, action: &Action) -> bool {
        matches!(
            action,
            Action::AddInputChar(_)
                | Action::DeletePrevChar
                | Action::DeletePrevWord
                | Action::DeleteNextChar
                | Action::GoToPrevChar
                | Action::GoToNextChar
                | Action::GoToInputStart
                | Action::GoToInputEnd
                | Action::ToggleSelectionDown
                | Action::ToggleSelectionUp
                | Action::ConfirmSelection
                | Action::SelectNextEntry
                | Action::SelectPrevEntry
                | Action::SelectNextPage
                | Action::SelectPrevPage
                | Action::ScrollPreviewDown
                | Action::ScrollPreviewUp
                | Action::ScrollPreviewHalfPageDown
                | Action::ScrollPreviewHalfPageUp
                | Action::ToggleRemoteControl
                | Action::ToggleSendToChannel
                | Action::ToggleHelp
                | Action::TogglePreview
                | Action::CopyEntryToClipboard
        ) || self.channel.running()
    }

    #[allow(clippy::unused_async)]
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
            | Action::DeletePrevWord
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
                    | Action::DeletePrevWord
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
            Action::ToggleSelectionDown | Action::ToggleSelectionUp => {
                if matches!(self.mode, Mode::Channel) {
                    if let Some(entry) = self.get_selected_entry(None) {
                        self.channel.toggle_selection(&entry);
                        if matches!(action, Action::ToggleSelectionDown) {
                            self.select_next_entry(1);
                        } else {
                            self.select_prev_entry(1);
                        }
                    }
                }
            }
            Action::ConfirmSelection => {
                match self.mode {
                    Mode::Channel => {
                        self.action_tx
                            .as_ref()
                            .unwrap()
                            .send(Action::SelectAndExit)?;
                    }
                    Mode::RemoteControl => {
                        if let Some(entry) =
                            self.get_selected_entry(Some(Mode::RemoteControl))
                        {
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
                    }
                    Mode::SendToChannel => {
                        if let Some(entry) =
                            self.get_selected_entry(Some(Mode::RemoteControl))
                        {
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
                    if let Some(entries) = self.get_selected_entries(None) {
                        let mut ctx = ClipboardContext::new().unwrap();
                        ctx.set_contents(
                            entries
                                .iter()
                                .map(|e| e.name.clone())
                                .collect::<Vec<_>>()
                                .join(" "),
                        )
                        .unwrap();
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
            Action::TogglePreview => {
                self.config.ui.show_preview_panel =
                    !self.config.ui.show_preview_panel;
            }
            _ => {}
        }

        Ok(if self.should_render(&action) {
            Some(Action::Render)
        } else {
            None
        })
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
        let selected_entry = self
            .get_selected_entry(Some(Mode::Channel))
            .unwrap_or(ENTRY_PLACEHOLDER);

        let layout = Layout::build(
            &Dimensions::from(self.config.ui.ui_scale),
            area,
            !matches!(self.mode, Mode::Channel),
            self.config.ui.show_help_bar,
            self.config.ui.show_preview_panel
                && !matches!(selected_entry.preview_type, PreviewType::None),
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
                &self.colorscheme,
            ),
            self.mode,
            &self.app_metadata,
            &self.colorscheme,
        );

        self.results_area_height =
            u32::from(layout.results.height.saturating_sub(2)); // 2 for the borders
        self.preview_pane_height = match layout.preview_window {
            Some(preview) => preview.height,
            None => 0,
        };

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
            self.channel.selected_entries(),
            &mut self.results_picker.relative_state,
            self.config.ui.input_bar_position,
            self.config.ui.use_nerd_font_icons,
            &mut self.icon_color_cache,
            &self.colorscheme,
            &self
                .config
                .keybindings
                .get(&self.mode)
                .unwrap()
                .get(&Action::ToggleHelp)
                // just display the first keybinding
                .unwrap()
                .to_string(),
            &self
                .config
                .keybindings
                .get(&self.mode)
                .unwrap()
                .get(&Action::TogglePreview)
                // just display the first keybinding
                .unwrap()
                .to_string(),
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
            &self.colorscheme,
        )?;

        if self.config.ui.show_preview_panel
            && !matches!(selected_entry.preview_type, PreviewType::None)
        {
            // preview content
            let maybe_preview = self.previewer.preview(&selected_entry);

            let _ = self.previewer.preview(&selected_entry);

            if let Some(preview) = &maybe_preview {
                self.current_preview_total_lines = preview.total_lines;
                // initialize preview scroll
                self.maybe_init_preview_scroll(
                    selected_entry
                        .line_number
                        .map(|l| u16::try_from(l).unwrap_or(0)),
                    layout.preview_window.unwrap().height,
                );
            }

            draw_preview_content_block(
                f,
                layout.preview_window.unwrap(),
                &selected_entry,
                &maybe_preview,
                &self.rendered_preview_cache,
                self.preview_scroll.unwrap_or(0),
                self.config.ui.use_nerd_font_icons,
                &self.colorscheme,
            )?;
        }

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
                &self.mode,
                &self.colorscheme,
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
    pub fn to_displayable(&self) -> FxHashMap<Mode, DisplayableKeybindings> {
        // channel mode keybindings
        let channel_bindings: FxHashMap<DisplayableAction, Vec<String>> =
            FxHashMap::from_iter(vec![
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
                    serialized_keys_for_actions(
                        self,
                        &[Action::ConfirmSelection],
                    ),
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
                    DisplayableAction::ToggleHelpBar,
                    serialized_keys_for_actions(self, &[Action::ToggleHelp]),
                ),
            ]);

        // remote control mode keybindings
        let remote_control_bindings: FxHashMap<
            DisplayableAction,
            Vec<String>,
        > = FxHashMap::from_iter(vec![
            (
                DisplayableAction::ResultsNavigation,
                serialized_keys_for_actions(
                    self,
                    &[Action::SelectPrevEntry, Action::SelectNextEntry],
                ),
            ),
            (
                DisplayableAction::SelectEntry,
                serialized_keys_for_actions(self, &[Action::ConfirmSelection]),
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
        let send_to_channel_bindings: FxHashMap<
            DisplayableAction,
            Vec<String>,
        > = FxHashMap::from_iter(vec![
            (
                DisplayableAction::ResultsNavigation,
                serialized_keys_for_actions(
                    self,
                    &[Action::SelectPrevEntry, Action::SelectNextEntry],
                ),
            ),
            (
                DisplayableAction::SelectEntry,
                serialized_keys_for_actions(self, &[Action::ConfirmSelection]),
            ),
            (
                DisplayableAction::Cancel,
                serialized_keys_for_actions(
                    self,
                    &[Action::ToggleSendToChannel],
                ),
            ),
        ]);

        FxHashMap::from_iter(vec![
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
