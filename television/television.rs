use crate::{
    action::Action,
    cable::Cable,
    channels::{
        action_picker::{ActionEntry, ActionPicker},
        channel::ChannelKind as CableChannel,
        entry::Entry,
        prototypes::{ChannelPrototype, CommandSpec, Template},
        remote_control::{CableEntry, RemoteControl},
    },
    config::{
        Theme,
        layers::{ConfigLayers, MergedConfig},
    },
    draw::{ChannelState, Ctx, TvState},
    errors::os_error_exit,
    frecency::FrecencyHandle,
    input::convert_action_to_input_request,
    picker::{Movement, Picker},
    previewer::{
        Config as PreviewerConfig, Preview, Previewer,
        Request as PreviewRequest, Ticket, state::PreviewState,
    },
    render::UiState,
    screen::{
        colors::Colorscheme,
        layout::{InputPosition, Orientation},
    },
    utils::{
        clipboard::CLIPBOARD,
        metadata::AppMetadata,
        strings::{EMPTY_STRING, SPACE},
    },
};
use anyhow::Result;
use ratatui::layout::Rect;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, sync::Arc};
use tokio::sync::mpsc::{
    UnboundedReceiver, UnboundedSender, unbounded_channel,
};
use tracing::{debug, error};

#[derive(PartialEq, Copy, Clone, Hash, Eq, Debug, Serialize, Deserialize)]
pub enum Mode {
    Channel,
    RemoteControl,
    ActionPicker,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Channel => write!(f, "Channel"),
            Mode::RemoteControl => write!(f, "Remote Control"),
            Mode::ActionPicker => write!(f, "Action Picker"),
        }
    }
}

/// State for the missing requirements popup dialog.
///
/// This popup is shown when a user attempts to switch to a channel
/// that has unmet binary requirements.
#[derive(Debug, Clone)]
pub struct MissingRequirementsPopup {
    pub channel_name: String,
    pub missing_requirements: Vec<String>,
}

#[derive(PartialEq, Copy, Clone, Hash, Eq, Debug, Serialize, Deserialize)]
pub enum MatchingMode {
    Substring,
    Fuzzy,
}

pub struct Television {
    action_tx: UnboundedSender<Action>,
    pub layered_config: ConfigLayers,
    pub merged_config: MergedConfig,
    pub channel: CableChannel,
    pub remote_control: Option<RemoteControl>,
    pub action_picker: Option<ActionPicker>,
    pub mode: Mode,
    pub currently_selected: Option<Entry>,
    pub current_pattern: String,
    pub matching_mode: MatchingMode,
    pub results_picker: Picker<Entry>,
    pub rc_picker: Picker<CableEntry>,
    pub ap_picker: Picker<ActionEntry>,
    pub preview_state: PreviewState,
    pub preview_handles:
        Option<(UnboundedSender<PreviewRequest>, UnboundedReceiver<Preview>)>,
    pub app_metadata: Arc<AppMetadata>,
    pub colorscheme: Arc<Colorscheme>,
    pub ticks: u64,
    pub ui_state: UiState,
    /// Frecency manager for ranking previously-selected entries
    frecency: FrecencyHandle,
    /// Popup shown when attempting to switch to a channel with missing requirements
    pub missing_requirements_popup: Option<MissingRequirementsPopup>,
}

