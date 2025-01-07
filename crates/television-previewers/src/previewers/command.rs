/// git log --oneline --date=short --pretty="format:%C(auto)%h %s %Cblue%an %C(green)%cd" "$@" | ~/code/rust/television/target/release/tv --preview 'git show -p --stat --pretty=fuller --color=always {0}' --delimiter ' '
use crate::previewers::cache::PreviewCache;
use crate::previewers::{Preview, PreviewContent};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use regex::Regex;
use rustc_hash::FxHashSet;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use television_channels::entry::{Entry, PreviewCommand};
use television_utils::command::shell_command;
use tracing::debug;

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct CommandPreviewer {
    cache: Arc<Mutex<PreviewCache>>,
    config: CommandPreviewerConfig,
    concurrent_preview_tasks: Arc<AtomicU8>,
    last_previewed: Arc<Mutex<Arc<Preview>>>,
    in_flight_previews: Arc<Mutex<FxHashSet<String>>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CommandPreviewerConfig {
    delimiter: String,
}

const DEFAULT_DELIMITER: &str = ":";

impl Default for CommandPreviewerConfig {
    fn default() -> Self {
        CommandPreviewerConfig {
            delimiter: String::from(DEFAULT_DELIMITER),
        }
    }
}

impl CommandPreviewerConfig {
    pub fn new(delimiter: &str) -> Self {
        CommandPreviewerConfig {
            delimiter: String::from(delimiter),
        }
    }
}

const MAX_CONCURRENT_PREVIEW_TASKS: u8 = 3;

impl CommandPreviewer {
    pub fn new(config: Option<CommandPreviewerConfig>) -> Self {
        let config = config.unwrap_or_default();
        CommandPreviewer {
            cache: Arc::new(Mutex::new(PreviewCache::default())),
            config,
            concurrent_preview_tasks: Arc::new(AtomicU8::new(0)),
            last_previewed: Arc::new(Mutex::new(Arc::new(
                Preview::default().stale(),
            ))),
            in_flight_previews: Arc::new(Mutex::new(FxHashSet::default())),
        }
    }

    pub fn preview(
        &mut self,
        entry: &Entry,
        command: &PreviewCommand,
    ) -> Arc<Preview> {
        // do we have a preview in cache for that entry?
        if let Some(preview) = self.cache.lock().get(&entry.name) {
            return preview.clone();
        }
        debug!("Preview cache miss for {:?}", entry.name);

        // are we already computing a preview in the background for that entry?
        if self.in_flight_previews.lock().contains(&entry.name) {
            debug!("Preview already in flight for {:?}", entry.name);
            return self.last_previewed.lock().clone();
        }

        if self.concurrent_preview_tasks.load(Ordering::Relaxed)
            < MAX_CONCURRENT_PREVIEW_TASKS
        {
            self.concurrent_preview_tasks
                .fetch_add(1, Ordering::Relaxed);
            let cache = self.cache.clone();
            let entry_c = entry.clone();
            let concurrent_tasks = self.concurrent_preview_tasks.clone();
            let command = command.clone();
            let last_previewed = self.last_previewed.clone();
            tokio::spawn(async move {
                try_preview(
                    &command,
                    &entry_c,
                    &cache,
                    &concurrent_tasks,
                    &last_previewed,
                );
            });
        } else {
            debug!("Too many concurrent preview tasks running");
        }

        self.last_previewed.lock().clone()
    }
}

lazy_static! {
    static ref COMMAND_PLACEHOLDER_REGEX: Regex =
        Regex::new(r"\{(\d+)\}").unwrap();
}

/// Format the command with the entry name and provided placeholders
///
/// # Example
/// ```
/// use television_channels::entry::{PreviewCommand, PreviewType, Entry};
/// use television_previewers::previewers::command::format_command;
///
/// let command = PreviewCommand {
///     command: "something {} {2} {0}".to_string(),
///     delimiter: ":".to_string(),
/// };
/// let entry = Entry::new("a:given:entry:to:preview".to_string(), PreviewType::Command(command.clone()));
/// let formatted_command = format_command(&command, &entry);
///
/// assert_eq!(formatted_command, "something a:given:entry:to:preview entry a");
/// ```
pub fn format_command(command: &PreviewCommand, entry: &Entry) -> String {
    let parts = entry.name.split(&command.delimiter).collect::<Vec<&str>>();
    debug!("Parts: {:?}", parts);

    let mut formatted_command = command.command.replace("{}", &entry.name);

    formatted_command = COMMAND_PLACEHOLDER_REGEX
        .replace_all(&formatted_command, |caps: &regex::Captures| {
            let index =
                caps.get(1).unwrap().as_str().parse::<usize>().unwrap();
            parts[index].to_string()
        })
        .to_string();

    formatted_command
}

pub fn try_preview(
    command: &PreviewCommand,
    entry: &Entry,
    cache: &Arc<Mutex<PreviewCache>>,
    concurrent_tasks: &Arc<AtomicU8>,
    last_previewed: &Arc<Mutex<Arc<Preview>>>,
) {
    debug!("Computing preview for {:?}", entry.name);
    let command = format_command(command, entry);
    debug!("Formatted preview command: {:?}", command);

    let output = shell_command()
        .arg(&command)
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let content = String::from_utf8_lossy(&output.stdout);
        let preview = Arc::new(Preview::new(
            entry.name.clone(),
            PreviewContent::AnsiText(content.to_string()),
            None,
            false,
        ));

        cache.lock().insert(entry.name.clone(), &preview);
        let mut tp = last_previewed.lock();
        *tp = preview.stale().into();
    } else {
        let content = String::from_utf8_lossy(&output.stderr);
        let preview = Arc::new(Preview::new(
            entry.name.clone(),
            PreviewContent::AnsiText(content.to_string()),
            None,
            false,
        ));
        cache.lock().insert(entry.name.clone(), &preview);
    }

    concurrent_tasks.fetch_sub(1, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;
    use television_channels::entry::{Entry, PreviewType};

    #[test]
    fn test_format_command() {
        let command = PreviewCommand {
            command: "something {} {2} {0}".to_string(),
            delimiter: ":".to_string(),
        };
        let entry = Entry::new(
            "an:entry:to:preview".to_string(),
            PreviewType::Command(command.clone()),
        );
        let formatted_command = format_command(&command, &entry);

        assert_eq!(formatted_command, "something an:entry:to:preview to an");
    }

    #[test]
    fn test_format_command_no_placeholders() {
        let command = PreviewCommand {
            command: "something".to_string(),
            delimiter: ":".to_string(),
        };
        let entry = Entry::new(
            "an:entry:to:preview".to_string(),
            PreviewType::Command(command.clone()),
        );
        let formatted_command = format_command(&command, &entry);

        assert_eq!(formatted_command, "something");
    }

    #[test]
    fn test_format_command_with_global_placeholder_only() {
        let command = PreviewCommand {
            command: "something {}".to_string(),
            delimiter: ":".to_string(),
        };
        let entry = Entry::new(
            "an:entry:to:preview".to_string(),
            PreviewType::Command(command.clone()),
        );
        let formatted_command = format_command(&command, &entry);

        assert_eq!(formatted_command, "something an:entry:to:preview");
    }

    #[test]
    fn test_format_command_with_positional_placeholders_only() {
        let command = PreviewCommand {
            command: "something {0} -t {2}".to_string(),
            delimiter: ":".to_string(),
        };
        let entry = Entry::new(
            "an:entry:to:preview".to_string(),
            PreviewType::Command(command.clone()),
        );
        let formatted_command = format_command(&command, &entry);

        assert_eq!(formatted_command, "something an -t to");
    }
}
