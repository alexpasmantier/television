use crate::preview::{Preview, PreviewContent};
use crate::utils::command::shell_command;
use crate::{
    channels::{entry::Entry, preview::PreviewCommand},
    preview::cache::PreviewCache,
};
use parking_lot::Mutex;
use regex::Regex;
use rustc_hash::FxHashSet;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use tracing::debug;

#[allow(dead_code)]
#[derive(Debug)]
pub struct CommandPreviewer {
    cache: Arc<Mutex<PreviewCache>>,
    config: CommandPreviewerConfig,
    concurrent_preview_tasks: Arc<AtomicU8>,
    in_flight_previews: Arc<Mutex<FxHashSet<String>>>,
    command_re: Regex,
}

impl Default for CommandPreviewer {
    fn default() -> Self {
        CommandPreviewer::new(None)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CommandPreviewerConfig {
    delimiter: String,
}

const DEFAULT_DELIMITER: &str = " ";

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
            in_flight_previews: Arc::new(Mutex::new(FxHashSet::default())),
            command_re: Regex::new(r"\{(\d+)\}").unwrap(),
        }
    }

    pub fn cached(&self, entry: &Entry) -> Option<Arc<Preview>> {
        self.cache.lock().get(&entry.name)
    }

    pub fn preview(
        &mut self,
        entry: &Entry,
        command: &PreviewCommand,
    ) -> Option<Arc<Preview>> {
        if let Some(preview) = self.cached(entry) {
            Some(preview)
        } else {
            // preview is not in cache, spawn a task to compute the preview
            debug!("Preview cache miss for {:?}", entry.name);
            self.handle_preview_request(entry, command);
            None
        }
    }

    pub fn handle_preview_request(
        &mut self,
        entry: &Entry,
        command: &PreviewCommand,
    ) {
        if self.in_flight_previews.lock().contains(&entry.name) {
            debug!("Preview already in flight for {:?}", entry.name);
            return;
        }

        if self.concurrent_preview_tasks.load(Ordering::Relaxed)
            < MAX_CONCURRENT_PREVIEW_TASKS
        {
            self.in_flight_previews.lock().insert(entry.name.clone());
            self.concurrent_preview_tasks
                .fetch_add(1, Ordering::Relaxed);
            let cache = self.cache.clone();
            let entry_c = entry.clone();
            let concurrent_tasks = self.concurrent_preview_tasks.clone();
            let command = command.clone();
            let in_flight_previews = self.in_flight_previews.clone();
            let command_re = self.command_re.clone();
            tokio::spawn(async move {
                try_preview(
                    &command,
                    &entry_c,
                    &cache,
                    &concurrent_tasks,
                    &in_flight_previews,
                    &command_re,
                );
            });
        } else {
            debug!(
                "Too many concurrent preview tasks, skipping {:?}",
                entry.name
            );
        }
    }
}

/// Format the command with the entry name and provided placeholders
///
/// # Example
/// ```
/// use television::channels::{preview::{PreviewCommand, PreviewType}, entry::Entry};
/// use television::preview::previewers::command::format_command;
///
/// let command = PreviewCommand {
///     command: "something {} {2} {0}".to_string(),
///     delimiter: ":".to_string(),
/// };
/// let entry = Entry::new("a:given:entry:to:preview".to_string(), PreviewType::Command(command.clone()));
/// let formatted_command = format_command(&command, &entry, &regex::Regex::new(r"\{(\d+)\}").unwrap());
///
/// assert_eq!(formatted_command, "something 'a:given:entry:to:preview' 'entry' 'a'");
/// ```
pub fn format_command(
    command: &PreviewCommand,
    entry: &Entry,
    command_re: &Regex,
) -> String {
    let parts = entry.name.split(&command.delimiter).collect::<Vec<&str>>();
    debug!("Parts: {:?}", parts);

    let mut formatted_command = command
        .command
        .replace("{}", format!("'{}'", entry.name).as_str());

    formatted_command = command_re
        .replace_all(&formatted_command, |caps: &regex::Captures| {
            let index =
                caps.get(1).unwrap().as_str().parse::<usize>().unwrap();
            format!("'{}'", parts[index])
        })
        .to_string();

    formatted_command
}

pub fn try_preview(
    command: &PreviewCommand,
    entry: &Entry,
    cache: &Arc<Mutex<PreviewCache>>,
    concurrent_tasks: &Arc<AtomicU8>,
    in_flight_previews: &Arc<Mutex<FxHashSet<String>>>,
    command_re: &Regex,
) {
    debug!("Computing preview for {:?}", entry.name);
    let command = format_command(command, entry, command_re);
    debug!("Formatted preview command: {:?}", command);

    let child = shell_command(false)
        .arg(&command)
        .output()
        .expect("failed to execute process");

    if child.status.success() {
        let content = String::from_utf8_lossy(&child.stdout);
        let preview = Arc::new(Preview::new(
            entry.name.clone(),
            PreviewContent::AnsiText(content.to_string()),
            None,
            None,
            u16::try_from(content.lines().count()).unwrap_or(u16::MAX),
        ));

        cache.lock().insert(entry.name.clone(), &preview);
    } else {
        let content = String::from_utf8_lossy(&child.stderr);
        let preview = Arc::new(Preview::new(
            entry.name.clone(),
            PreviewContent::AnsiText(content.to_string()),
            None,
            None,
            u16::try_from(content.lines().count()).unwrap_or(u16::MAX),
        ));
        cache.lock().insert(entry.name.clone(), &preview);
    }

    concurrent_tasks.fetch_sub(1, Ordering::Relaxed);
    in_flight_previews.lock().remove(&entry.name);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channels::{entry::Entry, preview::PreviewType};

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
        let formatted_command = format_command(
            &command,
            &entry,
            &Regex::new(r"\{(\d+)\}").unwrap(),
        );

        assert_eq!(
            formatted_command,
            "something 'an:entry:to:preview' 'to' 'an'"
        );
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
        let formatted_command = format_command(
            &command,
            &entry,
            &Regex::new(r"\{(\d+)\}").unwrap(),
        );

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
        let formatted_command = format_command(
            &command,
            &entry,
            &Regex::new(r"\{(\d+)\}").unwrap(),
        );

        assert_eq!(formatted_command, "something 'an:entry:to:preview'");
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
        let formatted_command = format_command(
            &command,
            &entry,
            &Regex::new(r"\{(\d+)\}").unwrap(),
        );

        assert_eq!(formatted_command, "something 'an' -t 'to'");
    }
}