impl Television {
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::fn_params_excessive_bools)]
    #[must_use]
    pub fn new(
        action_tx: UnboundedSender<Action>,
        layered_config: ConfigLayers,
        cable_channels: Cable,
        frecency: FrecencyHandle,
    ) -> Self {
        let merged_config = {
            // this is to keep the outer merged config immutable
            let mut m = layered_config.merge();
            m.input_map.merge_globals_with(
                &cable_channels.get_channels_shortcut_keybindings(),
            );
            m
        };

        let mut results_picker = Picker::new(merged_config.input.clone());
        if merged_config.input_bar_position == InputPosition::Bottom {
            results_picker = results_picker.inverted();
        }

        let matching_mode = if merged_config.exact_match {
            MatchingMode::Substring
        } else {
            MatchingMode::Fuzzy
        };

        // previewer
        let preview_handles = merged_config
            .channel_preview_command
            .as_ref()
            .map(|command| {
                Self::setup_previewer(
                    command,
                    merged_config.channel_preview_cached,
                    merged_config.channel_preview_offset.clone(),
                    merged_config.preview_panel_header.clone(),
                    merged_config.preview_panel_footer.clone(),
                )
            });

        let frecency_config =
            if merged_config.channel_frecency && !merged_config.no_sort {
                Some((frecency.clone(), merged_config.channel_name.clone()))
            } else {
                None
            };

        let mut channel = CableChannel::new(
            merged_config.channel_source_command.clone(),
            merged_config.channel_source_entry_delimiter,
            merged_config.channel_source_ansi,
            merged_config.channel_source_display.clone(),
            merged_config.channel_source_output.clone(),
            merged_config.channel_preview_command.is_some(),
            merged_config.no_sort,
            frecency_config,
        );

        let app_metadata = AppMetadata::new(
            env!("CARGO_PKG_VERSION").to_string(),
            std::env::current_dir()
                .unwrap_or_else(|e| os_error_exit(&e.to_string()))
                .to_string_lossy()
                .to_string(),
        );
        let base_theme = Theme::from_name(&merged_config.theme);
        let theme = base_theme
            .merge_with_overrides(&merged_config.theme_overrides)
            .unwrap_or_else(|e| {
                error!("Failed to apply theme overrides: {}", e);
                base_theme
            });
        let colorscheme = (&theme).into();

        let pattern = Television::preprocess_pattern(
            matching_mode,
            &merged_config
                .input
                .clone()
                .unwrap_or(EMPTY_STRING.to_string()),
        );

        channel.find(&pattern);

        let preview_state = PreviewState::new(
            channel.supports_preview(),
            Preview::default(),
            0,
        );

        let remote_control = if merged_config.remote_disabled {
            None
        } else {
            Some(RemoteControl::new(
                cable_channels,
                merged_config.remote_sort_alphabetically,
            ))
        };

        // Action picker is lazily initialized when toggled
        let action_picker = None;

        Self {
            action_tx,
            merged_config: layered_config.merge(),
            layered_config,
            channel,
            remote_control,
            action_picker,
            mode: Mode::Channel,
            currently_selected: None,
            current_pattern: EMPTY_STRING.to_string(),
            results_picker,
            matching_mode,
            rc_picker: Picker::default(),
            ap_picker: Picker::default(),
            preview_state,
            preview_handles,
            app_metadata: Arc::new(app_metadata),
            colorscheme: Arc::new(colorscheme),
            ticks: 0,
            ui_state: UiState::default(),
            frecency,
            missing_requirements_popup: None,
        }
    }

    fn setup_previewer(
        command: &CommandSpec,
        cached: bool,
        offset_expr: Option<Template>,
        title_template: Option<Template>,
        footer_template: Option<Template>,
    ) -> (UnboundedSender<PreviewRequest>, UnboundedReceiver<Preview>) {
        let (preview_requests_tx, preview_requests_rx) = unbounded_channel();
        let (preview_results_tx, preview_results_rx) = unbounded_channel();
        let previewer = Previewer::new(
            command,
            offset_expr,
            title_template,
            footer_template,
            // NOTE: this could be a per-channel configuration option in the future
            PreviewerConfig::default(),
            preview_requests_rx,
            preview_requests_tx.clone(),
            preview_results_tx,
            cached,
        );
        tokio::spawn(async move { previewer.run().await });
        (preview_requests_tx, preview_results_rx)
    }

    pub fn update_ui_state(&mut self, ui_state: UiState) {
        self.ui_state = ui_state;
    }

    pub fn dump_context(&self) -> Ctx {
        let channel_state = ChannelState::new(
            self.current_channel(),
            self.channel.selected_entries().clone(),
            self.channel.total_count(),
            self.channel.running(),
            self.channel.current_command().to_string(),
            self.channel.source_index(),
            self.channel.source_count(),
        );
        let tv_state = TvState::new(
            self.mode,
            self.currently_selected.clone(),
            self.results_picker.clone(),
            self.rc_picker.clone(),
            self.ap_picker.clone(),
            channel_state,
            self.preview_state.for_render_context(
                self.ui_state
                    .layout
                    .preview_window
                    .as_ref()
                    .map_or(0, |r| r.height as usize),
            ),
            self.missing_requirements_popup.clone(),
        );

        Ctx::new(
            tv_state,
            self.merged_config.clone(),
            self.colorscheme.clone(),
            self.app_metadata.clone(),
            std::time::Instant::now(),
            self.ui_state.layout,
        )
    }

    pub fn current_channel(&self) -> String {
        self.merged_config.channel_name.clone()
    }

    pub fn change_channel(&mut self, channel_prototype: &ChannelPrototype) {
        // shutdown the current channel and reset state
        self.preview_state.reset();
        self.reset_picker_selection();
        self.reset_picker_input();
        self.current_pattern = EMPTY_STRING.to_string();
        self.channel.shutdown();
        if let Some((sender, _)) = &self.preview_handles {
            sender
                .send(PreviewRequest::Shutdown)
                .expect("Failed to send shutdown signal to previewer");
        }
        // setup the new channel
        debug!("Changing channel to {:?}", channel_prototype);
        self.layered_config
            .update_channel(channel_prototype.clone());
        self.merged_config = self.layered_config.merge();
        // merge channel shortcuts if remote control is enabled
        if let Some(rc) = &mut self.remote_control {
            self.merged_config.input_map.merge_globals_with(
                &rc.cable_channels.get_channels_shortcut_keybindings(),
            );
        }

        self.preview_handles =
            self.merged_config.channel_preview_command.as_ref().map(
                |command| {
                    Self::setup_previewer(
                        command,
                        self.merged_config.channel_preview_cached,
                        self.merged_config.channel_preview_offset.clone(),
                        self.merged_config.preview_panel_header.clone(),
                        self.merged_config.preview_panel_footer.clone(),
                    )
                },
            );
        // Set preview state enabled based on both channel capability and UI configuration
        self.preview_state.enabled = channel_prototype.preview.is_some()
            && !self.merged_config.preview_panel_hidden;

        // Build frecency config if enabled for this channel and sorting is enabled
        let frecency_config = if self.merged_config.channel_frecency
            && !self.merged_config.no_sort
        {
            Some((
                self.frecency.clone(),
                self.merged_config.channel_name.clone(),
            ))
        } else {
            None
        };

        self.channel = CableChannel::new(
            self.merged_config.channel_source_command.clone(),
            self.merged_config.channel_source_entry_delimiter,
            self.merged_config.channel_source_ansi,
            self.merged_config.channel_source_display.clone(),
            self.merged_config.channel_source_output.clone(),
            self.merged_config.channel_preview_command.is_some(),
            self.merged_config.no_sort,
            frecency_config,
        );
        self.channel.load();
    }

    pub fn find(&mut self, pattern: &str) {
        match self.mode {
            Mode::Channel => {
                let processed_pattern =
                    Self::preprocess_pattern(self.matching_mode, pattern);
                self.channel.find(&processed_pattern);
            }
            Mode::RemoteControl => {
                if let Some(rc) = self.remote_control.as_mut() {
                    rc.find(pattern);
                }
            }
            Mode::ActionPicker => {
                if let Some(ap) = self.action_picker.as_mut() {
                    ap.find(pattern);
                }
            }
        }
    }

    fn preprocess_pattern(mode: MatchingMode, pattern: &str) -> String {
        if mode == MatchingMode::Substring {
            let parts: Vec<&str> = pattern.split_ascii_whitespace().collect();
            if parts.is_empty() {
                return pattern.to_string();
            }

            let capacity = parts.iter().map(|s| s.len() + 2).sum::<usize>()
                + parts.len()
                - 1;
            let mut result = String::with_capacity(capacity);

            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    result.push(' ');
                }
                result.push('\'');
                result.push_str(part);
            }
            result
        } else {
            pattern.to_string()
        }
    }

    pub fn get_selected_entry(&mut self) -> Option<Entry> {
        if self.channel.result_count() == 0 {
            return None;
        }
        self.selected_index()
            .map(|idx| self.channel.get_result(idx))
            .and_then(|entry| entry)
    }

    pub fn get_selected_cable_entry(&mut self) -> Option<CableEntry> {
        if self
            .remote_control
            .as_ref()
            .expect("remote control should be Some when in RC mode")
            .result_count()
            == 0
        {
            return None;
        }
        self.selected_index().and_then(|idx| {
            self.remote_control.as_mut().map(|rc| rc.get_result(idx))
        })
    }

    /// Return the currently selected index across pickers, depending on the
    /// active mode.
    #[allow(clippy::cast_possible_truncation)]
    fn selected_index(&self) -> Option<u32> {
        match self.mode {
            Mode::Channel => self.results_picker.selected().map(|i| i as u32),
            Mode::RemoteControl => self.rc_picker.selected().map(|i| i as u32),
            Mode::ActionPicker => self.ap_picker.selected().map(|i| i as u32),
        }
    }

    #[must_use]
    pub fn get_selected_entries(&mut self) -> Option<FxHashSet<Entry>> {
        // if nothing is selected, return the currently hovered entry
        if self.channel.selected_entries().is_empty() {
            return self
                .get_selected_entry()
                .map(|e| FxHashSet::from_iter([e]));
        }
        Some(self.channel.selected_entries().clone())
    }

    /// Unified cursor movement for both Channel and Remote-control pickers.
    pub fn move_cursor(&mut self, movement: Movement, step: u32) {
        match self.mode {
            Mode::Channel => {
                self.results_picker.move_cursor(
                    movement,
                    step,
                    self.channel.result_count() as usize,
                    self.ui_state.layout.results.height.saturating_sub(2)
                        as usize,
                );
            }
            Mode::RemoteControl => {
                let total_results = self
                    .remote_control
                    .as_ref()
                    .expect("remote control should be Some when in RC mode")
                    .result_count()
                    as usize;
                self.rc_picker.move_cursor(
                    movement,
                    step,
                    total_results,
                    self.ui_state.layout.remote_control.expect(
                        "remote UI panel should be contained in the layout when in RC mode"
                    ).height.saturating_sub(5) // accounting for borders (2) and input box (3)
                        as usize,
                );
            }
            Mode::ActionPicker => {
                let total_results =
                    self.action_picker
                        .as_ref()
                        .expect("action picker should be Some when in AP mode")
                        .result_count() as usize;
                self.ap_picker.move_cursor(
                    movement,
                    step,
                    total_results,
                    self.ui_state.layout.action_picker.expect(
                        "action picker UI panel should be contained in the layout when in AP mode"
                    ).height.saturating_sub(5) // accounting for borders (2) and input box (3)
                        as usize,
                );
            }
        }
    }

    fn reset_picker_selection(&mut self) {
        match self.mode {
            Mode::Channel => self.results_picker.reset_selection(),
            Mode::RemoteControl => {
                self.rc_picker.reset_selection();
            }
            Mode::ActionPicker => {
                self.ap_picker.reset_selection();
            }
        }
    }

    fn reset_picker_input(&mut self) {
        match self.mode {
            Mode::Channel => self.results_picker.reset_input(),
            Mode::RemoteControl => {
                self.rc_picker.reset_input();
            }
            Mode::ActionPicker => {
                self.ap_picker.reset_input();
            }
        }
    }

    /// Update the current pattern and input field (used for history navigation)
    pub fn set_pattern(&mut self, pattern: &str) {
        self.current_pattern = pattern.to_string();
        if self.mode == Mode::Channel {
            self.results_picker.input = self
                .results_picker
                .input
                .clone()
                .with_value(pattern.to_string());
            self.find(pattern);
            self.reset_picker_selection();
        }
    }
}

