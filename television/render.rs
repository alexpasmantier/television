use anyhow::Result;
use crossterm::terminal::{BeginSynchronizedUpdate, EndSynchronizedUpdate};
use crossterm::{execute, queue};
use ratatui::layout::Rect;
use std::io::{stderr, stdout, LineWriter};
use tracing::{debug, warn};

use tokio::sync::mpsc;

use crate::draw::Ctx;
use crate::screen::layout::Layout;
use crate::{action::Action, draw::draw, tui::Tui};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub enum RenderingTask {
    ClearScreen,
    Render(Box<Ctx>),
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

#[derive(Default)]
/// The state of the UI after rendering.
///
/// This struct is returned by the UI thread to the main thread after each rendering cycle.
/// It contains information that the main thread might be able to exploit to make certain
/// decisions and optimizations.
pub struct UiState {
    pub layout: Layout,
}

impl UiState {
    pub fn new(layout: Layout) -> Self {
        Self { layout }
    }
}

/// The main UI rendering task loop.
///
/// This function is responsible for rendering the UI based on the rendering tasks it receives from
/// the main thread via `render_rx`.
///
/// This has a handle to the main action queue `action_tx` (for things like self-triggering
/// subsequent rendering instructions) and the UI state queue `ui_state_tx` to send back the layout
/// of the UI after each rendering cycle to the main thread to help make decisions and
/// optimizations.
///
/// When starting the rendering loop, a choice is made to either render to stdout or stderr based
/// on if the output is believed to be a TTY or not.
pub async fn render(
    mut render_rx: mpsc::UnboundedReceiver<RenderingTask>,
    action_tx: mpsc::UnboundedSender<Action>,
    ui_state_tx: mpsc::UnboundedSender<UiState>,
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
                                match draw(&context, frame, frame.area()) {
                                    Ok(layout) => {
                                        if layout != context.layout {
                                            let _ = ui_state_tx
                                                .send(UiState::new(layout));
                                        }
                                    }
                                    Err(err) => {
                                        warn!("Failed to draw: {:?}", err);
                                        let _ = action_tx.send(Action::Error(
                                            format!("Failed to draw: {err:?}"),
                                        ));
                                    }
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
