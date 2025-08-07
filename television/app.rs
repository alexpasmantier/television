use std::{thread::sleep, time::Duration};

use crate::{
    action::Action,
    cable::Cable,
    channels::{
        entry::Entry,
        prototypes::{ActionSpec, ExecutionMode},
    },
    config::layers::LayeredConfig,
    event::{
        ControlEvent, Event, EventLoop, InputEvent, Key, MouseInputEvent,
    },
    history::History,
    render::{RenderingTask, UiState, render},
    television::{Mode, Television},
    tui::{IoStream, Tui, TuiMode},
    utils::command::execute_action,
};
use anyhow::Result;
use rustc_hash::FxHashSet;
use tokio::sync::mpsc;
use tracing::{debug, error, trace};

/// The main application struct that holds the state of the application.
pub struct App {
    pub television: Television,
    /// A flag that indicates whether the application should quit during the next frame.
    should_quit: bool,
    /// A sender channel for actions.
    ///
    /// This is made public so that tests, for instance, can send actions to a running application.
    pub action_tx: mpsc::UnboundedSender<Action>,
    /// The receiver channel for actions.
    action_rx: mpsc::UnboundedReceiver<Action>,
    /// The receiver channel for events.
    event_rx: mpsc::UnboundedReceiver<Event<Key>>,
    /// A sender channel to control the event loop.
    event_control_tx: mpsc::UnboundedSender<ControlEvent>,
    /// A sender channel for rendering tasks.
    render_tx: mpsc::UnboundedSender<RenderingTask>,
    /// The receiver channel for rendering tasks.
    ///
    /// This will most of the time get replaced by the rendering task handle once the rendering
    /// task is started but is needed for tests, where we start the app in "headless" mode and
    /// need to keep a fake rendering channel alive so that the rest of the application can run
    /// without any further modifications.
    #[allow(dead_code)]
    render_rx: mpsc::UnboundedReceiver<RenderingTask>,
    /// A channel that listens to UI updates.
    ui_state_rx: mpsc::UnboundedReceiver<UiState>,
    ui_state_tx: mpsc::UnboundedSender<UiState>,
    /// Render task handle
    render_task: Option<tokio::task::JoinHandle<Result<()>>>,
    /// Watch timer task handle for periodic reloading
    watch_timer_task: Option<tokio::task::JoinHandle<()>>,
    /// Global history for selected entries
    history: History,
}

/// The outcome of an action.
#[derive(Debug, PartialEq)]
pub enum ActionOutcome {
    Entries(FxHashSet<Entry>),
    EntriesWithExpect(FxHashSet<Entry>, Key),
    Input(String),
    None,
    ExternalAction(ActionSpec, FxHashSet<Entry>),
}

/// The result of the application.
#[derive(Debug)]
pub struct AppOutput {
    pub selected_entries: Option<FxHashSet<Entry>>,
    pub expect_key: Option<Key>,
    pub external_action: Option<(ActionSpec, FxHashSet<Entry>)>,
}

impl AppOutput {
    pub fn new(action_outcome: ActionOutcome) -> Self {
        match action_outcome {
            ActionOutcome::Entries(entries) => Self {
                selected_entries: Some(entries),
                expect_key: None,
                external_action: None,
            },
            ActionOutcome::EntriesWithExpect(entries, expect_key) => Self {
                selected_entries: Some(entries),
                expect_key: Some(expect_key),
                external_action: None,
            },
            ActionOutcome::Input(input) => Self {
                selected_entries: Some(FxHashSet::from_iter([Entry::new(
                    input,
                )])),
                expect_key: None,
                external_action: None,
            },
            ActionOutcome::None => Self {
                selected_entries: None,
                expect_key: None,
                external_action: None,
            },
            ActionOutcome::ExternalAction(action_spec, entries) => Self {
                selected_entries: None,
                expect_key: None,
                external_action: Some((action_spec, entries)),
            },
        }
    }
}

const EVENT_BUF_SIZE: usize = 4;
const ACTION_BUF_SIZE: usize = 8;

