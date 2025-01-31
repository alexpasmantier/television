use anyhow::Result;
use crossterm::terminal::{BeginSynchronizedUpdate, EndSynchronizedUpdate};
use crossterm::{execute, queue};
use ratatui::layout::Rect;
use std::io::{stderr, stdout, LineWriter};
use tracing::{debug, warn};

use tokio::sync::mpsc;

use crate::draw::Ctx;
use crate::{action::Action, draw::draw, tui::Tui};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub enum RenderingTask {
    ClearScreen,
    Render(Ctx),
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
    is_output_tty: bool,
) -> Result<()> {
    let stream = if is_output_tty {
        debug!("Rendering to stdout");
        IoStream::Stdout.to_stream()
    } else {
        debug!("Rendering to stderr");
        IoStream::BufferedStderr.to_stream()
    };
    let mut tui = Tui::new(stream)?;

    debug!("Entering tui");
    tui.enter()?;

    let mut buffer = Vec::with_capacity(256);

    // Rendering loop
    'rendering: while render_rx.recv_many(&mut buffer, 256).await > 0 {
        // deduplicate events
        buffer.sort_unstable();
        buffer.dedup();
        for event in buffer.drain(..) {
            match event {
                RenderingTask::ClearScreen => {
                    tui.terminal.clear()?;
                }
                RenderingTask::Render(context) => {
                    if let Ok(size) = tui.size() {
                        // Ratatui uses `u16`s to encode terminal dimensions and its
                        // content for each terminal cell is stored linearly in a
                        // buffer with a `u16` index which means we can't support
                        // terminal areas larger than `u16::MAX`.
                        if size.width.checked_mul(size.height).is_some() {
                            queue!(stderr(), BeginSynchronizedUpdate).ok();
                            tui.terminal.draw(|frame| {
                                if let Err(err) =
                                    draw(&context, frame, frame.area())
                                {
                                    warn!("Failed to draw: {:?}", err);
                                    let _ = action_tx.send(Action::Error(
                                        format!("Failed to draw: {err:?}"),
                                    ));
                                }
                            })?;
                            execute!(stderr(), EndSynchronizedUpdate).ok();
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
                    debug!("Exiting rendering loop");
                    tui.exit()?;
                    break 'rendering;
                }
            }
        }
    }

    Ok(())
}
