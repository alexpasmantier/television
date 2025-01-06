use std::collections::HashSet;
use std::sync::Arc;

use color_eyre::Result;
use television_screen::mode::Mode;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, info};

use crate::config::{parse_key, Config};
use crate::keymap::Keymap;
use crate::television::Television;
use crate::{
    action::Action,
    event::{Event, EventLoop, Key},
    render::{render, RenderingTask},
};
use television_channels::channels::TelevisionChannel;
use television_channels::entry::Entry;

/// The main application struct that holds the state of the application.
pub struct App {
    keymap: Keymap,
    // maybe move these two into config instead of passing them
    // via the cli?
    tick_rate: f64,
    frame_rate: f64,
    /// The television instance that handles channels and entries.
    television: Arc<Mutex<Television>>,
    /// A flag that indicates whether the application should quit during the next frame.
    should_quit: bool,
    /// A flag that indicates whether the application should suspend during the next frame.
    should_suspend: bool,
    /// A sender channel for actions.
    action_tx: mpsc::UnboundedSender<Action>,
    /// The receiver channel for actions.
    action_rx: mpsc::UnboundedReceiver<Action>,
    /// The receiver channel for events.
    event_rx: mpsc::UnboundedReceiver<Event<Key>>,
    /// A sender channel to abort the event loop.
    event_abort_tx: mpsc::UnboundedSender<()>,
    /// A sender channel for rendering tasks.
    render_tx: mpsc::UnboundedSender<RenderingTask>,
}

/// The outcome of an action.
#[derive(Debug)]
pub enum ActionOutcome {
    Entries(HashSet<Entry>),
    Input(String),
    Passthrough(HashSet<Entry>, String),
    None,
}

/// The result of the application.
#[derive(Debug)]
pub struct AppOutput {
    pub selected_entries: Option<HashSet<Entry>>,
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

impl App {
    pub fn new(
        channel: TelevisionChannel,
        config: Config,
        passthrough_keybindings: &[String],
        input: Option<String>,
    ) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let (render_tx, _) = mpsc::unbounded_channel();
        let (_, event_rx) = mpsc::unbounded_channel();
        let (event_abort_tx, _) = mpsc::unbounded_channel();
        let frame_rate = config.config.frame_rate;
        let tick_rate = config.config.tick_rate;
        let keymap = Keymap::from(&config.keybindings).with_mode_mappings(
            Mode::Channel,
            passthrough_keybindings
                .iter()
                .flat_map(|s| match parse_key(s) {
                    Ok(key) => Ok((key, Action::SelectPassthrough(s.clone()))),
                    Err(e) => Err(e),
                })
                .collect(),
        )?;
        debug!("{:?}", keymap);
        let television =
            Arc::new(Mutex::new(Television::new(channel, config, input)));

        Ok(Self {
            keymap,
            tick_rate,
            frame_rate,
            television,
            should_quit: false,
            should_suspend: false,
            action_tx,
            action_rx,
            event_rx,
            event_abort_tx,
            render_tx,
        })
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
    pub async fn run(&mut self, is_output_tty: bool) -> Result<AppOutput> {
        debug!("Starting backend event loop");
        let event_loop = EventLoop::new(self.tick_rate, true);
        self.event_rx = event_loop.rx;
        self.event_abort_tx = event_loop.abort_tx;

        // Rendering loop
        debug!("Starting rendering loop");
        let (render_tx, render_rx) = mpsc::unbounded_channel();
        self.render_tx = render_tx.clone();
        let action_tx_r = self.action_tx.clone();
        let television_r = self.television.clone();
        let frame_rate = self.frame_rate;
        let rendering_task = tokio::spawn(async move {
            render(
                render_rx,
                action_tx_r,
                television_r,
                frame_rate,
                is_output_tty,
            )
            .await
        });

        // event handling loop
        debug!("Starting event handling loop");
        let action_tx = self.action_tx.clone();
        loop {
            // handle event and convert to action
            if let Some(event) = self.event_rx.recv().await {
                let action = self.convert_event_to_action(event).await;
                action_tx.send(action)?;
            }

            let action_outcome = self.handle_actions().await?;

            if self.should_quit {
                // send a termination signal to the event loop
                self.event_abort_tx.send(())?;

                // wait for the rendering task to finish
                rendering_task.await??;

                return Ok(AppOutput::from(action_outcome));
            }
        }
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
    async fn convert_event_to_action(&self, event: Event<Key>) -> Action {
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
                self.keymap
                    .get(&self.television.lock().await.mode)
                    .and_then(|keymap| keymap.get(&keycode).cloned())
                    .unwrap_or(if let Key::Char(c) = keycode {
                        Action::AddInputChar(c)
                    } else {
                        Action::NoOp
                    })
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
    async fn handle_actions(&mut self) -> Result<ActionOutcome> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != Action::Tick && action != Action::Render {
                debug!("{action:?}");
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
                    self.render_tx.send(RenderingTask::Quit)?;
                    if let Some(entries) = self
                        .television
                        .lock()
                        .await
                        .get_selected_entries(Some(Mode::Channel))
                    {
                        return Ok(ActionOutcome::Entries(entries));
                    }
                    return Ok(ActionOutcome::Input(
                        self.television.lock().await.current_pattern.clone(),
                    ));
                }
                Action::SelectPassthrough(passthrough) => {
                    self.should_quit = true;
                    self.render_tx.send(RenderingTask::Quit)?;
                    if let Some(entries) = self
                        .television
                        .lock()
                        .await
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
                    self.render_tx.send(RenderingTask::Render)?;
                }
                _ => {}
            }
            // forward action to the television handler
            if let Some(action) =
                self.television.lock().await.update(action.clone()).await?
            {
                self.action_tx.send(action)?;
            };
        }
        Ok(ActionOutcome::None)
    }
}
