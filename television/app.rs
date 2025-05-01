use rustc_hash::FxHashSet;

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{debug, trace};

use crate::channels::{
    entry::Entry, preview::PreviewType, OnAir, TelevisionChannel,
};
use crate::config::{default_tick_rate, Config};
use crate::keymap::Keymap;
use crate::render::UiState;
use crate::television::{Mode, Television};
use crate::{
    action::Action,
    event::{Event, EventLoop, Key},
    render::{render, RenderingTask},
};

#[allow(clippy::struct_excessive_bools)]
pub struct AppOptions {
    /// Whether the application should use subsring matching instead of fuzzy
    /// matching.
    pub exact: bool,
    /// Whether the application should automatically select the first entry if there is only one
    /// entry available.
    pub select_1: bool,
    /// Whether the application should disable the remote control feature.
    pub no_remote: bool,
    /// Whether the application should disable the help panel feature.
    pub no_help: bool,
    pub tick_rate: f64,
}

impl Default for AppOptions {
    fn default() -> Self {
        Self {
            exact: false,
            select_1: false,
            no_remote: false,
            no_help: false,
            tick_rate: default_tick_rate(),
        }
    }
}

impl AppOptions {
    #[allow(clippy::fn_params_excessive_bools)]
    pub fn new(
        exact: bool,
        select_1: bool,
        no_remote: bool,
        no_help: bool,
        tick_rate: f64,
    ) -> Self {
        Self {
            exact,
            select_1,
            no_remote,
            no_help,
            tick_rate,
        }
    }
}

/// The main application struct that holds the state of the application.
pub struct App {
    keymap: Keymap,
    /// The television instance that handles channels and entries.
    television: Television,
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

impl From<ActionOutcome> for AppOutput {
    fn from(outcome: ActionOutcome) -> Self {
        match outcome {
            ActionOutcome::Entries(entries) => Self {
                selected_entries: Some(entries),
            },
            ActionOutcome::Input(input) => Self {
                selected_entries: Some(FxHashSet::from_iter([Entry::new(
                    input,
                    PreviewType::None,
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
        channel: TelevisionChannel,
        config: Config,
        input: Option<String>,
        options: AppOptions,
    ) -> Self {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let (render_tx, render_rx) = mpsc::unbounded_channel();
        let (_, event_rx) = mpsc::unbounded_channel();
        let (event_abort_tx, _) = mpsc::unbounded_channel();
        let keymap = Keymap::from(&config.keybindings);

        debug!("{:?}", keymap);
        let (ui_state_tx, ui_state_rx) = mpsc::unbounded_channel();
        let television = Television::new(
            action_tx.clone(),
            channel,
            config,
            input,
            options.no_remote,
            options.no_help,
            options.exact,
        );

        Self {
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
        }
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
            let event_loop = EventLoop::new(self.options.tick_rate, true);
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

            // If `self.select_1` is true, the channel is not running, and there is
            // only one entry available, automatically select the first entry.
            if self.options.select_1
                && !self.television.channel.running()
                && self.television.channel.total_count() == 1
            {
                if let Some(outcome) = self.maybe_select_1() {
                    action_outcome = outcome;
                }
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

                return Ok(AppOutput::from(action_outcome));
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
                        self.should_quit = true;
                        self.render_tx.send(RenderingTask::Quit)?;
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
                        if let Some(entries) = self
                            .television
                            .get_selected_entries(Some(Mode::Channel))
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
                // forward action to the television handler
                if let Some(action) = self.television.update(&action)? {
                    self.action_tx.send(action)?;
                };
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
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::channels::stdin::Channel as StdinChannel;

    #[test]
    fn test_maybe_select_1() {
        let mut app = App::new(
            TelevisionChannel::Stdin(StdinChannel::new(PreviewType::None)),
            Config::default(),
            None,
            AppOptions::default(),
        );
        app.television
            .results_picker
            .entries
            .push(Entry::new("test".to_string(), PreviewType::None));
        let outcome = app.maybe_select_1();
        assert!(outcome.is_some());
        assert_eq!(
            outcome.unwrap(),
            ActionOutcome::Entries(FxHashSet::from_iter([Entry::new(
                "test".to_string(),
                PreviewType::None
            )]))
        );
    }
}
