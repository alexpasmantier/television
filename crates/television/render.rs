use color_eyre::Result;
use ratatui::layout::Rect;
use std::{
    io::{stderr, stdout, LineWriter},
    sync::Arc,
};
use tracing::{debug, warn};

use tokio::{
    select,
    sync::{mpsc, Mutex},
};

use crate::television::Television;
use crate::{action::Action, config::Config, tui::Tui};

#[derive(Debug)]
pub(crate) enum RenderingTask {
    ClearScreen,
    Render,
    Resize(u16, u16),
    Resume,
    Suspend,
    Quit,
}

#[derive(Debug, Clone)]
enum IoStream {
    Stdout,
    BufferedStderr,
}

impl IoStream {
    fn to_stream(&self) -> Box<dyn std::io::Write + Send> {
        match self {
            IoStream::Stdout => Box::new(stdout()),
            IoStream::BufferedStderr => Box::new(LineWriter::new(stderr())),
        }
    }
}

pub async fn render(
    mut render_rx: mpsc::UnboundedReceiver<RenderingTask>,
    action_tx: mpsc::UnboundedSender<Action>,
    config: Config,
    television: Arc<Mutex<Television>>,
    frame_rate: f64,
    is_output_tty: bool,
) -> Result<()> {
    let stream = if is_output_tty {
        debug!("Rendering to stdout");
        IoStream::Stdout.to_stream()
    } else {
        debug!("Rendering to stderr");
        IoStream::BufferedStderr.to_stream()
    };
    let mut tui = Tui::new(stream)?.frame_rate(frame_rate);

    debug!("Entering tui");
    tui.enter()?;

    debug!("Registering action handler and config handler");
    television
        .lock()
        .await
        .register_action_handler(action_tx.clone())?;
    television
        .lock()
        .await
        .register_config_handler(config.clone())?;

    // Rendering loop
    loop {
        select! {
            () = tokio::time::sleep(tokio::time::Duration::from_secs_f64(1.0 / frame_rate)) => {
                action_tx.send(Action::Render)?;
            }
            maybe_task = render_rx.recv() => {
                if let Some(task) = maybe_task {
                    match task {
                        RenderingTask::ClearScreen => {
                            tui.terminal.clear()?;
                        }
                        RenderingTask::Render => {
                            let mut television = television.lock().await;
                            if let Ok(size) = tui.size() {
                                // Ratatui uses u16s to encode terminal dimensions and its
                                // content for each terminal cell is stored linearly in a
                                // buffer with a u16 index which means we can't support
                                // terminal areas larger than u16::MAX.
                                if size.width.checked_mul(size.height).is_some() {
                                    tui.terminal.draw(|frame| {
                                        if let Err(err) = television.draw(frame, frame.area()) {
                                            warn!("Failed to draw: {:?}", err);
                                            let _ = action_tx
                                                .send(Action::Error(format!("Failed to draw: {err:?}")));
                                        }
                                    })?;

                                } else {
                                    warn!("Terminal area too large");
                                }
                            }
                        }
                        RenderingTask::Resize(w, h) => {
                            tui.resize(Rect::new(0, 0, w, h))?;
                            action_tx.send(Action::Render)?;
                        }
                        RenderingTask::Suspend => {
                            tui.suspend()?;
                            action_tx.send(Action::Resume)?;
                            action_tx.send(Action::ClearScreen)?;
                            tui.enter()?;
                        }
                        RenderingTask::Resume => {
                            tui.enter()?;
                        }
                        RenderingTask::Quit => {
                            tui.exit()?;
                            break Ok(());
                        }
                    }
                }
            }
        }
    }
}