/// Always render the first N ticks.
///
/// This is to ensure there are no startup artefacts and the UI
/// stabilizes rapidly after startup.
const FIRST_TICKS_TO_RENDER: u64 = 10;
/// Render every N ticks.
///
/// Without any user input, this is the default rendering interval.
const RENDERING_INTERVAL: u64 = 25;
/// Render every N ticks if the channel is currently running.
///
/// This ensures that the UI stays in sync with the channel
/// state (loading indicator, updating results, etc.).
const RENDERING_INTERVAL_FAST: u64 = 3;

impl Television {
    /// This contains the logic to determine whether a render should be performed
    /// based on the current tick count, channel state, and the action that
    /// triggered the update.
    fn should_render(&self, action: &Action) -> bool {
        // always render the first N ticks
        (self.ticks < FIRST_TICKS_TO_RENDER
            // then render at regular intervals
            || self.ticks.is_multiple_of(RENDERING_INTERVAL)
            // more frequently if the channel is running
            || (self.channel.running()
                && self.ticks.is_multiple_of(RENDERING_INTERVAL_FAST))
            // always render on input actions that modify the ui state
            || matches!(
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
                    | Action::ToggleHelp
                    | Action::TogglePreview
                    | Action::ToggleStatusBar
                    | Action::ToggleRemoteControl
                    | Action::ToggleActionPicker
                    | Action::ToggleOrientation
                    | Action::CopyEntryToClipboard
                    | Action::CycleSources
                    | Action::CyclePreviews
                    | Action::ReloadSource
            ))
            // We want to avoid too much rendering while the channel is reloading
            // to prevent UI flickering.
            && !self
                .channel
                .reloading()
    }

