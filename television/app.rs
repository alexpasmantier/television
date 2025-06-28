use rustc_hash::FxHashSet;

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{debug, trace};

use crate::{
    action::Action,
    cable::Cable,
    channels::{entry::Entry, prototypes::ChannelPrototype},
    config::{Config, DEFAULT_PREVIEW_SIZE, default_tick_rate},
    event::{Event, EventLoop, Key},
    keymap::Keymap,
    render::{RenderingTask, UiState, render},
    television::{Mode, Television},
};

#[allow(clippy::struct_excessive_bools)]
pub struct AppOptions {
    /// Whether the application should use subsring matching instead of fuzzy
    /// matching.
    pub exact: bool,
    /// Whether the application should automatically select the first entry if there is only one
    /// entry available.
    pub select_1: bool,
    /// Whether the application should take the first entry after the channel has finished loading.
    pub take_1: bool,
    /// Whether the application should take the first entry as soon as it becomes available.
    pub take_1_fast: bool,
    /// Whether the application should disable the remote control feature.
    pub no_remote: bool,
    /// Whether the application should disable the preview panel feature.
    pub no_preview: bool,
    /// The size of the preview panel in lines/columns.
    pub preview_size: Option<u16>,
    /// The tick rate of the application in ticks per second (Hz).
    pub tick_rate: f64,
    /// Watch interval in seconds for automatic reloading (0 = disabled).
    pub watch_interval: f64,
}

impl Default for AppOptions {
    fn default() -> Self {
        Self {
            exact: false,
            select_1: false,
            take_1: false,
            take_1_fast: false,
            no_remote: false,
            no_preview: false,
            preview_size: Some(DEFAULT_PREVIEW_SIZE),
            tick_rate: default_tick_rate(),
            watch_interval: 0.0,
        }
    }
}

impl AppOptions {
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::fn_params_excessive_bools)]
    pub fn new(
        exact: bool,
        select_1: bool,
        take_1: bool,
        take_1_fast: bool,
        no_remote: bool,
        no_preview: bool,
        preview_size: Option<u16>,
        tick_rate: f64,
        watch_interval: f64,
    ) -> Self {
        Self {
            exact,
            select_1,
            take_1,
            take_1_fast,
            no_remote,
            no_preview,
            preview_size,
            tick_rate,
            watch_interval,
        }
    }
}

/// The main application struct that holds the state of the application.
pub struct App {
    keymap: Keymap,
    /// The television instance that handles channels and entries.
    pub television: Television,
    /// A flag that indicates whether the application should quit during the next frame.
    should_quit: bool,
    /// A flag that indicates whether the application should suspend during the next frame.
    should_suspend: bool,
    /// A sender channel for actions.
    ///
    /// This is made public so that tests for instance can send actions to a running application.
    pub action_tx: mpsc::UnboundedSender<Action>,
    /// The receiver channel for actions.
    action_rx: mpsc::UnboundedReceiver<Action>,
    /// The receiver channel for events.
    event_rx: mpsc::UnboundedReceiver<Event<Key>>,
    /// A sender channel to abort the event loop.
    event_abort_tx: mpsc::UnboundedSender<()>,
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
    options: AppOptions,
    /// Watch timer task handle for periodic reloading
    watch_timer_task: Option<tokio::task::JoinHandle<()>>,
}

/// The outcome of an action.
#[derive(Debug, PartialEq)]
pub enum ActionOutcome {
    Entries(FxHashSet<Entry>),
    Input(String),
    None,
}

/// The result of the application.
#[derive(Debug)]
pub struct AppOutput {
    pub selected_entries: Option<FxHashSet<Entry>>,
}

impl AppOutput {
    pub fn new(action_outcome: ActionOutcome) -> Self {
        match action_outcome {
            ActionOutcome::Entries(entries) => Self {
                selected_entries: Some(entries),
            },
            ActionOutcome::Input(input) => Self {
                selected_entries: Some(FxHashSet::from_iter([Entry::new(
                    input,
                )])),
            },
            ActionOutcome::None => Self {
                selected_entries: None,
            },
        }
    }
}

