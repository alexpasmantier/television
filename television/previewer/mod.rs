use std::{
    cmp::Ordering,
    time::{Duration, Instant},
};

use ansi_to_tui::IntoText;
use anyhow::{Context, Result};
use devicons::FileIcon;
use ratatui::text::Text;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    time::timeout,
};
use tracing::debug;

use crate::{
    channels::{
        entry::Entry,
        prototypes::{CommandSpec, PreviewSpec},
    },
    utils::{
        command::shell_command,
        strings::{
            EMPTY_STRING, ReplaceNonPrintableConfig, replace_non_printable,
        },
    },
};

pub mod state;

pub struct Config {
    request_max_age: Duration,
    job_timeout: Duration,
}

pub const DEFAULT_REQUEST_MAX_AGE: Duration = Duration::from_millis(1000);
pub const DEFAULT_JOB_TIMEOUT: Duration = Duration::from_millis(500);

impl Default for Config {
    fn default() -> Self {
        Self {
            request_max_age: DEFAULT_REQUEST_MAX_AGE,
            job_timeout: DEFAULT_JOB_TIMEOUT,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum Request {
    Preview(Ticket),
    Shutdown,
}

impl PartialOrd for Request {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Request {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            // Shutdown signals always have priority
            (Self::Shutdown, _) => Ordering::Greater,
            (_, Self::Shutdown) => Ordering::Less,
            // Otherwise fall back to ticket age comparison
            (Self::Preview(t1), Self::Preview(t2)) => t1.cmp(t2),
        }
    }
}

#[derive(PartialEq, Eq)]
pub struct Ticket {
    entry: Entry,
    timestamp: Instant,
}

impl PartialOrd for Ticket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ticket {
    fn cmp(&self, other: &Self) -> Ordering {
        self.age().cmp(&other.age())
    }
}

impl Ticket {
    pub fn new(entry: Entry) -> Self {
        Self {
            entry,
            timestamp: Instant::now(),
        }
    }

    fn age(&self) -> Duration {
        Instant::now().duration_since(self.timestamp)
    }
}

#[derive(Debug, Clone)]
pub struct Preview {
    pub title: String,
    // NOTE: this does couple the previewer with ratatui but allows
    // to only parse ansi text once and reuse it in the UI.
    pub content: Text<'static>,
    pub icon: Option<FileIcon>,
    pub total_lines: u16,
    pub footer: String,
}

const DEFAULT_PREVIEW_TITLE: &str = "Select an entry to preview";

impl Default for Preview {
    fn default() -> Self {
        Self {
            title: DEFAULT_PREVIEW_TITLE.to_string(),
            content: Text::from(EMPTY_STRING),
            icon: None,
            total_lines: 1,
            footer: String::new(),
        }
    }
}

impl Preview {
    fn new(
        title: &str,
        content: Text<'static>,
        icon: Option<FileIcon>,
        total_lines: u16,
        footer: String,
    ) -> Self {
        Self {
            title: title.to_string(),
            content,
            icon,
            total_lines,
            footer,
        }
    }
}

pub struct Previewer {
    config: Config,
    // FIXME: maybe use a bounded channel here with a single slot
    requests: UnboundedReceiver<Request>,
    last_job_entry: Option<Entry>,
    preview_spec: PreviewSpec,
    results: UnboundedSender<Preview>,
}

impl Previewer {
    pub fn new(
        preview_command: &PreviewSpec,
        config: Config,
        receiver: UnboundedReceiver<Request>,
        sender: UnboundedSender<Preview>,
    ) -> Self {
        Self {
            config,
            requests: receiver,
            last_job_entry: None,
            preview_spec: preview_command.clone(),
            results: sender,
        }
    }

    pub async fn run(mut self) {
        let mut buffer = Vec::with_capacity(32);
        loop {
            let num = self.requests.recv_many(&mut buffer, 32).await;
            if num > 0 {
                debug!("Previewer received {num} request(s)!");
                // only keep the newest request
                match buffer.drain(..).max().unwrap() {
                    Request::Preview(ticket) => {
                        if ticket.age() > self.config.request_max_age {
                            debug!("Preview request is stale, skipping");
                            continue;
                        }
                        let results_handle = self.results.clone();
                        self.last_job_entry = Some(ticket.entry.clone());
                        // try to execute the preview with a timeout
                        let preview_command =
                            self.preview_spec.command.clone();
                        match timeout(
                            self.config.job_timeout,
                            tokio::spawn(async move {
                                if let Err(e) = try_preview(
                                    &preview_command,
                                    &ticket.entry,
                                    &results_handle,
                                ) {
                                    debug!(
                                        "Failed to generate preview for entry '{}': {}",
                                        ticket.entry.raw,
                                        e
                                    );
                                }
                            }),
                        )
                        .await
                        {
                            Ok(_) => {
                                debug!("Preview job completed successfully");
                            }
                            Err(e) => {
                                debug!("Preview job timeout: {}", e);
                            }
                        }
                    }
                    Request::Shutdown => {
                        debug!(
                            "Received shutdown signal, breaking out of the previewer loop."
                        );
                        break;
                    }
                }
            } else {
                debug!(
                    "Preview request channel closed and no messages left, breaking out of the previewer loop."
                );
                break;
            }
        }
    }
}

pub fn try_preview(
    command: &CommandSpec,
    entry: &Entry,
    results_handle: &UnboundedSender<Preview>,
) -> Result<()> {
    debug!("Preview command: {}", command);

    let formatted_command = command.get_nth(0).format(&entry.raw)?;

    let child =
        shell_command(&formatted_command, command.interactive, &command.env)
            .output()?;

    let preview: Preview = {
        if child.status.success() {
            let mut text = child
                .stdout
                .into_text()
                .unwrap_or_else(|_| Text::from(EMPTY_STRING));

            text.lines.iter_mut().for_each(|line| {
                // replace non-printable characters
                line.spans.iter_mut().for_each(|span| {
                    span.content = replace_non_printable(
                        &span.content.bytes().collect::<Vec<_>>(),
                        &ReplaceNonPrintableConfig::default(),
                    )
                    .0
                    .into();
                });
            });

            let total_lines = u16::try_from(text.lines.len()).unwrap_or(0);

            Preview::new(&entry.raw, text, None, total_lines, String::new())
        } else {
            let mut text = child
                .stderr
                .into_text()
                .unwrap_or_else(|_| Text::from(EMPTY_STRING));

            text.lines.iter_mut().for_each(|line| {
                // replace non-printable characters
                line.spans.iter_mut().for_each(|span| {
                    span.content = replace_non_printable(
                        &span.content.bytes().collect::<Vec<_>>(),
                        &ReplaceNonPrintableConfig::default(),
                    )
                    .0
                    .into();
                });
            });

            let total_lines = u16::try_from(text.lines.len()).unwrap_or(0);

            Preview::new(&entry.raw, text, None, total_lines, String::new())
        }
    };
    results_handle
        .send(preview)
        .with_context(|| "Failed to send preview result to main thread.")
}