    pub fn update_preview_state(
        &mut self,
        selected_entry: &Option<Entry>,
    ) -> Result<()> {
        if let Some(selected_entry) = selected_entry {
            if let Some((sender, receiver)) = &mut self.preview_handles {
                // send a preview request if the preview state is out of sync
                // with the currently selected entry
                // FIXME: this can't only rely on raw (ex: lines numbers may change for text
                // but we don't want to regenerate the preview if the file is the same)
                // NOTE: this is fine for now since we'll get a cache hit if cache is enabled
                if selected_entry.raw != self.preview_state.preview.entry_raw {
                    sender.send(PreviewRequest::Preview(Ticket::new(
                        selected_entry.clone(),
                    )))?;
                }
                // try to receive a preview update
                if let Ok(preview) = receiver.try_recv() {
                    let initial_scroll = Self::calculate_scroll(
                        &preview,
                        self.ui_state.layout.preview_window.as_ref(),
                    );
                    self.preview_state.update(preview, initial_scroll);
                    self.action_tx.send(Action::Render)?;
                }
            }
        } else {
            self.preview_state.reset();
        }
        Ok(())
    }

    fn calculate_scroll(
        preview: &Preview,
        preview_window: Option<&Rect>,
    ) -> u16 {
        if let Some(window) = preview_window
            && let Some(target_line) = preview.target_line
        {
            // this places the target line 3 lines above the center of the preview window
            return target_line
                .saturating_sub((window.height / 2).saturating_sub(3));
        }
        0
    }

