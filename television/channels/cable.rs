use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use std::process::Stdio;

use rustc_hash::{FxBuildHasher, FxHashSet};
use tracing::debug;

use crate::channels::{
    entry::Entry,
    preview::PreviewCommand,
    prototypes::{ChannelPrototype, DEFAULT_DELIMITER},
};
use crate::matcher::Matcher;
use crate::matcher::{config::Config, injector::Injector};
use crate::utils::command::shell_command;

#[allow(dead_code)]
pub struct Channel {
    pub name: String,
    matcher: Matcher<String>,
    entries_command: String,
    pub preview_command: Option<PreviewCommand>,
    selected_entries: FxHashSet<Entry>,
    crawl_handle: tokio::task::JoinHandle<()>,
}

impl Default for Channel {
    fn default() -> Self {
        Self::new(
            "files",
            "find . -type f",
            false,
            Some(PreviewCommand::new("cat {}", ":", None)),
        )
    }
}

impl From<ChannelPrototype> for Channel {
    fn from(prototype: ChannelPrototype) -> Self {
        Self::new(
            &prototype.name,
            &prototype.source_command,
            prototype.interactive,
            match prototype.preview_command {
                Some(command) => Some(PreviewCommand::new(
                    &command,
                    &prototype
                        .preview_delimiter
                        .unwrap_or(DEFAULT_DELIMITER.to_string()),
                    prototype.preview_offset,
                )),
                None => None,
            },
        )
    }
}

impl Channel {
    pub fn new(
        name: &str,
        entries_command: &str,
        interactive: bool,
        preview_command: Option<PreviewCommand>,
    ) -> Self {
        let matcher = Matcher::new(Config::default());
        let injector = matcher.injector();
        let crawl_handle = tokio::spawn(load_candidates(
            entries_command.to_string(),
            interactive,
            injector,
        ));
        Self {
            matcher,
            entries_command: entries_command.to_string(),
            preview_command,
            name: name.to_string(),
            selected_entries: HashSet::with_hasher(FxBuildHasher),
            crawl_handle,
        }
    }

    pub fn find(&mut self, pattern: &str) {
        self.matcher.find(pattern);
    }

    pub fn results(&mut self, num_entries: u32, offset: u32) -> Vec<Entry> {
        self.matcher.tick();
        self.matcher
            .results(num_entries, offset)
            .into_iter()
            .map(|item| {
                let path = item.matched_string;
                Entry::new(path).with_name_match_indices(&item.match_indices)
            })
            .collect()
    }

    pub fn get_result(&self, index: u32) -> Option<Entry> {
        self.matcher.get_result(index).map(|item| {
            let path = item.matched_string;
            Entry::new(path)
        })
    }

    pub fn selected_entries(&self) -> &FxHashSet<Entry> {
        &self.selected_entries
    }

    pub fn toggle_selection(&mut self, entry: &Entry) {
        if self.selected_entries.contains(entry) {
            self.selected_entries.remove(entry);
        } else {
            self.selected_entries.insert(entry.clone());
        }
    }

    pub fn result_count(&self) -> u32 {
        self.matcher.matched_item_count
    }

    pub fn total_count(&self) -> u32 {
        self.matcher.total_item_count
    }

    pub fn running(&self) -> bool {
        self.matcher.status.running || !self.crawl_handle.is_finished()
    }

    pub fn shutdown(&self) {}

    pub fn supports_preview(&self) -> bool {
        self.preview_command.is_some()
    }
}

#[allow(clippy::unused_async)]
async fn load_candidates(
    command: String,
    interactive: bool,
    injector: Injector<String>,
) {
    debug!("Loading candidates from command: {:?}", command);
    let mut child = shell_command(interactive)
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to execute process");

    if let Some(out) = child.stdout.take() {
        let reader = BufReader::new(out);
        let mut produced_output = false;

        #[allow(clippy::manual_flatten)]
        for line in reader.lines() {
            if let Ok(l) = line {
                if !l.trim().is_empty() {
                    let () = injector.push(l, |e, cols| {
                        cols[0] = e.clone().into();
                    });
                    produced_output = true;
                }
            }
        }

        if !produced_output {
            let reader = BufReader::new(child.stderr.take().unwrap());
            for line in reader.lines() {
                let line = line.unwrap();
                if !line.trim().is_empty() {
                    let () = injector.push(line, |e, cols| {
                        cols[0] = e.clone().into();
                    });
                }
            }
        }
    }
    let _ = child.wait();
}