impl App {
    pub fn new(layered_config: LayeredConfig, cable_channels: Cable) -> Self {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let (render_tx, render_rx) = mpsc::unbounded_channel();
        let (_, event_rx) = mpsc::unbounded_channel();
        let (event_abort_tx, _) = mpsc::unbounded_channel();

        let (ui_state_tx, ui_state_rx) = mpsc::unbounded_channel();
        let television =
            Television::new(action_tx.clone(), layered_config, cable_channels);

        let mut history = History::new(
            television.merged_config.history_size,
            &television.merged_config.channel_name,
            television.merged_config.global_history,
            &television.merged_config.data_dir.clone(),
        );
        if let Err(e) = history.init() {
            error!("Failed to initialize history: {}", e);
        }

        let mut app = Self {
            television,
            should_quit: false,
            action_tx,
            action_rx,
            event_rx,
            event_control_tx: event_abort_tx,
            render_tx,
            render_rx,
            ui_state_rx,
            ui_state_tx,
            render_task: None,
            watch_timer_task: None,
            history,
        };

        // populate input_map by going through all cable channels and adding their shortcuts if remote
        // control is present
        app.update_input_map();

        app
    }

    /// Check if the watch timer is currently active.
    fn watch_active(&self) -> bool {
        self.watch_timer_task.is_some()
    }

    /// Start the watch timer if watch interval is configured
    fn start_watch_timer(&mut self) {
        if self.television.merged_config.watch > 0.0 && !self.watch_active() {
            let action_tx = self.action_tx.clone();
            let interval = std::time::Duration::from_secs_f64(
                self.television.merged_config.watch,
            );

            debug!("Starting watch timer with interval: {:?}", interval);

            let task = tokio::spawn(async move {
                let mut timer = tokio::time::interval(interval);
                timer.set_missed_tick_behavior(
                    tokio::time::MissedTickBehavior::Skip,
                );

                loop {
                    timer.tick().await;
                    if action_tx.send(Action::WatchTimer).is_err() {
                        break;
                    }
                }
            });

            self.watch_timer_task = Some(task);
        }
    }

    /// Stop the watch timer
    fn stop_watch_timer(&mut self) {
        if let Some(task) = self.watch_timer_task.take() {
            task.abort();
        }
        self.watch_timer_task = None;
        debug!("Stopped watch timer");
    }

    /// Restart the watch timer
    fn restart_watch_timer(&mut self) {
        self.stop_watch_timer();
        self.start_watch_timer();
    }

    /// Update the `input_map` from the television's current config.
    ///
    /// This should be called whenever the channel changes to ensure the `input_map` includes the
    /// channel's keybindings and shortcuts for all other channels if the remote control is
    /// enabled.
    fn update_input_map(&mut self) {
        // Add channel specific shortcuts
        if let Some(rc) = &self.television.remote_control {
            let shortcut_keybindings =
                rc.cable_channels.get_channels_shortcut_keybindings();
            self.television
                .merged_config
                .input_map
                .merge_key_bindings(&shortcut_keybindings);
        }
        debug!(
            "Updated input_map (with shortcuts): {:?}",
            self.television.merged_config.input_map
        );
    }

    /// Updates the history configuration to match the current channel.
    fn update_history(&mut self) {
        // Update existing history with new channel context
        self.history.update_channel_context(
            &self.television.merged_config.channel_name,
            self.television.merged_config.global_history,
        );
    }