    pub fn update_results_picker_state(&mut self) {
        if self.results_picker.selected().is_none()
            && self.channel.result_count() > 0
        {
            self.results_picker.select(Some(0));
            self.results_picker.relative_select(Some(0));
        }

        {
            let offset = u32::try_from(self.results_picker.offset()).unwrap();
            let height =
                self.ui_state.layout.results.height.saturating_sub(2).into(); // -2 for borders

            self.results_picker.entries =
                Arc::new(self.channel.results(height, offset));
        }
        self.results_picker.total_items = self.channel.result_count();
    }

    pub fn update_rc_picker_state(&mut self) {
        if self.rc_picker.selected().is_none()
            && self.remote_control.as_ref().unwrap().result_count() > 0
        {
            self.rc_picker.select(Some(0));
            self.rc_picker.relative_select(Some(0));
        }

        {
            let offset = u32::try_from(self.rc_picker.offset()).unwrap();
            let height = self
                .ui_state
                .layout
                .remote_control
                .unwrap_or_default()
                .height
                .saturating_sub(5)
                .into();
            let new_entries = self
                .remote_control
                .as_mut()
                .unwrap()
                .results(height, offset);

            self.rc_picker.entries = Arc::new(new_entries);
        }
        self.rc_picker.total_items =
            self.remote_control.as_ref().unwrap().total_count();
    }

    pub fn update_ap_picker_state(&mut self) {
        let Some(ap) = self.action_picker.as_ref() else {
            return;
        };

        if self.ap_picker.selected().is_none() && ap.result_count() > 0 {
            self.ap_picker.select(Some(0));
            self.ap_picker.relative_select(Some(0));
        }

        {
            let offset = u32::try_from(self.ap_picker.offset()).unwrap();
            let height = self
                .ui_state
                .layout
                .action_picker
                .unwrap_or_default()
                .height
                .saturating_sub(5)
                .into();
            let new_entries =
                self.action_picker.as_mut().unwrap().results(height, offset);

            self.ap_picker.entries = Arc::new(new_entries);
        }
        self.ap_picker.total_items =
            self.action_picker.as_ref().unwrap().total_count();
    }

    /// Initialize the action picker with the current channel's actions.
    fn init_action_picker(&mut self) {
        // Build a map from action strings to keybindings
        let mut action_keybindings = rustc_hash::FxHashMap::default();
        for (key, actions) in
            self.merged_config.input_map.channel_keybindings.iter()
        {
            for action in actions.as_slice() {
                if let Action::ExternalAction(action_str) = action {
                    action_keybindings.insert(action_str.clone(), *key);
                }
            }
        }
        // Also check global keybindings for external actions
        for (key, actions) in
            self.merged_config.input_map.global_keybindings.iter()
        {
            for action in actions.as_slice() {
                if let Action::ExternalAction(action_str) = action {
                    action_keybindings
                        .entry(action_str.clone())
                        .or_insert(*key);
                }
            }
        }

        self.action_picker = Some(ActionPicker::new(
            &self.merged_config.channel_actions,
            &action_keybindings,
        ));
    }

    pub fn handle_input_action(&mut self, action: &Action) {
        let input = match self.mode {
            Mode::Channel => &mut self.results_picker.input,
            Mode::RemoteControl => &mut self.rc_picker.input,
            Mode::ActionPicker => &mut self.ap_picker.input,
        };
        input.handle(convert_action_to_input_request(action).unwrap());
        match action {
            Action::AddInputChar(_)
            | Action::DeletePrevChar
            | Action::DeletePrevWord
            | Action::DeleteLine
            | Action::DeleteNextChar => {
                let new_pattern = input.value().to_string();
                if new_pattern != self.current_pattern {
                    self.current_pattern.clone_from(&new_pattern);
                    self.find(&new_pattern);
                    self.reset_picker_selection();
                }
            }
            _ => {}
        }
    }

