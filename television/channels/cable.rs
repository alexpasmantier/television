use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use std::process::Stdio;

use preview::{parse_preview_kind, PreviewKind};
use prototypes::{CableChannelPrototype, DEFAULT_DELIMITER};
use rustc_hash::{FxBuildHasher, FxHashSet};
use tracing::debug;

use crate::channels::entry::{Entry, PreviewCommand, PreviewType};
use crate::channels::OnAir;
use crate::matcher::Matcher;
use crate::matcher::{config::Config, injector::Injector};
use crate::utils::command::shell_command;

pub mod preview;
pub mod prototypes;

#[allow(dead_code)]
pub struct Channel {
    pub name: String,
    matcher: Matcher<String>,
    entries_command: String,
    preview_kind: PreviewKind,
    selected_entries: FxHashSet<Entry>,
    crawl_handle: tokio::task::JoinHandle<()>,
}

impl Default for Channel {
    fn default() -> Self {
        Self::new(
            "files",
            "find . -type f",
            false,
            Some(PreviewCommand::new("cat {}", ":")),
        )
    }
}

impl From<CableChannelPrototype> for Channel {
    fn from(prototype: CableChannelPrototype) -> Self {
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
        let preview_kind = match preview_command {
            Some(command) => {
                parse_preview_kind(&command).unwrap_or_else(|_| {
                    panic!("Invalid preview command: {command}")
                })
            }
            None => PreviewKind::None,
        };
        debug!("Preview kind: {:?}", preview_kind);
        Self {
            matcher,
            entries_command: entries_command.to_string(),
            preview_kind,
            name: name.to_string(),
            selected_entries: HashSet::with_hasher(FxBuildHasher),
            crawl_handle,
        }
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

impl OnAir for Channel {
    fn find(&mut self, pattern: &str) {
        self.matcher.find(pattern);
    }

    fn results(&mut self, num_entries: u32, offset: u32) -> Vec<Entry> {
        self.matcher.tick();
        self.matcher
            .results(num_entries, offset)
            .into_iter()
            .map(|item| {
                let path = item.matched_string;
                Entry::new(
                    path,
                    match &self.preview_kind {
                        PreviewKind::Command(ref preview_command) => {
                            PreviewType::Command(preview_command.clone())
                        }
                        PreviewKind::Builtin(preview_type) => {
                            preview_type.clone()
                        }
                        PreviewKind::None => PreviewType::None,
                    },
                )
                .with_name_match_indices(&item.match_indices)
            })
            .collect()
    }

    fn get_result(&self, index: u32) -> Option<Entry> {
        self.matcher.get_result(index).map(|item| {
            let path = item.matched_string;
            Entry::new(
                path,
                match &self.preview_kind {
                    PreviewKind::Command(ref preview_command) => {
                        PreviewType::Command(preview_command.clone())
                    }
                    PreviewKind::Builtin(preview_type) => preview_type.clone(),
                    PreviewKind::None => PreviewType::None,
                },
            )
        })
    }

    fn selected_entries(&self) -> &FxHashSet<Entry> {
        &self.selected_entries
    }

    fn toggle_selection(&mut self, entry: &Entry) {
        if self.selected_entries.contains(entry) {
            self.selected_entries.remove(entry);
        } else {
            self.selected_entries.insert(entry.clone());
        }
    }

    fn result_count(&self) -> u32 {
        self.matcher.matched_item_count
    }

    fn total_count(&self) -> u32 {
        self.matcher.total_item_count
    }

    fn running(&self) -> bool {
        self.matcher.status.running || !self.crawl_handle.is_finished()
    }

    fn shutdown(&self) {}

    fn supports_preview(&self) -> bool {
        self.preview_kind != PreviewKind::None
    }
}