    /// Run the application main loop.
    ///
    /// This function will start the event loop and the rendering loop and handle
    /// all actions that are sent to the application.
    /// The function will return the selected entry if the application is exited.
    ///
    /// # Arguments
    /// * `is_output_tty` - A flag that indicates whether the output is a tty.
    ///
    /// # Returns
    /// The selected entry (if any) if the application is exited.
    ///
    /// # Errors
    /// If an error occurs during the execution of the application.
    pub async fn run(
        &mut self,
        is_output_tty: bool,
        headless: bool,
    ) -> Result<AppOutput> {
        // Rendering loop
        if !headless {
            debug!("Starting rendering loop");
            let (render_tx, render_rx) = mpsc::unbounded_channel();
            self.render_tx = render_tx.clone();
            let ui_state_tx = self.ui_state_tx.clone();
            let action_tx_r = self.action_tx.clone();
            let tui_mode = Self::determine_tui_mode(
                self.television.merged_config.height,
                self.television.merged_config.width,
                self.television.merged_config.inline,
            )?;
            let stream = if is_output_tty {
                debug!("Rendering to stdout");
                IoStream::Stdout.to_stream()
            } else {
                debug!("Rendering to stderr");
                IoStream::BufferedStderr.to_stream()
            };
            let mut tui = Tui::new(stream, &tui_mode)
                .expect("Failed to create TUI instance");
            debug!("Entering tui");
            tui.enter().expect("Failed to enter TUI mode");

            self.render_task = Some(tokio::spawn(async move {
                render(render_rx, action_tx_r, ui_state_tx, tui).await
            }));
            self.action_tx
                .send(Action::Render)
                .expect("Unable to send init render action.");
        }

        // Event loop
        if !headless {
            debug!("Starting backend event loop");
            let event_loop =
                EventLoop::new(self.television.merged_config.tick_rate);
            self.event_rx = event_loop.rx;
            self.event_control_tx = event_loop.control_tx;
        }

        // Start watch timer if configured
        self.start_watch_timer();

        // Main loop
        debug!("Starting event handling loop");
        let action_tx = self.action_tx.clone();
        let mut event_buf = Vec::with_capacity(EVENT_BUF_SIZE);
        let mut action_buf = Vec::with_capacity(ACTION_BUF_SIZE);
        let mut action_outcome;

        trace!("Entering main event loop");
        loop {
            // handle event and convert to action
            trace!("Waiting for new events...");
            if self
                .event_rx
                .recv_many(&mut event_buf, EVENT_BUF_SIZE)
                .await
                > 0
            {
                for event in event_buf.drain(..) {
                    let actions = self.convert_event_to_actions(event);
                    for action in actions {
                        if action != Action::Tick {
                            debug!("Queuing new action: {action:?}");
                        }
                        action_tx.send(action)?;
                    }
                }
            }
            trace!("Event buffer processed, handling actions...");
            // It's important that this shouldn't block if no actions are available
            action_outcome = self.handle_actions(&mut action_buf).await?;

            if self.television.merged_config.select_1
                && !self.television.channel.running()
                && self.television.channel.total_count() == 1
            {
                // If `self.select_1` is true, the channel is not running, and there is
                // only one entry available, automatically select the first entry.
                if let Some(outcome) = self.maybe_select_1() {
                    action_outcome = outcome;
                }
            } else if self.television.merged_config.take_1
                && !self.television.channel.running()
            {
                // If `take_1` is true and the channel has finished loading,
                // automatically take the first entry regardless of count.
                // If there are no entries, exit with None.
                action_outcome = self.maybe_take_1();
            } else if self.television.merged_config.take_1_fast {
                // If `take_1_fast` is true, immediately take the first entry without
                // waiting for loading to finish. If there are no entries, exit with None.
                action_outcome = self.maybe_take_1();
            }

            if self.should_quit {
                // send a termination signal to the event loop
                if !headless {
                    self.event_control_tx.send(ControlEvent::Abort)?;
                }

                // persist search history
                if let Err(e) = self.history.save_to_file() {
                    error!("Failed to persist history: {}", e);
                }

                // wait for the rendering task to finish
                if let Some(rendering_task) = self.render_task.take() {
                    rendering_task.await?.expect("Rendering task failed");
                }

                return Ok(AppOutput::new(action_outcome));
            }
        }
    }

    /// Run the application in headless mode.
    ///
    /// This function will start the event loop and handle all actions that are sent to the
    /// application but will never start the rendering loop. This is mostly used in tests as
    /// a means to run the application and control it via the actions channel.
    pub async fn run_headless(&mut self) -> Result<AppOutput> {
        self.run(false, true).await
    }