    pub fn handle_toggle_selection(&mut self, action: &Action) {
        if matches!(self.mode, Mode::Channel)
            && let Some(entry) = &self.currently_selected
        {
            self.channel.toggle_selection(entry);
            if matches!(action, Action::ToggleSelectionDown) {
                self.move_cursor(Movement::Next, 1);
            } else {
                self.move_cursor(Movement::Prev, 1);
            }
        }
    }

    pub fn handle_confirm_selection(&mut self) -> Result<()> {
        match self.mode {
            Mode::Channel => {
                self.action_tx.send(Action::SelectAndExit)?;
            }
            Mode::RemoteControl => {
                if let Some(entry) = self.get_selected_cable_entry() {
                    // Check for missing requirements
                    let missing: Vec<String> = entry
                        .requirements
                        .iter()
                        .filter(|r| !r.is_met())
                        .map(|r| r.bin_name.clone())
                        .collect();

                    if !missing.is_empty() {
                        // Show popup instead of changing channel
                        self.missing_requirements_popup =
                            Some(MissingRequirementsPopup {
                                channel_name: entry.channel_name.clone(),
                                missing_requirements: missing,
                            });
                        return Ok(());
                    }

                    let new_channel = self
                        .remote_control
                        .as_ref()
                        .unwrap()
                        .zap(&entry.channel_name);
                    // this resets the RC picker
                    self.reset_picker_selection();
                    self.reset_picker_input();
                    self.remote_control.as_mut().unwrap().find(EMPTY_STRING);
                    self.mode = Mode::Channel;
                    self.change_channel(&new_channel);
                }
            }
            Mode::ActionPicker => {
                if let Some(entry) = self.get_selected_action_entry() {
                    // Close the action picker and dispatch the action
                    self.reset_picker_selection();
                    self.reset_picker_input();
                    if let Some(ap) = self.action_picker.as_mut() {
                        ap.find(EMPTY_STRING);
                    }
                    self.mode = Mode::Channel;
                    self.action_tx
                        .send(Action::ExternalAction(entry.action_string))?;
                }
            }
        }
        Ok(())
    }

    pub fn get_selected_action_entry(&mut self) -> Option<ActionEntry> {
        if self
            .action_picker
            .as_ref()
            .is_none_or(|ap| ap.result_count() == 0)
        {
            return None;
        }
        self.selected_index().and_then(|idx| {
            self.action_picker.as_mut().map(|ap| ap.get_result(idx))
        })
    }

    pub fn handle_copy_entry_to_clipboard(&mut self) {
        if self.mode == Mode::Channel
            && let Some(entries) = self.get_selected_entries()
        {
            let copied_string = entries
                .iter()
                .map(|e| e.raw.clone())
                .collect::<Vec<_>>()
                .join(SPACE);

            tokio::spawn(CLIPBOARD.set(copied_string));
        }
    }

    pub fn cycle_sources(&mut self) {
        if self.mode == Mode::Channel {
            self.channel.cycle_sources();
            self.reset_picker_selection();
        }
    }

    pub fn cycle_previews(&mut self) {
        if self.mode == Mode::Channel
            && let Some((sender, _)) = &self.preview_handles
        {
            sender.send(PreviewRequest::CycleCommand).expect(
                "Failed to send cycle preview command request to previewer",
            );
        }
    }

    pub fn handle_reload_source(&mut self) {
        if self.mode == Mode::Channel {
            let current_pattern = self.current_pattern.clone();
            self.channel.reload();
            // Preserve the current pattern and re-run the search
            self.find(&current_pattern);
        }
    }

