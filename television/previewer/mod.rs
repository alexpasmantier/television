use std::{
    cmp::Ordering,
    sync::Arc,
    time::{Duration, Instant},
};

use ansi_to_tui::IntoText;
use anyhow::{Context, Result};
use parking_lot::Mutex;
use ratatui::text::Text;
use tokio::process::Command as TokioCommand;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::spawn,
    time::timeout,
};
use tracing::{debug, trace, warn};

use crate::{
    channels::{
        entry::Entry,
        prototypes::{CommandSpec, Template},
    },
    previewer::cache::Cache,
    utils::{
        command::shell_command,
        strings::{
            EMPTY_STRING, ReplaceNonPrintableConfig,
            replace_non_printable_bulk,
        },
    },
};

mod cache;
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

#[allow(
    clippy::large_enum_variant,
    reason = "requests are almost exclusively preview jobs"
)]
#[derive(PartialEq, Eq)]
pub enum Request {
    Preview(Ticket),
    Shutdown,
    CycleCommand,
}

impl PartialOrd for Request {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Request {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            // Shutdown/Cycle signals always have priority
            (Self::Shutdown | Self::CycleCommand, _) => Ordering::Greater,
            (_, Self::Shutdown | Self::CycleCommand) => Ordering::Less,
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Preview {
    pub entry_raw: String,
    pub formatted_command: String,
    pub title: String,
    // NOTE: this does couple the previewer with ratatui but allows
    // to only parse ansi text once and reuse it in the UI.
    pub content: Text<'static>,
    pub target_line: Option<u16>,
    pub total_lines: u16,
    pub footer: Option<String>,
}

const DEFAULT_PREVIEW_TITLE: &str = "Select an entry to preview";

impl Default for Preview {
    fn default() -> Self {
        Self {
            entry_raw: EMPTY_STRING.to_string(),
            formatted_command: EMPTY_STRING.to_string(),
            title: DEFAULT_PREVIEW_TITLE.to_string(),
            content: Text::from(EMPTY_STRING),
            target_line: None,
            total_lines: 1,
            footer: None,
        }
    }
}

impl Preview {
    fn new(
        entry_raw: String,
        formatted_command: String,
        title: &str,
        displayable_content: Text<'static>,
        line_number: Option<u16>,
        total_lines: u16,
        footer: Option<String>,
    ) -> Self {
        Self {
            entry_raw,
            formatted_command,
            title: title.to_string(),
            content: displayable_content,
            target_line: line_number,
            total_lines,
            footer,
        }
    }
}

pub struct Previewer {
    config: Config,
    requests_tx: UnboundedSender<Request>,
    requests_rx: UnboundedReceiver<Request>,
    last_job_entry: Option<Entry>,
    command: CommandSpec,
    /// The current cycle index for commands with multiple variants.
    cycle_index: usize,
    title_template: Option<Template>,
    footer_template: Option<Template>,
    offset_expr: Option<Template>,
    results: UnboundedSender<Preview>,
    cache: Option<Arc<Mutex<Cache>>>,
}

impl Previewer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        command: &CommandSpec,
        offset_expr: Option<Template>,
        title_template: Option<Template>,
        footer_template: Option<Template>,
        config: Config,
        requests_rx: UnboundedReceiver<Request>,
        requests_tx: UnboundedSender<Request>,
        results_tx: UnboundedSender<Preview>,
        cache: bool,
    ) -> Self {
        let cache = if cache {
            Some(Arc::new(Mutex::new(Cache::default())))
        } else {
            None
        };
        Self {
            config,
            requests_tx,
            requests_rx,
            last_job_entry: None,
            command: command.clone(),
            cycle_index: 0,
            title_template,
            footer_template,
            offset_expr,
            results: results_tx,
            cache,
        }
    }

