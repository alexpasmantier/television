use rustc_hash::FxHashSet;

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{debug, info, trace};

use crate::channels::entry::Entry;
use crate::channels::TelevisionChannel;
use crate::config::{
    merge_keybindings, parse_key, Binding, Config, KeyBindings,
};
use crate::keymap::Keymap;
use crate::render::UiState;
use crate::television::{Mode, Television};
use crate::{
    action::Action,
    event::{Event, EventLoop, Key},
    render::{render, RenderingTask},
};

/// The main application struct that holds the state of the application.
pub struct App {
    keymap: Keymap,
    // maybe move these two into config instead of passing them
    // via the cli?
    tick_rate: f64,
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
}

/// The outcome of an action.
#[derive(Debug)]
pub enum ActionOutcome {
    Entries(FxHashSet<Entry>),
    Input(String),
    Passthrough(FxHashSet<Entry>, String),
    None,
}

/// The result of the application.
#[derive(Debug)]
pub struct AppOutput {
    pub selected_entries: Option<FxHashSet<Entry>>,
    pub passthrough: Option<String>,
}

impl From<ActionOutcome> for AppOutput {
    fn from(outcome: ActionOutcome) -> Self {
        match outcome {
            ActionOutcome::Entries(entries) => Self {
                selected_entries: Some(entries),
                passthrough: None,
            },
            ActionOutcome::Input(input) => Self {
                selected_entries: None,
                passthrough: Some(input),
            },
            ActionOutcome::Passthrough(entries, key) => Self {
                selected_entries: Some(entries),
                passthrough: Some(key),
            },
            ActionOutcome::None => Self {
                selected_entries: None,
                passthrough: None,
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
        passthrough_keybindings: &[String],
        input: Option<String>,
    ) -> Self {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let (render_tx, render_rx) = mpsc::unbounded_channel();
        let (_, event_rx) = mpsc::unbounded_channel();
        let (event_abort_tx, _) = mpsc::unbounded_channel();
        let tick_rate = config.config.tick_rate;
        let keybindings = merge_keybindings(config.keybindings.clone(), {
            &KeyBindings::from(passthrough_keybindings.iter().filter_map(
                |s| match parse_key(s) {
                    Ok(key) => Some((
                        Action::SelectPassthrough(s.to_string()),
                        Binding::SingleKey(key),
                    )),
                    Err(e) => {
                        debug!("Failed to parse keybinding: {}", e);
                        None
                    }
                },
            ))
        });
        let keymap = Keymap::from(&keybindings);

        debug!("{:?}", keymap);
        let (ui_state_tx, ui_state_rx) = mpsc::unbounded_channel();
        let television =
            Television::new(action_tx.clone(), channel, config, input);

        Self {
            keymap,
            tick_rate,
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
        if !headless {
            debug!("Starting backend event loop");
            let event_loop = EventLoop::new(self.tick_rate, true);
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

        // event handling loop
        debug!("Starting event handling loop");
        let action_tx = self.action_tx.clone();
        let mut event_buf = Vec::with_capacity(EVENT_BUF_SIZE);
        let mut action_buf = Vec::with_capacity(ACTION_BUF_SIZE);
        loop {
            // handle event and convert to action
            if self
                .event_rx
                .recv_many(&mut event_buf, EVENT_BUF_SIZE)
                .await
                > 0
            {
                for event in event_buf.drain(..) {
                    let action = self.convert_event_to_action(event);
                    action_tx.send(action)?;
                }
            }
            let action_outcome = self.handle_actions(&mut action_buf).await?;

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
    fn convert_event_to_action(&self, event: Event<Key>) -> Action {
        match event {
            Event::Input(keycode) => {
                info!("{:?}", keycode);
                // text input events
                match keycode {
                    Key::Backspace => return Action::DeletePrevChar,
                    Key::Ctrl('w') => return Action::DeletePrevWord,
                    Key::Delete => return Action::DeleteNextChar,
                    Key::Left => return Action::GoToPrevChar,
                    Key::Right => return Action::GoToNextChar,
                    Key::Home | Key::Ctrl('a') => {
                        return Action::GoToInputStart
                    }
                    Key::End | Key::Ctrl('e') => return Action::GoToInputEnd,
                    Key::Char(c) => return Action::AddInputChar(c),
                    _ => {}
                }
                // get action based on keybindings
                self.keymap.get(&keycode).cloned().unwrap_or(
                    if let Key::Char(c) = keycode {
                        Action::AddInputChar(c)
                    } else {
                        Action::NoOp
                    },
                )
            }
            // terminal events
            Event::Tick => Action::Tick,
            Event::Resize(x, y) => Action::Resize(x, y),
            Event::FocusGained => Action::Resume,
            Event::FocusLost => Action::Suspend,
            Event::Closed => Action::NoOp,
        }
    }

    /// Handle actions.
    ///
    /// This function will handle all actions that are sent to the application.
    /// The function will return the selected entry if the application is exited.
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
                    Action::SelectPassthrough(passthrough) => {
                        self.should_quit = true;
                        self.render_tx.send(RenderingTask::Quit)?;
                        if let Some(entries) = self
                            .television
                            .get_selected_entries(Some(Mode::Channel))
                        {
                            return Ok(ActionOutcome::Passthrough(
                                entries,
                                passthrough,
                            ));
                        }
                        return Ok(ActionOutcome::None);
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
}
