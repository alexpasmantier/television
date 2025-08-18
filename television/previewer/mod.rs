use crate::{
    channels::{
        entry::Entry,
        prototypes::{CommandSpec, Template},
    },
    previewer::cache::Cache,
    selector::process_entries,
    utils::{
        command::shell_command,
        strings::{
            EMPTY_STRING, ReplaceNonPrintableConfig, replace_non_printable,
        },
    },
};
use ansi_to_tui::IntoText;
use anyhow::{Context, Result};
use parking_lot::Mutex;
use ratatui::text::Text;
use std::{
    cmp::Ordering,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    time::timeout,
};
use tracing::debug;

mod cache;
pub mod state;

#[derive(Clone)]
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
    entries: Vec<Entry>,
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
    pub fn new(entries: Vec<Entry>) -> Self {
        Self {
            entries,
            timestamp: Instant::now(),
        }
    }

    pub fn from_single(entry: Entry) -> Self {
        Self::new(vec![entry])
    }

    fn age(&self) -> Duration {
        Instant::now().duration_since(self.timestamp)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Preview {
    pub title: String,
    // NOTE: this does couple the previewer with ratatui but allows
    // to only parse ansi text once and reuse it in the UI.
    pub content: Text<'static>,
    pub line_number: Option<u16>,
    pub total_lines: u16,
    pub footer: String,
}

const DEFAULT_PREVIEW_TITLE: &str = "Select an entry to preview";

impl Default for Preview {
    fn default() -> Self {
        Self {
            title: DEFAULT_PREVIEW_TITLE.to_string(),
            content: Text::from(EMPTY_STRING),
            line_number: None,
            total_lines: 1,
            footer: String::new(),
        }
    }
}

impl Preview {
    fn new(
        title: &str,
        content: Text<'static>,
        line_number: Option<u16>,
        total_lines: u16,
        footer: String,
    ) -> Self {
        Self {
            title: title.to_string(),
            content,
            line_number,
            total_lines,
            footer,
        }
    }
}

pub struct Previewer {
    config: Config,
    // FIXME: maybe use a bounded channel here with a single slot
    requests: UnboundedReceiver<Request>,
    last_job_entries: Vec<Entry>,
    command: CommandSpec,
    offset_expr: Option<Template>,
    results: UnboundedSender<Preview>,
    cache: Option<Arc<Mutex<Cache>>>,
}

impl Previewer {
    pub fn new(
        command: &CommandSpec,
        offset_expr: Option<Template>,
        config: Config,
        receiver: UnboundedReceiver<Request>,
        sender: UnboundedSender<Preview>,
        cache: bool,
    ) -> Self {
        let cache = if cache {
            Some(Arc::new(Mutex::new(Cache::default())))
        } else {
            None
        };
        Self {
            config,
            requests: receiver,
            last_job_entries: Vec::new(),
            command: command.clone(),
            offset_expr,
            results: sender,
            cache,
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
                        self.last_job_entries.clone_from(&ticket.entries);

                        // try to execute the preview with a timeout
                        let preview_command = self.command.clone();
                        let cache = self.cache.clone();
                        let offset_expr = self.offset_expr.clone();
                        let entries = ticket.entries;

                        match timeout(
                            self.config.job_timeout,
                            tokio::spawn(async move {
                                let entry_refs: Vec<&Entry> = entries.iter().collect();
                                let result = try_preview(
                                    &preview_command,
                                    &offset_expr,
                                    &entry_refs,
                                    &results_handle,
                                    &cache,
                                );

                                if let Err(e) = result {
                                    debug!(
                                        "Failed to generate preview for {} entries: {}",
                                        entries.len(),
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

/// Try to run the given command on the given entries and send the result
/// to the main thread via the given channel.
///
/// This function handles both single and multiple entry scenarios with different
/// processing strategies and configuration-aware template formatting.
///
/// ## Single Entry Processing
/// - Uses existing caching system for performance optimization
/// - Standard template formatting with optional shell escaping
/// - Supports both `Raw` and `StringPipeline` template types
/// - Caches successful results and errors to avoid re-execution
///
/// ## Multi-Select Processing
/// - `SelectorMode::Concatenate`: Joins all selected entries with the configured
///   separator and treats them as a single input for all template placeholders.
///   Ideal for commands that accept multiple arguments (e.g., `cat file1 file2`).
///
/// - `SelectorMode::OneToOne`: Maps each selected entry to individual template
///   placeholders in sequence. Analyzes template placeholder count and generates
///   warnings for mismatches. Best for commands with distinct argument slots
///   (e.g., `diff {} {}` expects exactly two files).
///
/// ## Shell Escaping
/// When `config.selector_shell_escaping` is enabled, all entry values are processed
/// through `shlex::try_quote()` to safely handle special characters, spaces, and
/// shell metacharacters in file paths and entry names.
///
/// ## Warning Generation
/// For `OneToOne` mode, automatically detects argument mapping issues:
/// - Excess entries: More selections than template placeholders
/// - Missing arguments: Fewer selections than template placeholders
/// - Warnings appear in the preview footer for user feedback
///
/// This function is responsible for the following tasks:
/// 1. execute a command with the preview configuration and selector settings
/// 2. analyze template structure and perform argument distribution based on selector mode
/// 3. generate user warnings for argument mapping mismatches in `OneToOne` mode
/// 4. apply shell escaping when configured for safe command execution
/// 5. ensure the result sent on the channel is well-formed with appropriate title/footer
/// 6. cache the result for future use (single entry only)
/// 7. send the result to the main thread via the channel
pub fn try_preview(
    command: &CommandSpec,
    offset_expr: &Option<Template>,
    entries: &[&Entry],
    results_handle: &UnboundedSender<Preview>,
    cache: &Option<Arc<Mutex<Cache>>>,
) -> Result<()> {
    if entries.is_empty() {
        return Err(anyhow::anyhow!(
            "Cannot generate preview with empty entries"
        ));
    }

    let is_single_entry = entries.len() == 1;
    let entry = entries[0];

    debug!("Preview template: {} (entries: {})", command, entries.len());

    // Check cache for single entry only
    if is_single_entry {
        if let Some(cache) = &cache {
            if let Some(preview) = cache.lock().get(entry) {
                debug!("Preview for entry [{}] found in cache", entry.raw);
                results_handle.send(preview).with_context(
                    || "Failed to send cached preview result to main thread.",
                )?;
                return Ok(());
            }
        }
    }

    let template = command.get_nth(0);

    // Convert entries to FxHashSet for process_entries
    let entries_set: rustc_hash::FxHashSet<Entry> =
        entries.iter().map(|&entry| entry.clone()).collect();

    // Use the unified selector system to process entries
    let (formatted_command, warning_message) =
        process_entries(&entries_set, template)
            .context("Failed to process entries for preview")?;

    debug!("Executing formatted command: {}", formatted_command);

    let child =
        shell_command(&formatted_command, command.interactive, &command.env)
            .output()?;

    // Create title based on entry count
    let title = if is_single_entry {
        entry.display().to_string()
    } else {
        format!("{} selected items", entries.len())
    };

    // Determine footer content (warnings for multi-select, empty for single entry)
    let footer = warning_message.unwrap_or_else(String::new);

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

            let line_number = if let Some(offset_expr) = offset_expr {
                let offset_str = offset_expr.format(&entry.raw)?;
                offset_str.parse::<u16>().ok()
            } else {
                None
            };

            Preview::new(
                &title,
                text,
                line_number,
                total_lines,
                footer.clone(),
            )
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

            Preview::new(&title, text, None, total_lines, footer)
        }
    };

    // Cache the preview if caching is enabled
    // Note: we're caching errors as well to avoid re-running potentially expensive commands
    if is_single_entry {
        if let Some(cache) = &cache {
            cache.lock().insert(entry, &preview);
            debug!("Preview for entry '{}' cached", entry.raw);
        }
    }

    results_handle
        .send(preview)
        .with_context(|| "Failed to send preview result to main thread.")
}