const EVENT_BUF_SIZE: usize = 4;
const ACTION_BUF_SIZE: usize = 8;

impl App {
    pub fn new(
        channel_prototype: ChannelPrototype,
        config: Config,
        input: Option<String>,
        options: AppOptions,
        cable_channels: Cable,
    ) -> Self {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let (render_tx, render_rx) = mpsc::unbounded_channel();
        let (_, event_rx) = mpsc::unbounded_channel();
        let (event_abort_tx, _) = mpsc::unbounded_channel();

        let (ui_state_tx, ui_state_rx) = mpsc::unbounded_channel();
        let television = Television::new(
            action_tx.clone(),
            channel_prototype,
            config,
            input,
            options.no_remote,
            options.no_preview,
            options.preview_size,
            options.exact,
            cable_channels,
        );

        // Create keymap from the merged config that includes channel prototype keybindings
        let keymap = Keymap::from(&television.config.keybindings);
        debug!("{:?}", keymap);

        let mut app = Self {
            keymap,
            television,
            should_quit: false,
            should_suspend: false,
            action_tx,
            action_rx,
            event_rx,
            event_abort_tx,
            render_tx,
            render_rx,
            ui_state_rx,
            ui_state_tx,
            render_task: None,
            options,
            watch_timer_task: None,
        };

        // populate keymap by going through all cable channels and adding their shortcuts if remote
        // control is present
        app.update_keymap();

        app
    }

    /// Check if the watch timer is currently active.
    fn watch_active(&self) -> bool {
        self.watch_timer_task.is_some()
    }