    pub fn handle_action(&mut self, action: &Action) -> Result<()> {
        // If popup is showing, only allow certain actions to dismiss it
        if self.missing_requirements_popup.is_some() {
            match action {
                // These actions dismiss the popup and stay in RemoteControl mode
                Action::ConfirmSelection
                | Action::Quit
                | Action::ToggleRemoteControl => {
                    self.missing_requirements_popup = None;
                    return Ok(());
                }
                _ => return Ok(()),
            }
        }

        // handle actions
        match action {
            Action::AddInputChar(_)
            | Action::DeletePrevChar
            | Action::DeletePrevWord
            | Action::DeleteNextChar
            | Action::DeleteLine
            | Action::GoToInputEnd
            | Action::GoToInputStart
            | Action::GoToNextChar
            | Action::GoToPrevChar => {
                self.handle_input_action(action);
            }
            Action::SelectNextEntry => {
                self.move_cursor(Movement::Next, 1);
            }
            Action::SelectPrevEntry => {
                self.move_cursor(Movement::Prev, 1);
            }
            Action::SelectNextPage => {
                if matches!(self.mode, Mode::Channel) {
                    self.move_cursor(
                        Movement::Next,
                        self.ui_state
                            .layout
                            .results
                            .height
                            .saturating_sub(2)
                            .into(),
                    );
                }
            }
            Action::SelectPrevPage => {
                if matches!(self.mode, Mode::Channel) {
                    self.move_cursor(
                        Movement::Prev,
                        self.ui_state
                            .layout
                            .results
                            .height
                            .saturating_sub(2)
                            .into(),
                    );
                }
            }
            Action::ScrollPreviewDown => self.preview_state.scroll_down(1),
            Action::ScrollPreviewUp => self.preview_state.scroll_up(1),
            Action::ScrollPreviewHalfPageDown => {
                self.preview_state.scroll_down(20);
            }
            Action::ScrollPreviewHalfPageUp => {
                self.preview_state.scroll_up(20);
            }

            Action::ToggleSelectionDown | Action::ToggleSelectionUp => {
                self.handle_toggle_selection(action);
            }
            Action::ConfirmSelection => {
                self.handle_confirm_selection()?;
            }
            Action::CopyEntryToClipboard => {
                self.handle_copy_entry_to_clipboard();
            }
            Action::CycleSources => {
                self.cycle_sources();
            }
            Action::CyclePreviews => {
                self.cycle_previews();
            }
            Action::ReloadSource | Action::WatchTimer => {
                self.handle_reload_source();
            }
            Action::SwitchToChannel(channel_name) => {
                if let Some(rc) = &self.remote_control {
                    let prototype = rc.zap(channel_name);
                    self.change_channel(&prototype);
                }
            }
            Action::ToggleRemoteControl => {
                if self.remote_control.is_none()
                    || self.merged_config.remote_disabled
                {
                    return Ok(());
                }
                match self.mode {
                    Mode::Channel => {
                        self.mode = Mode::RemoteControl;
                        self.remote_control
                            .as_mut()
                            .unwrap()
                            .find(EMPTY_STRING);
                        // Reset `ticks` to force an immediate render
                        // See `Television::should_render` for more details
                        self.ticks = 0;
                    }
                    Mode::RemoteControl => {
                        // Reset the RC picker when leaving remote control mode
                        self.reset_picker_input();
                        self.remote_control
                            .as_mut()
                            .unwrap()
                            .find(EMPTY_STRING);
                        self.reset_picker_selection();
                        self.mode = Mode::Channel;
                    }
                    Mode::ActionPicker => {
                        // Close action picker and open remote control
                        self.reset_picker_input();
                        if let Some(ap) = self.action_picker.as_mut() {
                            ap.find(EMPTY_STRING);
                        }
                        self.reset_picker_selection();
                        self.mode = Mode::RemoteControl;
                        self.remote_control
                            .as_mut()
                            .unwrap()
                            .find(EMPTY_STRING);
                        self.ticks = 0;
                    }
                }
            }
            Action::ToggleActionPicker => {
                // Only allow if channel has actions defined
                if self.merged_config.channel_actions.is_empty() {
                    return Ok(());
                }
                match self.mode {
                    Mode::Channel => {
                        self.init_action_picker();
                        self.mode = Mode::ActionPicker;
                        if let Some(ap) = self.action_picker.as_mut() {
                            ap.find(EMPTY_STRING);
                        }
                        self.ticks = 0;
                    }
                    Mode::ActionPicker => {
                        self.reset_picker_input();
                        if let Some(ap) = self.action_picker.as_mut() {
                            ap.find(EMPTY_STRING);
                        }
                        self.reset_picker_selection();
                        self.mode = Mode::Channel;
                    }
                    Mode::RemoteControl => {
                        // Close remote control and open action picker
                        self.reset_picker_input();
                        self.remote_control
                            .as_mut()
                            .unwrap()
                            .find(EMPTY_STRING);
                        self.reset_picker_selection();
                        self.init_action_picker();
                        self.mode = Mode::ActionPicker;
                        if let Some(ap) = self.action_picker.as_mut() {
                            ap.find(EMPTY_STRING);
                        }
                        self.ticks = 0;
                    }
                }
            }
            Action::ToggleHelp => {
                // Only allow toggling if the help panel is not disabled
                if !self.merged_config.help_panel_disabled {
                    self.merged_config.help_panel_hidden =
                        !self.merged_config.help_panel_hidden;
                }
            }
            Action::TogglePreview => {
                // Only allow toggling if in Channel mode and preview is not disabled
                if self.mode == Mode::Channel
                    && !self.merged_config.preview_panel_disabled
                {
                    self.merged_config.preview_panel_hidden =
                        !self.merged_config.preview_panel_hidden;
                }
            }
            Action::ToggleStatusBar => {
                // Only allow toggling if the status bar is not disabled
                if !self.merged_config.status_bar_disabled {
                    self.merged_config.status_bar_hidden =
                        !self.merged_config.status_bar_hidden;
                }
            }
            Action::ToggleOrientation => match self.merged_config.layout {
                Orientation::Portrait => {
                    self.merged_config.layout = Orientation::Landscape;
                }
                Orientation::Landscape => {
                    self.merged_config.layout = Orientation::Portrait;
                }
            },
            _ => {}
        }
        Ok(())
    }

