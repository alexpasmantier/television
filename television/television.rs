use crate::{
    action::Action,
    cable::Cable,
    channels::{
        channel::Channel as CableChannel,
        entry::Entry,
        prototypes::ChannelPrototype,
        remote_control::{CableEntry, RemoteControl},
    },
    config::{Config, Theme},
    draw::{ChannelState, Ctx, TvState},
    errors::os_error_exit,
    features::FeatureFlags,
    input::convert_action_to_input_request,
    picker::{Movement, Picker},
    previewer::{
        Config as PreviewerConfig, Preview, Previewer,
        Request as PreviewRequest, Ticket, state::PreviewState,
    },
    render::UiState,
    screen::{
        colors::Colorscheme,
        layout::InputPosition,
        spinner::{Spinner, SpinnerState},
    },
    utils::{
        clipboard::CLIPBOARD, metadata::AppMetadata, strings::EMPTY_STRING,
    },
};
use anyhow::Result;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tokio::sync::mpsc::{
    UnboundedReceiver, UnboundedSender, unbounded_channel,
};
use tracing::{debug, error};

#[derive(PartialEq, Copy, Clone, Hash, Eq, Debug, Serialize, Deserialize)]
pub enum Mode {
    Channel,
    RemoteControl,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Channel => write!(f, "Channel"),
            Mode::RemoteControl => write!(f, "Remote Control"),
        }
    }
}

#[derive(PartialEq, Copy, Clone, Hash, Eq, Debug, Serialize, Deserialize)]
pub enum MatchingMode {
    Substring,
    Fuzzy,
}

pub struct Television {
    action_tx: UnboundedSender<Action>,
    base_config: Config,
    pub config: Config,
    pub channel: CableChannel,
    pub remote_control: Option<RemoteControl>,
    pub mode: Mode,
    pub currently_selected: Option<Entry>,
    pub current_pattern: String,
    pub matching_mode: MatchingMode,
    pub results_picker: Picker<Entry>,
    pub rc_picker: Picker<CableEntry>,
    pub preview_state: PreviewState,
    pub preview_handles:
        Option<(UnboundedSender<PreviewRequest>, UnboundedReceiver<Preview>)>,
    pub spinner: Spinner,
    pub spinner_state: SpinnerState,
    pub app_metadata: AppMetadata,
    pub colorscheme: Colorscheme,
    pub ticks: u64,
    pub ui_state: UiState,
    pub no_preview: bool,
    pub preview_size: Option<u16>,
    pub current_command_index: usize,
    pub channel_prototype: ChannelPrototype,
}