    /// Start the watch timer if watch interval is configured
    fn start_watch_timer(&mut self) {
        if self.options.watch_interval > 0.0 && !self.watch_active() {
            let action_tx = self.action_tx.clone();
            let interval = std::time::Duration::from_secs_f64(
                self.options.watch_interval,
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

    /// Restart the watch timer based on current channel's watch configuration
    fn restart_watch_timer(&mut self) {
        self.stop_watch_timer();
        // Update the watch interval from the current channel prototype
        self.options.watch_interval = self.television.channel_prototype.watch;
        self.start_watch_timer();
    }

    /// Update the keymap from the television's current config.
    ///
    /// This should be called whenever the channel changes to ensure the keymap includes the
    /// channel's keybindings and shortcuts for all other channels if the remote control is
    /// enabled.
    fn update_keymap(&mut self) {
        let mut keymap = Keymap::from(&self.television.config.keybindings);

        // Add channel specific shortcuts
        if let Some(rc) = &self.television.remote_control {
            keymap.merge(&rc.cable_channels.shortcut_keymap());
        }

        self.keymap = keymap;
        debug!("Updated keymap (with shortcuts): {:?}", self.keymap);
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
        // Event loop
        if !headless {
            debug!("Starting backend event loop");
            let event_loop = EventLoop::new(self.options.tick_rate);
            self.event_rx = event_loop.rx;
            self.event_abort_tx = event_loop.abort_tx;
        }

        // Rendering loop
        if !headless {
            debug!("Starting rendering loop");
            let (render_tx, render_rx) = mpsc::unbounded_channel();
            self.render_tx = render_tx.clone();
            let ui_state_tx = self.ui_state_tx.clone();
            let action_tx_r = self.action_tx.clone();
            self.render_task = Some(tokio::spawn(async move {
                render(render_rx, action_tx_r, ui_state_tx, is_output_tty)
                    .await
            }));
            self.action_tx
                .send(Action::Render)
                .expect("Unable to send init render action.");
        }

        // Start watch timer if configured
        self.start_watch_timer();

        // Main loop
        debug!("Starting event handling loop");
        let action_tx = self.action_tx.clone();
        let mut event_buf = Vec::with_capacity(EVENT_BUF_SIZE);
        let mut action_buf = Vec::with_capacity(ACTION_BUF_SIZE);
        let mut action_outcome;

        loop {
            // handle event and convert to action
            if self
                .event_rx
                .recv_many(&mut event_buf, EVENT_BUF_SIZE)
                .await
                > 0
            {
                for event in event_buf.drain(..) {
                    if let Some(action) = self.convert_event_to_action(event) {
                        if action != Action::Tick {
                            debug!("Queuing new action: {action:?}");
                        }
                        action_tx.send(action)?;
                    }
                }
            }
            // It's important that this shouldn't block if no actions are available
            action_outcome = self.handle_actions(&mut action_buf).await?;

            if self.options.select_1
                && !self.television.channel.running()
                && self.television.channel.total_count() == 1
            {
                // If `self.select_1` is true, the channel is not running, and there is
                // only one entry available, automatically select the first entry.
                if let Some(outcome) = self.maybe_select_1() {
                    action_outcome = outcome;
                }
            } else if self.options.take_1 && !self.television.channel.running()
            {
                // If `take_1` is true and the channel has finished loading,
                // automatically take the first entry regardless of count.
                // If there are no entries, exit with None.
                action_outcome = self.maybe_take_1();
            } else if self.options.take_1_fast {
                // If `take_1_fast` is true, immediately take the first entry without
                // waiting for loading to finish. If there are no entries, exit with None.
                action_outcome = self.maybe_take_1();
            }

            if self.should_quit {
                // send a termination signal to the event loop
                if !headless {
                    self.event_abort_tx.send(())?;
                }

                // wait for the rendering task to finish
                if let Some(rendering_task) = self.render_task.take() {
                    rendering_task.await??;
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
    /// * `event` - The event to convert to an action.
    ///
    /// # Returns
    /// The action that corresponds to the given event.
    fn convert_event_to_action(&self, event: Event<Key>) -> Option<Action> {
        let action = match event {
            Event::Input(keycode) => {
                // get action based on keybindings
                if let Some(action) = self.keymap.get(&keycode) {
                    debug!("Keybinding found: {action:?}");
                    action.clone()
                } else {
                    // text input events
                    match keycode {
                        Key::Backspace => Action::DeletePrevChar,
                        Key::Ctrl('w') => Action::DeletePrevWord,
                        Key::Ctrl('u') => Action::DeleteLine,
                        Key::Delete => Action::DeleteNextChar,
                        Key::Left => Action::GoToPrevChar,
                        Key::Right => Action::GoToNextChar,
                        Key::Home | Key::Ctrl('a') => Action::GoToInputStart,
                        Key::End | Key::Ctrl('e') => Action::GoToInputEnd,
                        Key::Char(c) => Action::AddInputChar(c),
                        _ => Action::NoOp,
                    }
                }
            }
            // terminal events
            Event::Tick => Action::Tick,
            Event::Resize(x, y) => Action::Resize(x, y),
            Event::FocusGained => Action::Resume,
            Event::FocusLost => Action::Suspend,
            Event::Closed => Action::NoOp,
        };

        if action != Action::Tick {
            trace!("Converted event to action: {action:?}");
        }

        if action == Action::NoOp {
            None
        } else {
            Some(action)
        }
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
                        self.should_suspend = true;
                        self.render_tx.send(RenderingTask::Suspend)?;
                    }
                    Action::Resume => {
                        self.should_suspend = false;
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
                            return Ok(ActionOutcome::Entries(entries));
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
                    _ => {}
                }
                // Check if we're switching from remote control to channel mode
                let was_remote_control =
                    self.television.mode == Mode::RemoteControl;

                // forward action to the television handler
                if let Some(action) = self.television.update(&action)? {
                    self.action_tx.send(action)?;
                }

                // Update keymap if channel changed (remote control to channel mode transition)
                // This ensures channel-specific keybindings are properly loaded
                if was_remote_control
                    && matches!(action, Action::ConfirmSelection)
                    && self.television.mode == Mode::Channel
                {
                    self.update_keymap();
                    self.restart_watch_timer();
                } else if matches!(action, Action::SwitchToChannel(_)) {
                    // Channel changed via shortcut, refresh keymap and watch timer
                    self.update_keymap();
                    self.restart_watch_timer();
                }
            }
        }
        Ok(ActionOutcome::None)
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
}