    /// Convert an event to an action.
    ///
    /// This function will convert an event to an action based on the current
    /// mode the television is in.
    ///
    /// # Arguments
    /// * `event` - The event to convert to actions.
    ///
    /// # Returns
    /// A vector of actions that correspond to the given event. Multiple actions
    /// will be returned for keys/events bound to action sequences.
    fn convert_event_to_actions(&self, event: Event<Key>) -> Vec<Action> {
        let actions = match event {
            Event::Input(keycode) => {
                // First try to get actions based on keybindings
                if let Some(actions) = self
                    .television
                    .merged_config
                    .input_map
                    .get_actions_for_key(&keycode)
                {
                    let actions_vec = actions.as_slice().to_vec();
                    debug!("Keybinding found: {actions_vec:?}");
                    actions_vec
                } else {
                    // fallback to text input events
                    match keycode {
                        Key::Char(c) => vec![Action::AddInputChar(c)],
                        _ => vec![Action::NoOp],
                    }
                }
            }
            Event::Mouse(mouse_event) => {
                // Convert mouse event to InputEvent and use the input_map
                if self.television.mode == Mode::Channel {
                    let input_event = InputEvent::Mouse(MouseInputEvent {
                        kind: mouse_event.kind,
                        position: (mouse_event.column, mouse_event.row),
                    });
                    self.television
                        .merged_config
                        .input_map
                        .get_actions_for_input(&input_event)
                        .unwrap_or_else(|| vec![Action::NoOp])
                } else {
                    vec![Action::NoOp]
                }
            }
            // terminal events
            Event::Tick => vec![Action::Tick],
            Event::Resize(x, y) => vec![Action::Resize(x, y)],
            Event::FocusGained => vec![Action::Resume],
            Event::FocusLost => vec![Action::Suspend],
            Event::Closed => vec![Action::NoOp],
        };

        // Filter out Tick actions for logging
        let non_tick_actions: Vec<&Action> =
            actions.iter().filter(|a| **a != Action::Tick).collect();
        if !non_tick_actions.is_empty() {
            trace!("Converted {event:?} to actions: {non_tick_actions:?}");
        }

        // Filter out NoOp actions
        actions
            .into_iter()
            .filter(|action| *action != Action::NoOp)
            .collect()
    }