impl Television {
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::fn_params_excessive_bools)]
    #[must_use]
    pub fn new(
        action_tx: UnboundedSender<Action>,
        channel_prototype: ChannelPrototype,
        base_config: Config,
        input: Option<String>,
        no_remote: bool,
        no_preview: bool,
        preview_size: Option<u16>,
        exact: bool,
        cable_channels: Cable,
    ) -> Self {
        let mut config = Self::merge_base_config_with_prototype_specs(
            &base_config,
            &channel_prototype,
        );

        // Apply CLI overrides after prototype merging to ensure they take precedence
        Self::apply_cli_overrides(&mut config, no_preview, preview_size);

        debug!("Merged config: {:?}", config);

        let mut results_picker = Picker::new(input.clone());
        if config.ui.input_bar_position == InputPosition::Bottom {
            results_picker = results_picker.inverted();
        }

        let matching_mode = if exact {
            MatchingMode::Substring
        } else {
            MatchingMode::Fuzzy
        };

        // previewer
        let preview_handles = Self::setup_previewer(&channel_prototype);

        let mut channel = CableChannel::new(channel_prototype.clone());
        channel.load();

        let app_metadata = AppMetadata::new(
            env!("CARGO_PKG_VERSION").to_string(),
            std::env::current_dir()
                .unwrap_or_else(|e| os_error_exit(&e.to_string()))
                .to_string_lossy()
                .to_string(),
        );
        let base_theme = Theme::from_name(&config.ui.theme);
        let theme = base_theme
            .merge_with_overrides(&config.ui.theme_overrides)
            .unwrap_or_else(|e| {
                error!("Failed to apply theme overrides: {}", e);
                base_theme
            });
        let colorscheme = (&theme).into();

        let patrnn = Television::preprocess_pattern(
            matching_mode,
            &input.unwrap_or(EMPTY_STRING.to_string()),
        );

        channel.find(&patrnn);
        let spinner = Spinner::default();

        let preview_state = PreviewState::new(
            channel.supports_preview(),
            Preview::default(),
            0,
            None,
        );

        let remote_control = if no_remote {
            None
        } else {
            Some(RemoteControl::new(
                cable_channels,
                &config.ui.remote_control,
            ))
        };

        Self {
            action_tx,
            base_config,
            config,
            channel,
            remote_control,
            mode: Mode::Channel,
            currently_selected: None,
            current_pattern: EMPTY_STRING.to_string(),
            results_picker,
            matching_mode,
            rc_picker: Picker::default(),
            preview_state,
            preview_handles,
            spinner,
            spinner_state: SpinnerState::from(&spinner),
            app_metadata,
            colorscheme,
            ticks: 0,
            ui_state: UiState::default(),
            no_preview,
            preview_size,
            current_command_index: 0,
            channel_prototype,
        }
    }

    fn merge_base_config_with_prototype_specs(
        base_config: &Config,
        channel_prototype: &ChannelPrototype,
    ) -> Config {
        let mut config = base_config.clone();
        // keybindings
        if let Some(keybindings) = &channel_prototype.keybindings {
            config.merge_keybindings(&keybindings.bindings);
        }
        // ui
        if let Some(ui_spec) = &channel_prototype.ui {
            config.apply_prototype_ui_spec(ui_spec);
            if config.ui.input_header.is_none() {
                if let Some(header_tpl) = &ui_spec.input_header {
                    config.ui.input_header = Some(header_tpl.clone());
                }
            }
            if config.ui.preview_panel.header.is_none() {
                if let Some(preview_panel) = &ui_spec.preview_panel {
                    if let Some(ph) = &preview_panel.header {
                        config.ui.preview_panel.header = Some(ph.clone());
                    }
                }
            }
            if config.ui.preview_panel.footer.is_none() {
                if let Some(preview_panel) = &ui_spec.preview_panel {
                    if let Some(pf) = &preview_panel.footer {
                        config.ui.preview_panel.footer = Some(pf.clone());
                    }
                }
            }
        }
        config
    }

    /// Apply CLI overrides to ensure they take precedence over channel prototype settings
    fn apply_cli_overrides(
        config: &mut Config,
        no_preview: bool,
        preview_size: Option<u16>,
    ) {
        // Handle preview panel flags - this mirrors the logic in main.rs but only for the subset
        // of flags that Television manages directly
        if no_preview {
            config.ui.features.disable(FeatureFlags::PreviewPanel);
            config.keybindings.remove(&Action::TogglePreview);
        }

        // Apply preview size regardless of preview state
        if let Some(ps) = preview_size {
            config.ui.preview_panel.size = ps;
        }
    }

    fn setup_previewer(
        channel_prototype: &ChannelPrototype,
    ) -> Option<(UnboundedSender<PreviewRequest>, UnboundedReceiver<Preview>)>
    {
        if let Some(preview_spec) = &channel_prototype.preview {
            let (pv_request_tx, pv_request_rx) = unbounded_channel();
            let (pv_preview_tx, pv_preview_rx) = unbounded_channel();
            let previewer = Previewer::new(
                preview_spec,
                // NOTE: this could be a per-channel configuration option in the future
                PreviewerConfig::default(),
                pv_request_rx,
                pv_preview_tx,
                preview_spec.cached,
            );
            tokio::spawn(async move { previewer.run().await });
            Some((pv_request_tx, pv_preview_rx))
        } else {
            None
        }
    }

    pub fn update_ui_state(&mut self, ui_state: UiState) {
        self.ui_state = ui_state;
    }

    pub fn dump_context(&self) -> Ctx {
        let channel_state = ChannelState::new(
            self.channel.prototype.metadata.name.clone(),
            self.channel.selected_entries().clone(),
            self.channel.total_count(),
            self.channel.running(),
            self.channel.current_command().to_string(),
        );
        let tv_state = TvState::new(
            self.mode,
            self.currently_selected.clone(),
            self.results_picker.clone(),
            self.rc_picker.clone(),
            channel_state,
            self.spinner,
            self.preview_state.for_render_context(),
        );

        Ctx::new(
            tv_state,
            self.config.clone(),
            self.colorscheme.clone(),
            self.app_metadata.clone(),
            std::time::Instant::now(),
            self.ui_state.layout,
        )
    }

    pub fn current_channel(&self) -> String {
        self.channel.prototype.metadata.name.clone()
    }

    pub fn change_channel(&mut self, channel_prototype: ChannelPrototype) {
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
        self.preview_handles = Self::setup_previewer(&channel_prototype);
        self.config = Self::merge_base_config_with_prototype_specs(
            &self.base_config,
            &channel_prototype,
        );
        // Reapply CLI overrides to ensure they persist across channel changes
        Self::apply_cli_overrides(
            &mut self.config,
            self.no_preview,
            self.preview_size,
        );
        // Set preview state enabled based on both channel capability and UI configuration
        self.preview_state.enabled = channel_prototype.preview.is_some()
            && self
                .config
                .ui
                .features
                .is_enabled(FeatureFlags::PreviewPanel);
        self.channel_prototype = channel_prototype.clone();
        self.current_command_index = 0;
        self.channel = CableChannel::new(channel_prototype);
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

    pub fn get_selected_entry(&self) -> Option<Entry> {
        if self.channel.result_count() == 0 {
            return None;
        }
        self.selected_index()
            .map(|idx| self.channel.get_result(idx))
            .and_then(|entry| entry)
    }

    pub fn get_selected_cable_entry(&self) -> Option<CableEntry> {
        if self
            .remote_control
            .as_ref()
            .map_or(0, RemoteControl::result_count)
            == 0
        {
            return None;
        }
        self.selected_index().and_then(|idx| {
            self.remote_control.as_ref().map(|rc| rc.get_result(idx))
        })
    }

    /// Return the currently selected index across pickers, depending on the
    /// active mode.
    #[allow(clippy::cast_possible_truncation)]
    fn selected_index(&self) -> Option<u32> {
        match self.mode {
            Mode::Channel => self.results_picker.selected().map(|i| i as u32),
            Mode::RemoteControl => self.rc_picker.selected().map(|i| i as u32),
        }
    }

    #[must_use]
    pub fn get_selected_entries(&self) -> Option<FxHashSet<Entry>> {
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
        let viewport =
            self.ui_state.layout.results.height.saturating_sub(2) as usize;

        match self.mode {
            Mode::Channel => {
                let total = self.channel.result_count() as usize;
                if total == 0 {
                    return;
                }
                self.results_picker
                    .move_cursor(movement, step, total, viewport);
            }
            Mode::RemoteControl => {
                let total = self
                    .remote_control
                    .as_ref()
                    .map_or(0, RemoteControl::result_count)
                    as usize;
                if total == 0 {
                    return;
                }
                self.rc_picker.move_cursor(movement, step, total, viewport);
            }
        }
    }

    fn reset_picker_selection(&mut self) {
        match self.mode {
            Mode::Channel => self.results_picker.reset_selection(),
            Mode::RemoteControl => {
                self.rc_picker.reset_selection();
            }
        }
    }

    fn reset_picker_input(&mut self) {
        match self.mode {
            Mode::Channel => self.results_picker.reset_input(),
            Mode::RemoteControl => {
                self.rc_picker.reset_input();
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
const RENDERING_INTERVAL: u64 = 10;
/// Render every N ticks if the channel is currently running.
///
/// This ensures that the UI stays in sync with the channel
/// state (displaying a spinner, updating results, etc.).
const RENDERING_INTERVAL_FAST: u64 = 3;

impl Television {
    fn should_render(&self, action: &Action) -> bool {
        (self.ticks < FIRST_TICKS_TO_RENDER
            || self.ticks % RENDERING_INTERVAL == 0
            || (self.channel.running()
                && self.ticks % RENDERING_INTERVAL_FAST == 0)
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
                    | Action::CopyEntryToClipboard
                    | Action::CycleSources
                    | Action::ReloadSource
            ))
            // We want to avoid too much rendering while the channel is reloading
            // to prevent UI flickering.
            && !self
                .channel
                .reloading
                .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn update_preview_state(
        &mut self,
        selected_entry: &Option<Entry>,
    ) -> Result<()> {
        if selected_entry.is_none() {
            self.preview_state.reset();
            return Ok(());
        }
        if let Some((sender, receiver)) = &mut self.preview_handles {
            // preview requests
            if *selected_entry != self.currently_selected {
                sender.send(PreviewRequest::Preview(Ticket::new(
                    selected_entry.as_ref().unwrap().clone(),
                )))?;
            }
            // available previews
            let entry = selected_entry.as_ref().unwrap();
            if let Ok(mut preview) = receiver.try_recv() {
                if let Some(template) = &self.config.ui.preview_panel.header {
                    preview.title = template
                        .format(&entry.raw)
                        .unwrap_or_else(|_| entry.raw.clone());
                }

                if let Some(template) = &self.config.ui.preview_panel.footer {
                    preview.footer = template
                        .format(&entry.raw)
                        .unwrap_or_else(|_| String::new());
                }

                let scroll = entry
                    .line_number
                    .unwrap_or(0)
                    .saturating_sub(
                        (self
                            .ui_state
                            .layout
                            .preview_window
                            .map_or(0, |w| w.height.saturating_sub(2)) // borders
                            / 2)
                        .into(),
                    )
                    .saturating_add(3) // 3 lines above the center
                    .try_into()
                    // if the scroll doesn't fit in a u16, just scroll to the top
                    // this is a current limitation of ratatui
                    .unwrap_or(0);
                self.preview_state.update(
                    preview,
                    scroll,
                    entry.line_number.and_then(|l| l.try_into().ok()),
                );
                self.action_tx.send(Action::Render)?;
            }
        }
        Ok(())
    }

    pub fn update_results_picker_state(&mut self) {
        if self.results_picker.selected().is_none()
            && self.channel.result_count() > 0
        {
            self.results_picker.select(Some(0));
            self.results_picker.relative_select(Some(0));
        }

        {
            // Capture immutable data (`offset`, `height`) first, then mutate
            // the picker entries to satisfy the borrow-checker.
            let offset = u32::try_from(self.results_picker.offset()).unwrap();
            let height = self.ui_state.layout.results.height.into();

            let entries = &mut self.results_picker.entries;
            // Re-use the existing allocation instead of constructing a new
            // `Vec` every tick:
            entries.clear();
            entries.extend(self.channel.results(height, offset));
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
            // Capture immutable data (`offset`, `height`) first, then mutate
            // the picker entries to satisfy the borrow-checker.
            let offset = u32::try_from(self.rc_picker.offset()).unwrap();
            let height = self.ui_state.layout.results.height.into();
            let new_entries = self
                .remote_control
                .as_mut()
                .unwrap()
                .results(height, offset);

            let entries = &mut self.rc_picker.entries;
            // Re-use the existing allocation instead of constructing a new
            // `Vec` every tick:
            entries.clear();
            entries.extend(new_entries);
        }
        self.rc_picker.total_items =
            self.remote_control.as_ref().unwrap().total_count();
    }

    pub fn handle_input_action(&mut self, action: &Action) {
        let input = match self.mode {
            Mode::Channel => &mut self.results_picker.input,
            Mode::RemoteControl => &mut self.rc_picker.input,
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
        if matches!(self.mode, Mode::Channel) {
            if let Some(entry) = &self.currently_selected {
                self.channel.toggle_selection(entry);
                if matches!(action, Action::ToggleSelectionDown) {
                    self.move_cursor(Movement::Next, 1);
                } else {
                    self.move_cursor(Movement::Prev, 1);
                }
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
                    self.change_channel(new_channel);
                }
            }
        }
        Ok(())
    }

    pub fn handle_copy_entry_to_clipboard(&mut self) {
        if self.mode == Mode::Channel {
            if let Some(entries) = self.get_selected_entries() {
                let copied_string = entries
                    .iter()
                    .map(|e| e.raw.clone())
                    .collect::<Vec<_>>()
                    .join(" ");

                tokio::spawn(CLIPBOARD.set(copied_string));
            }
        }
    }

    pub fn cycle_sources(&mut self) {
        if self.mode == Mode::Channel {
            self.channel.cycle_sources();
            self.reset_picker_selection();
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
            Action::SelectNextEntry => match self.mode {
                Mode::Channel | Mode::RemoteControl => {
                    self.move_cursor(Movement::Next, 1);
                }
            },
            Action::SelectPrevEntry => match self.mode {
                Mode::Channel | Mode::RemoteControl => {
                    self.move_cursor(Movement::Prev, 1);
                }
            },
            Action::SelectNextPage => match self.mode {
                Mode::Channel | Mode::RemoteControl => {
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
            },
            Action::SelectPrevPage => match self.mode {
                Mode::Channel | Mode::RemoteControl => {
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
            },
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
            Action::ReloadSource | Action::WatchTimer => {
                self.handle_reload_source();
            }
            Action::SwitchToChannel(channel_name) => {
                if let Some(rc) = &self.remote_control {
                    let prototype = rc.zap(channel_name);
                    self.change_channel(prototype);
                }
            }
            Action::ToggleRemoteControl => {
                if self.remote_control.is_none() {
                    return Ok(());
                }
                match self.mode {
                    Mode::Channel => {
                        self.mode = Mode::RemoteControl;
                        self.remote_control
                            .as_mut()
                            .unwrap()
                            .find(EMPTY_STRING);
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
                }
                self.config
                    .ui
                    .features
                    .toggle_visible(FeatureFlags::RemoteControl);
            }
            Action::ToggleHelp => {
                self.config
                    .ui
                    .features
                    .toggle_visible(FeatureFlags::HelpPanel);
            }
            Action::TogglePreview => {
                if self.mode == Mode::Channel {
                    self.config
                        .ui
                        .features
                        .toggle_visible(FeatureFlags::PreviewPanel);
                }
            }
            Action::ToggleStatusBar => {
                self.config
                    .ui
                    .features
                    .toggle_visible(FeatureFlags::StatusBar);
            }
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

        self.update_results_picker_state();

        if self.remote_control.is_some() {
            self.update_rc_picker_state();
        }

        if self.mode == Mode::Channel {
            let selected_entry = self.get_selected_entry();
            self.update_preview_state(&selected_entry)?;
            self.currently_selected = selected_entry;
        }
        self.ticks += 1;

        Ok(if self.should_render(action) {
            if self.channel.running() {
                self.spinner.tick();
            }

            Some(Action::Render)
        } else {
            None
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{
        action::Action,
        cable::Cable,
        config::Binding,
        event::Key,
        television::{MatchingMode, Television},
    };

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
        let config = crate::config::Config::default();
        let prototype = crate::channels::prototypes::ChannelPrototype::new(
            "test", "echo 1",
        );
        let tv = Television::new(
            tokio::sync::mpsc::unbounded_channel().0,
            prototype,
            config.clone(),
            None,
            true,
            false,
            Some(50),
            true,
            Cable::from_prototypes(vec![]),
        );

        assert_eq!(tv.matching_mode, MatchingMode::Substring);
        assert!(!tv.no_preview);
        assert!(tv.remote_control.is_none());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_channel_keybindings_take_precedence() {
        let mut config = crate::config::Config::default();
        config.keybindings.insert(
            Action::SelectNextEntry,
            Binding::SingleKey(Key::Ctrl('n')),
        );

        let prototype =
            toml::from_str::<crate::channels::prototypes::ChannelPrototype>(
                r#"
            [metadata]
            name = "test"

            [source]
            command = "echo 1"

            [keybindings]
            select_next_entry = "ctrl-j"
            "#,
            )
            .unwrap();

        let tv = Television::new(
            tokio::sync::mpsc::unbounded_channel().0,
            prototype,
            config.clone(),
            None,
            true,
            false,
            Some(50),
            true,
            Cable::from_prototypes(vec![]),
        );

        assert_eq!(
            tv.config.keybindings.get(&Action::SelectNextEntry),
            Some(&Binding::SingleKey(Key::Ctrl('j')))
        );
    }
}
