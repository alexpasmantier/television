/**

                                               The general idea
┌──────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                                      │
│       rendering thread                        event thread                       main thread         │
│                                                                                                      │
│              │                                      │                                  │             │
│                                                                                                      │
│              │                                      │                                  │             │
│                                                                                                      │
│              │                                      │                                  │             │
│                                             ┌───────┴───────┐                                        │
│              │                              │               │                          │             │
│                                             │ receive event │                                        │
│              │                              │               │                          │             │
│                                             └───────┬───────┘                                        │
│              │                                      │                                  │             │
│                                                     ▼                                                │
│              │                             ┌──────────────────┐             ┌──────────┴─────────┐   │
│                                            │                  │             │                    │   │
│              │                             │send on `event_rx`├────────────►│ receive `event_rx` │   │
│                                            │                  │             │                    │   │
│              │                             └──────────────────┘             └──────────┬─────────┘   │
│                                                                                        │             │
│              │                                                                         ▼             │
│                                                                             ┌────────────────────┐   │
│              │                                                              │    map to action   │   │
│                                                                             └──────────┬─────────┘   │
│              │                                                                         ▼             │
│                                                                             ┌────────────────────┐   │
│              │                                                              │ send on `action_tx`│   │
│                                                                             └──────────┬─────────┘   │
│              │                                                                                       │
│                                                                                                      │
│              │                                                              ┌──────────┴─────────┐   │
│                                                                             │ receive `action_rx`│   │
│              │                                                              └──────────┬─────────┘   │
│  ┌───────────┴────────────┐                                                            ▼             │
│  │                        │                                                 ┌────────────────────┐   │
│  │  receive `render_rx`   │◄────────────────────────────────────────────────┤  dispatch action   │   │
│  │                        │                                                 └──────────┬─────────┘   │
│  └───────────┬────────────┘                                                            │             │
│              │                                                                         │             │
│              ▼                                                                         ▼             │
│  ┌────────────────────────┐                                                 ┌────────────────────┐   │
│  │   render components    │                                                 │  update components │   │
│  └────────────────────────┘                                                 └────────────────────┘   │
│                                                                                                      │
└──────────────────────────────────────────────────────────────────────────────────────────────────────┘

*/
use std::sync::Arc;

use color_eyre::Result;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, info};

use crate::channels::{CliTvChannel, TelevisionChannel};
use crate::television::{Mode, Television};
use crate::{
    action::Action,
    config::Config,
    entry::Entry,
    event::{Event, EventLoop, Key},
    render::{render, RenderingTask},
};

pub struct App {
    config: Config,
    // maybe move these two into config instead of passing them
    // via the cli?
    tick_rate: f64,
    frame_rate: f64,
    television: Arc<Mutex<Television>>,
    should_quit: bool,
    should_suspend: bool,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    event_rx: mpsc::UnboundedReceiver<Event<Key>>,
    event_abort_tx: mpsc::UnboundedSender<()>,
    render_tx: mpsc::UnboundedSender<RenderingTask>,
}

impl App {
    pub fn new(
        channel: TelevisionChannel,
        tick_rate: f64,
        frame_rate: f64,
    ) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let (render_tx, _) = mpsc::unbounded_channel();
        let (_, event_rx) = mpsc::unbounded_channel();
        let (event_abort_tx, _) = mpsc::unbounded_channel();
        let television = Arc::new(Mutex::new(Television::new(channel)));

        Ok(Self {
            tick_rate,
            frame_rate,
            television,
            should_quit: false,
            should_suspend: false,
            config: Config::new()?,
            action_tx,
            action_rx,
            event_rx,
            event_abort_tx,
            render_tx,
        })
    }

    pub async fn run(&mut self, is_output_tty: bool) -> Result<Option<Entry>> {
        info!("Starting backend event loop");
        let event_loop = EventLoop::new(self.tick_rate, true);
        self.event_rx = event_loop.rx;
        self.event_abort_tx = event_loop.abort_tx;

        // Rendering loop
        debug!("Starting rendering loop");
        let (render_tx, render_rx) = mpsc::unbounded_channel();
        self.render_tx = render_tx.clone();
        let action_tx_r = self.action_tx.clone();
        let config_r = self.config.clone();
        let television_r = self.television.clone();
        let frame_rate = self.frame_rate;
        let rendering_task = tokio::spawn(async move {
            render(
                render_rx,
                action_tx_r,
                config_r,
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

            let maybe_selected = self.handle_actions().await?;

            if self.should_quit {
                // send a termination signal to the event loop
                self.event_abort_tx.send(())?;

                // wait for the rendering task to finish
                rendering_task.await??;

                return Ok(maybe_selected);
            }
        }
    }

    async fn convert_event_to_action(&self, event: Event<Key>) -> Action {
        match event {
            Event::Input(keycode) => {
                info!("{:?}", keycode);
                // text input events
                match keycode {
                    Key::Backspace => return Action::DeletePrevChar,
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
                self.config
                    .keybindings
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

    async fn handle_actions(&mut self) -> Result<Option<Entry>> {
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
                    return Ok(self
                        .television
                        .lock()
                        .await
                        .get_selected_entry(Some(Mode::Channel)));
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
            if let Some(action) =
                self.television.lock().await.update(action.clone()).await?
            {
                self.action_tx.send(action)?;
            };
        }
        Ok(None)
    }
}