    /// Handle actions.
    ///
    /// This function will handle all actions that are sent to the application.
    /// The function will return the selected entry if the application is exited.
    ///
    /// Note that this function will not block if no actions are available.
    ///
    /// # Returns
    /// The selected entry (if any) if the application is exited.
    ///
    /// # Errors
    /// If an error occurs during the execution of the application.
    async fn handle_actions(
        &mut self,
        buf: &mut Vec<Action>,
    ) -> Result<ActionOutcome> {
        if self.action_rx.is_empty() {
            return Ok(ActionOutcome::None);
        }
        if self.action_rx.recv_many(buf, ACTION_BUF_SIZE).await > 0 {
            for action in buf.drain(..) {
                if action != Action::Tick {
                    trace!("{action:?}");
                }
                match action {
                    Action::Quit => {
                        if self.television.mode == Mode::RemoteControl {
                            self.action_tx
                                .send(Action::ToggleRemoteControl)?;
                        } else {
                            self.stop_watch_timer();
                            self.should_quit = true;
                            self.render_tx.send(RenderingTask::Quit)?;
                        }
                    }
                    Action::Suspend => {
                        self.render_tx.send(RenderingTask::Suspend)?;
                    }
                    Action::Resume => {
                        self.render_tx.send(RenderingTask::Resume)?;
                    }
                    Action::SelectAndExit => {
                        self.should_quit = true;
                        if !self.render_tx.is_closed() {
                            self.render_tx.send(RenderingTask::Quit)?;
                        }
                        if let Some(entries) =
                            self.television.get_selected_entries()
                        {
                            // Add current query to history
                            let query =
                                self.television.current_pattern.clone();
                            self.history.add_entry(
                                query,
                                self.television.current_channel(),
                            )?;
                            return Ok(ActionOutcome::Entries(entries));
                        }

                        return Ok(ActionOutcome::Input(
                            self.television.current_pattern.clone(),
                        ));
                    }
                    Action::Expect(k) => {
                        self.should_quit = true;
                        if !self.render_tx.is_closed() {
                            self.render_tx.send(RenderingTask::Quit)?;
                        }
                        if let Some(entries) =
                            self.television.get_selected_entries()
                        {
                            // Add current query to history
                            let query =
                                self.television.current_pattern.clone();
                            self.history.add_entry(
                                query,
                                self.television.current_channel(),
                            )?;
                            return Ok(ActionOutcome::EntriesWithExpect(
                                entries, k,
                            ));
                        }

                        return Ok(ActionOutcome::Input(
                            self.television.current_pattern.clone(),
                        ));
                    }
                    Action::ClearScreen => {
                        self.render_tx.send(RenderingTask::ClearScreen)?;
                    }
                    Action::Resize(w, h) => {
                        self.render_tx.send(RenderingTask::Resize(w, h))?;
                    }
                    Action::Render => {
                        // forward to the rendering task
                        self.render_tx.send(RenderingTask::Render(
                            Box::new(self.television.dump_context()),
                        ))?;
                        // update the television UI state with the previous frame
                        if let Ok(ui_state) = self.ui_state_rx.try_recv() {
                            self.television.update_ui_state(ui_state);
                        }
                    }
                    Action::SelectPrevHistory => {
                        if let Some(history_entry) =
                            self.history.get_previous_entry()
                        {
                            self.television.set_pattern(&history_entry.query);
                        }
                    }
                    Action::SelectNextHistory => {
                        if let Some(history_entry) =
                            self.history.get_next_entry()
                        {
                            self.television.set_pattern(&history_entry.query);
                        } else {
                            // At the end of history, clear the input
                            self.television.set_pattern("");
                        }
                    }
                    Action::ExternalAction(ref action_name) => {
                        debug!("External action triggered: {}", action_name);

                        if let Some(selected_entries) =
                            self.television.get_selected_entries()
                        {
                            if let Some(action_spec) = self
                                .television
                                .merged_config
                                .channel_actions
                                .get(action_name)
                                .cloned()
                            {
                                match action_spec.mode {
                                    // suspend the TUI and execute the action
                                    ExecutionMode::Fork => {
                                        self.run_external_command_fork(
                                            &action_spec,
                                            &selected_entries,
                                        )?;
                                    }
                                    // clean up and exit the TUI and execute the action
                                    ExecutionMode::Execute => {
                                        self.run_external_command_execute(
                                            &action_spec,
                                            &selected_entries,
                                        )?;
                                    }
                                }
                            }
                        } else {
                            debug!("No entries available for external action");
                            self.action_tx.send(Action::Error(
                                "No entry available for external action"
                                    .to_string(),
                            ))?;
                        }
                    }
                    _ => {}
                }
                // Check if we're switching from remote control to channel mode
                let was_remote_control =
                    self.television.mode == Mode::RemoteControl;

                // forward action to the television handler
                if let Some(action) = self.television.update(&action)? {
                    self.action_tx.send(action)?;
                }

                // Update watch timer and history
                if was_remote_control
                    && matches!(action, Action::ConfirmSelection)
                    && self.television.mode == Mode::Channel
                    || matches!(action, Action::SwitchToChannel(_))
                {
                    self.update_history();
                    self.restart_watch_timer();
                }
            }
        }
        Ok(ActionOutcome::None)
    }

    fn run_external_command_fork(
        &self,
        action_spec: &ActionSpec,
        entries: &FxHashSet<Entry>,
    ) -> Result<()> {
        // suspend the event loop
        self.event_control_tx
            .send(ControlEvent::Pause)
            .map_err(|e| {
                error!("Failed to suspend event loop: {}", e);
                anyhow::anyhow!("Failed to suspend event loop: {}", e)
            })?;

        // execute the external command in a separate process
        execute_action(action_spec, entries).map_err(|e| {
            error!("Failed to execute external action: {}", e);
            anyhow::anyhow!("Failed to execute external action: {}", e)
        })?;
        // resume the event loop
        self.event_control_tx
            .send(ControlEvent::Resume)
            .map_err(|e| {
                anyhow::anyhow!("Failed to resume event loop: {}", e)
            })?;
        // resume the TUI (after the event loop so as not to produce any artifacts)
        self.render_tx.send(RenderingTask::Resume)?;

        Ok(())
    }

