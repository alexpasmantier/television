use crate::{
    action::Action,
    channels::{
        cable::Channel as CableChannel,
        entry::Entry,
        prototypes::{Cable, ChannelPrototype},
        remote_control::RemoteControl,
    },
    config::{Config, Theme},
    draw::{ChannelState, Ctx, TvState},
    input::convert_action_to_input_request,
    picker::Picker,
    previewer::{
        state::PreviewState, Config as PreviewerConfig, Preview, Previewer,
        Request as PreviewRequest, Ticket,
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
use rustc_hash::{FxBuildHasher, FxHashSet};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tokio::sync::mpsc::{
    unbounded_channel, UnboundedReceiver, UnboundedSender,
};

#[derive(PartialEq, Copy, Clone, Hash, Eq, Debug, Serialize, Deserialize)]
pub enum Mode {
    Channel,
    RemoteControl,
}

#[derive(PartialEq, Copy, Clone, Hash, Eq, Debug, Serialize, Deserialize)]
pub enum MatchingMode {
    Substring,
    Fuzzy,
}

pub struct Television {
    action_tx: UnboundedSender<Action>,
    pub config: Config,
    pub channel: CableChannel,
    pub remote_control: Option<RemoteControl>,
    pub mode: Mode,
    pub currently_selected: Option<Entry>,
    pub current_pattern: String,
    pub matching_mode: MatchingMode,
    pub results_picker: Picker,
    pub rc_picker: Picker,
    pub preview_state: PreviewState,
    pub preview_handles:
        Option<(UnboundedSender<PreviewRequest>, UnboundedReceiver<Preview>)>,
    pub spinner: Spinner,
    pub spinner_state: SpinnerState,
    pub app_metadata: AppMetadata,
    pub colorscheme: Colorscheme,
    pub ticks: u64,
    pub ui_state: UiState,
    pub no_help: bool,
}

impl Television {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        action_tx: UnboundedSender<Action>,
        channel_prototype: ChannelPrototype,
        mut config: Config,
        input: Option<String>,
        no_remote: bool,
        no_help: bool,
        exact: bool,
        cable_channels: Cable,
    ) -> Self {
        let mut results_picker = Picker::new(input.clone());
        if config.ui.input_bar_position == InputPosition::Bottom {
            results_picker = results_picker.inverted();
        }

        // previewer
        let preview_handles = Self::setup_previewer(&channel_prototype);

        let mut channel: CableChannel = channel_prototype.into();

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

        let preview_state = PreviewState::new(
            channel.supports_preview(),
            Preview::default(),
            0,
            None,
        );

        let remote_control = if no_remote {
            None
        } else {
            Some(RemoteControl::new(Some(cable_channels)))
        };

        if no_help {
            config.ui.show_help_bar = false;
            config.ui.no_help = true;
        }

        let matching_mode = if exact {
            MatchingMode::Substring
        } else {
            MatchingMode::Fuzzy
        };

        Self {
            action_tx,
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
            no_help,
        }
    }

    fn setup_previewer(
        channel_prototype: &ChannelPrototype,
    ) -> Option<(UnboundedSender<PreviewRequest>, UnboundedReceiver<Preview>)>
    {
        if channel_prototype.preview_command.is_some() {
            let (pv_request_tx, pv_request_rx) = unbounded_channel();
            let (pv_preview_tx, pv_preview_rx) = unbounded_channel();
            let previewer = Previewer::new(
                channel_prototype.preview_command().unwrap(),
                PreviewerConfig::default(),
                pv_request_rx,
                pv_preview_tx,
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
            self.channel.name.clone(),
            self.channel.selected_entries().clone(),
            self.channel.total_count(),
            self.channel.running(),
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
        self.channel.name.clone()
    }

    pub fn change_channel(&mut self, channel_prototype: ChannelPrototype) {
        self.preview_state.reset();
        self.preview_state.enabled =
            channel_prototype.preview_command.is_some();
        self.reset_picker_selection();
        self.reset_picker_input();
        self.current_pattern = EMPTY_STRING.to_string();
        self.channel.shutdown();
        if let Some((sender, _)) = &self.preview_handles {
            sender
                .send(PreviewRequest::Shutdown)
                .expect("Failed to send shutdown signal to previewer");
        }
        self.preview_handles = Self::setup_previewer(&channel_prototype);
        self.channel = channel_prototype.into();
    }

    pub fn find(&mut self, pattern: &str) {
        match self.mode {
            Mode::Channel => {
                self.channel.find(
                    Self::preprocess_pattern(self.matching_mode, pattern)
                        .as_str(),
                );
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
            return pattern
                .split_ascii_whitespace()
                .map(|x| {
                    let mut new = x.to_string();
                    new.insert(0, '\'');
                    new
                })
                .collect::<Vec<String>>()
                .join(" ");
        }
        pattern.to_string()
    }

    #[must_use]
    pub fn get_selected_entry(&self, mode: Option<Mode>) -> Option<Entry> {
        match mode.unwrap_or(self.mode) {
            Mode::Channel => {
                if let Some(i) = self.results_picker.selected() {
                    return self.channel.get_result(i.try_into().unwrap());
                }
                None
            }
            Mode::RemoteControl => {
                if let Some(i) = self.rc_picker.selected() {
                    if let Some(rc) = &self.remote_control {
                        return rc.get_result(i.try_into().unwrap());
                    }
                }
                None
            }
        }
    }

    #[must_use]
    pub fn get_selected_entries(
        &self,
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
            Mode::RemoteControl => (
                self.remote_control.as_ref().unwrap().total_count(),
                &mut self.rc_picker,
            ),
        };
        if result_count == 0 {
            return;
        }
        picker.select_prev(
            step,
            result_count as usize,
            self.ui_state.layout.results.height.saturating_sub(2) as usize,
        );
    }

    pub fn select_next_entry(&mut self, step: u32) {
        let (result_count, picker) = match self.mode {
            Mode::Channel => {
                (self.channel.result_count(), &mut self.results_picker)
            }
            Mode::RemoteControl => (
                self.remote_control.as_ref().unwrap().total_count(),
                &mut self.rc_picker,
            ),
        };
        if result_count == 0 {
            return;
        }
        picker.select_next(
            step,
            result_count as usize,
            self.ui_state.layout.results.height.saturating_sub(2) as usize,
        );
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
        self.ticks < FIRST_TICKS_TO_RENDER
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
                    | Action::ToggleRemoteControl
                    | Action::ToggleSendToChannel
                    | Action::ToggleHelp
                    | Action::TogglePreview
                    | Action::CopyEntryToClipboard
            )
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
            if let Ok(preview) = receiver.try_recv() {
                self.preview_state.update(
                    preview,
                    // scroll to center the selected entry
                    entry
                        .line_number
                        .unwrap_or(0)
                        .saturating_sub(
                            (self
                                .ui_state
                                .layout
                                .preview_window
                                .map_or(0, |w| w.height)
                                / 2)
                            .into(),
                        )
                        .try_into()
                        // if the scroll doesn't fit in a u16, just scroll to the top
                        // this is a current limitation of ratatui
                        .unwrap_or(0),
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

        self.results_picker.entries = self.channel.results(
            self.ui_state.layout.results.height.into(),
            u32::try_from(self.results_picker.offset()).unwrap(),
        );
        self.results_picker.total_items = self.channel.result_count();
    }

    pub fn update_rc_picker_state(&mut self) {
        if self.rc_picker.selected().is_none()
            && self.remote_control.as_ref().unwrap().result_count() > 0
        {
            self.rc_picker.select(Some(0));
            self.rc_picker.relative_select(Some(0));
        }

        self.rc_picker.entries =
            self.remote_control.as_mut().unwrap().results(
                // this'll be more than the actual rc height but it's fine
                self.ui_state.layout.results.height.into(),
                u32::try_from(self.rc_picker.offset()).unwrap(),
            );
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

    pub fn handle_toggle_rc(&mut self) {
        if self.remote_control.is_none() {
            return;
        }
        match self.mode {
            Mode::Channel => {
                self.mode = Mode::RemoteControl;
            }
            Mode::RemoteControl => {
                // this resets the RC picker
                self.reset_picker_input();
                self.remote_control.as_mut().unwrap().find(EMPTY_STRING);
                self.reset_picker_selection();
                self.mode = Mode::Channel;
            }
        }
    }

    pub fn handle_toggle_selection(&mut self, action: &Action) {
        if matches!(self.mode, Mode::Channel) {
            if let Some(entry) = &self.currently_selected {
                self.channel.toggle_selection(entry);
                if matches!(action, Action::ToggleSelectionDown) {
                    self.select_next_entry(1);
                } else {
                    self.select_prev_entry(1);
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
                if let Some(entry) = self.get_selected_entry(None) {
                    let new_channel = self
                        .remote_control
                        .as_ref()
                        .unwrap()
                        .zap(entry.name.as_str())?;
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
            if let Some(entries) = self.get_selected_entries(None) {
                let copied_string = entries
                    .iter()
                    .map(|e| e.name.clone())
                    .collect::<Vec<_>>()
                    .join(" ");

                tokio::spawn(CLIPBOARD.set(copied_string));
            }
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
            Action::SelectNextEntry => {
                self.select_next_entry(1);
            }
            Action::SelectPrevEntry => {
                self.select_prev_entry(1);
            }
            Action::SelectNextPage => {
                self.select_next_entry(
                    self.ui_state
                        .layout
                        .results
                        .height
                        .saturating_sub(2)
                        .into(),
                );
            }
            Action::SelectPrevPage => {
                self.select_prev_entry(
                    self.ui_state
                        .layout
                        .results
                        .height
                        .saturating_sub(2)
                        .into(),
                );
            }
            Action::ScrollPreviewDown => self.preview_state.scroll_down(1),
            Action::ScrollPreviewUp => self.preview_state.scroll_up(1),
            Action::ScrollPreviewHalfPageDown => {
                self.preview_state.scroll_down(20);
            }
            Action::ScrollPreviewHalfPageUp => {
                self.preview_state.scroll_up(20);
            }
            Action::ToggleRemoteControl => {
                self.handle_toggle_rc();
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
            Action::ToggleHelp => {
                if self.no_help {
                    return Ok(());
                }
                self.config.ui.show_help_bar = !self.config.ui.show_help_bar;
            }
            Action::TogglePreview => {
                self.config.ui.show_preview_panel =
                    !self.config.ui.show_preview_panel;
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
            let selected_entry = self.get_selected_entry(None);
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
    use crate::television::{MatchingMode, Television};

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
}