    pub async fn run(mut self) {
        let mut buffer = Vec::with_capacity(32);
        loop {
            let num = self.requests_rx.recv_many(&mut buffer, 32).await;
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
                        let preview_command = self.command.clone();
                        let cache = self.cache.clone();
                        let offset_expr = self.offset_expr.clone();
                        let title_template = self.title_template.clone();
                        let footer_template = self.footer_template.clone();
                        let job = spawn(try_preview(
                            preview_command,
                            self.cycle_index,
                            title_template,
                            footer_template,
                            offset_expr,
                            ticket.entry,
                            results_handle,
                            cache,
                        ));
                        match timeout(self.config.job_timeout, job).await {
                            Ok(Ok(Ok(()))) => {
                                trace!("Preview job completed successfully");
                            }
                            Ok(Ok(Err(e))) => warn!(
                                "Failed to generate preview for entry '{}': {}",
                                &self.last_job_entry.clone().unwrap().raw,
                                e
                            ),
                            Ok(Err(join_err)) => {
                                warn!(
                                    "Preview join error for '{}': {}",
                                    self.last_job_entry.clone().unwrap().raw,
                                    join_err
                                );
                            }
                            Err(e) => {
                                warn!("Preview job timeout: {}", e);
                            }
                        }
                    }
                    Request::CycleCommand => {
                        trace!("Cycling preview command.");
                        self.cycle_command();
                    }
                    Request::Shutdown => {
                        trace!(
                            "Received shutdown signal, breaking out of the previewer loop."
                        );
                        break;
                    }
                }
            } else {
                trace!(
                    "Preview request channel closed and no messages left, breaking out of the previewer loop."
                );
                break;
            }
        }
    }

    pub fn cycle_command(&mut self) {
        self.cycle_index = (self.cycle_index + 1) % self.command.inner.len();
        // re-request preview for the last entry if any
        if let Some(entry) = &self.last_job_entry {
            let _ = self
                .requests_tx
                .send(Request::Preview(Ticket::new(entry.clone())));
        }
    }
}

fn sanitize_text(text: &mut Text<'static>) {
    text.lines.iter_mut().for_each(|line| {
        // replace non-printable characters
        line.spans.iter_mut().for_each(|span| {
            span.content = replace_non_printable_bulk(
                &span.content,
                &ReplaceNonPrintableConfig::default(),
            )
            .0
            .into_owned()
            .into();
        });
    });
}

fn build_preview_from_text(
    formatted_command: &str,
    entry: &Entry,
    text: Text<'static>,
    title_template: Option<&Template>,
    footer_template: Option<&Template>,
    offset_expr: Option<&Template>,
) -> Result<Preview> {
    let total_lines = u16::try_from(text.lines.len()).unwrap_or(0);

    // try to extract a line number from the offset expression if provided
    let line_number = if let Some(offset_expr) = offset_expr.as_ref() {
        let offset_str = offset_expr.format(&entry.raw)?;
        offset_str.parse::<u16>().ok()
    } else {
        None
    };

    let title = if let Some(title_template) = title_template.as_ref() {
        title_template.format(&entry.raw)?
    } else {
        entry.display().to_string()
    };
    let footer = if let Some(footer_template) = footer_template.as_ref() {
        Some(footer_template.format(&entry.raw)?)
    } else {
        None
    };

    Ok(Preview::new(
        entry.raw.clone(),
        formatted_command.to_string(),
        &title,
        text,
        line_number,
        total_lines,
        footer,
    ))
}

#[allow(clippy::too_many_arguments)]
pub async fn try_preview(
    command: CommandSpec,
    cycle_index: usize,
    title_template: Option<Template>,
    footer_template: Option<Template>,
    offset_expr: Option<Template>,
    entry: Entry,
    results_handle: UnboundedSender<Preview>,
    cache: Option<Arc<Mutex<Cache>>>,
) -> Result<()> {
    let formatted_command = command.get_nth(cycle_index).format(&entry.raw)?;

    // Check if the entry is already cached
    if let Some(cache) = &cache
        && let Some(text) = cache.lock().get(&formatted_command)
    {
        trace!("Preview for command '{}' found in cache", formatted_command);
        let preview = build_preview_from_text(
            &formatted_command,
            &entry,
            text,
            title_template.as_ref(),
            footer_template.as_ref(),
            offset_expr.as_ref(),
        )?;
        results_handle.send(preview).with_context(
            || "Failed to send cached preview result to main thread.",
        )?;
        return Ok(());
    }

    debug!("Executing preview command: {}", &formatted_command);
    let command =
        shell_command(&formatted_command, command.interactive, &command.env);

    let child = TokioCommand::from(command).output().await?;

    let mut text = if child.status.success() {
        child
            .stdout
            .into_text()
            .unwrap_or_else(|_| Text::from(EMPTY_STRING))
    } else {
        child
            .stderr
            .into_text()
            .unwrap_or_else(|_| Text::from(EMPTY_STRING))
    };

    sanitize_text(&mut text);

    let preview = if let Some(cache) = &cache {
        let preview = build_preview_from_text(
            &formatted_command,
            &entry,
            text.clone(),
            title_template.as_ref(),
            footer_template.as_ref(),
            offset_expr.as_ref(),
        )?;
        cache.lock().insert(&formatted_command, &text);
        preview
    } else {
        build_preview_from_text(
            &formatted_command,
            &entry,
            text,
            title_template.as_ref(),
            footer_template.as_ref(),
            offset_expr.as_ref(),
        )?
    };
    // FIXME: ... and just send an Arc here as well
    results_handle
        .send(preview)
        .with_context(|| "Failed to send preview result to main thread.")
}