    fn run_external_command_execute(
        &mut self,
        action_spec: &ActionSpec,
        entries: &FxHashSet<Entry>,
    ) -> Result<()> {
        // cleanup
        self.render_tx.send(RenderingTask::Quit)?;
        // wait for the rendering task to finish
        if let Some(rendering_task) = self.render_task.take() {
            while !rendering_task.is_finished() {
                sleep(Duration::from_millis(10));
            }
        }

        execute_action(action_spec, entries).map_err(|e| {
            error!("Failed to execute external action: {}", e);
            anyhow::anyhow!("Failed to execute external action: {}", e)
        })?;

        Ok(())
    }

    /// Maybe select the first entry if there is only one entry available.
    fn maybe_select_1(&mut self) -> Option<ActionOutcome> {
        debug!("Automatically selecting the first entry");
        if let Some(unique_entry) =
            self.television.results_picker.entries.first()
        {
            self.should_quit = true;

            if !self.render_tx.is_closed() {
                let _ = self.render_tx.send(RenderingTask::Quit);
            }

            return Some(ActionOutcome::Entries(FxHashSet::from_iter([
                unique_entry.clone(),
            ])));
        }
        None
    }

    /// Take the first entry from the list regardless of how many entries are available.
    /// If the list is empty, exit with None.
    fn maybe_take_1(&mut self) -> ActionOutcome {
        if let Some(first_entry) =
            self.television.results_picker.entries.first()
        {
            debug!("Automatically taking the first entry");
            self.should_quit = true;

            if !self.render_tx.is_closed() {
                let _ = self.render_tx.send(RenderingTask::Quit);
            }

            ActionOutcome::Entries(FxHashSet::from_iter([first_entry.clone()]))
        } else {
            debug!("No entries available, exiting with None");
            self.should_quit = true;

            if !self.render_tx.is_closed() {
                let _ = self.render_tx.send(RenderingTask::Quit);
            }

            ActionOutcome::None
        }
    }

    /// Determine the TUI mode based on the provided options.
    fn determine_tui_mode(
        height: Option<u16>,
        width: Option<u16>,
        inline: bool,
    ) -> Result<TuiMode> {
        if inline {
            // Inline mode uses all available space at the bottom of the terminal
            Ok(TuiMode::Inline)
        } else if let Some(h) = height {
            // Fixed mode with specified height and width
            Ok(TuiMode::Fixed { width, height: h })
        } else if width.is_some() {
            // error if width is specified without height
            Err(anyhow::anyhow!(
                "TUI viewport: Width cannot be set without a given height."
            ))
        } else {
            // Fullscreen mode
            Ok(TuiMode::Fullscreen)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_tui_mode() {
        // Test inline mode
        assert_eq!(
            App::determine_tui_mode(None, None, true).unwrap(),
            TuiMode::Inline,
            "Passing `inline = true` should return Inline mode"
        );
        assert_eq!(
            App::determine_tui_mode(Some(0), None, true).unwrap(),
            TuiMode::Inline,
            "Passing `inline = true` should return Inline mode"
        );
        assert_eq!(
            App::determine_tui_mode(Some(0), Some(0), true).unwrap(),
            TuiMode::Inline,
            "Passing `inline = true` should return Inline mode"
        );

        // Test fixed mode
        assert_eq!(
            App::determine_tui_mode(Some(20), Some(80), false).unwrap(),
            TuiMode::Fixed {
                width: Some(80),
                height: 20
            }
        );
        assert_eq!(
            App::determine_tui_mode(Some(20), None, false).unwrap(),
            TuiMode::Fixed {
                width: None,
                height: 20
            }
        );

        // Test fullscreen mode
        assert_eq!(
            App::determine_tui_mode(None, None, false).unwrap(),
            TuiMode::Fullscreen
        );

        // Test error case for width without height
        assert!(App::determine_tui_mode(None, Some(80), false).is_err());
    }
}