    #[allow(clippy::unused_async)]
    /// Update the television state based on the action provided.
    ///
    /// This function may return an Action that'll be processed by the parent `App`.
    pub fn update(&mut self, action: &Action) -> Result<Option<Action>> {
        self.handle_action(action)?;

        // Always let the background matcher make progress
        self.channel.tick();

        // Only run the full results pipeline when the action could
        // have changed the results or the visible viewport
        if action.affects_results() {
            self.update_results_picker_state();
        }

        if self.remote_control.is_some() && self.mode == Mode::RemoteControl {
            self.update_rc_picker_state();
        }

        if self.action_picker.is_some() && self.mode == Mode::ActionPicker {
            self.update_ap_picker_state();
        }

        if self.mode == Mode::Channel {
            let selected_entry = self.get_selected_entry();
            self.update_preview_state(&selected_entry)?;
            self.currently_selected = selected_entry;
        }
        self.ticks += 1;

        Ok(if self.should_render(action) {
            Some(Action::Render)
        } else {
            None
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{
        action::{Action, Actions},
        cable::Cable,
        cli::{ChannelCli, GlobalCli},
        config::layers::ConfigLayers,
        event::Key,
        frecency::Frecency,
        television::{MatchingMode, Mode, Television},
    };
    use std::sync::Arc;
    use tempfile::tempdir;

    #[test]
    fn test_prompt_preprocessing() {
        let one_word = "test";
        let mult_word = "this is a specific test";
        let expect_one = "'test";
        let expect_mult = "'this 'is 'a 'specific 'test";
        assert_eq!(
            Television::preprocess_pattern(MatchingMode::Substring, one_word),
            expect_one
        );
        assert_eq!(
            Television::preprocess_pattern(MatchingMode::Substring, mult_word),
            expect_mult
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_cli_overrides() {
        use crate::cli::PostProcessedCli;

        let config = crate::config::Config::default();
        let prototype = crate::channels::prototypes::ChannelPrototype::new(
            "test", "echo 1",
        );
        let cli_args = PostProcessedCli {
            channel: ChannelCli {
                exact: true,
                ..Default::default()
            },
            global: GlobalCli {
                no_remote: true,
                ..Default::default()
            },
        };
        let layered_config =
            ConfigLayers::new(config, prototype, cli_args.clone());
        let dir = tempdir().unwrap();
        let frecency = Arc::new(Frecency::new(100, dir.path()));
        let tv = Television::new(
            tokio::sync::mpsc::unbounded_channel().0,
            layered_config,
            Cable::from_prototypes(vec![]),
            frecency,
        );

        assert_eq!(tv.matching_mode, MatchingMode::Substring);
        assert!(tv.remote_control.is_none());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_channel_keybindings_take_precedence() {
        use crate::cli::PostProcessedCli;

        let mut config = crate::config::Config::default();
        config
            .keybindings
            .insert(Key::Ctrl('n'), Action::SelectNextEntry.into());

        let prototype =
            toml::from_str::<crate::channels::prototypes::ChannelPrototype>(
                r#"
            [metadata]
            name = "test"

            [source]
            command = "echo 1"

            [keybindings]
            ctrl-j = "select_next_entry"
            "#,
            )
            .unwrap();

        let cli_args = PostProcessedCli::default();
        let layered_config = ConfigLayers::new(
            config.clone(),
            prototype.clone(),
            cli_args.clone(),
        );
        let dir = tempdir().unwrap();
        let frecency = Arc::new(Frecency::new(100, dir.path()));
        let tv = Television::new(
            tokio::sync::mpsc::unbounded_channel().0,
            layered_config,
            Cable::from_prototypes(vec![]),
            frecency,
        );

        assert_eq!(
            tv.merged_config
                .input_map
                .get_actions_for_key(&Key::Ctrl('j'), &Mode::Channel),
            Some(&Actions::single(Action::SelectNextEntry)),
        );
    }
}
