use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use std::process::Stdio;

use rustc_hash::{FxBuildHasher, FxHashSet};
use tracing::debug;

use crate::channels::{
    entry::Entry, preview::PreviewCommand, prototypes::ChannelPrototype,
};
use crate::matcher::Matcher;
use crate::matcher::{config::Config, injector::Injector};
use crate::utils::command::shell_command;

use super::prototypes::format_prototype_string;

pub struct Channel {
    pub name: String,
    matcher: Matcher<String>,
    pub preview_command: Option<PreviewCommand>,
    selected_entries: FxHashSet<Entry>,
    crawl_handle: tokio::task::JoinHandle<()>,
}

impl Default for Channel {
    fn default() -> Self {
        Self::new(&ChannelPrototype::new(
            "files",
            "find . -type f",
            false,
            Some(PreviewCommand::new("cat {}", ":", None)),
        ))
    }
}

impl Channel {
    pub fn new(prototype: &ChannelPrototype) -> Self {
        let matcher = Matcher::new(Config::default());
        let injector = matcher.injector();
        let crawl_handle = tokio::spawn(load_candidates(
            prototype.source_command.to_string(),
            prototype.interactive,
            injector,
        ));
        Self {
            matcher,
            preview_command: prototype.preview_command.clone(),
            name: prototype.name.to_string(),
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
            let name = item.matched_string;
            if let Some(cmd) = &self.preview_command {
                if let Some(offset_expr) = &cmd.offset_expr {
                    let offset_string = format_prototype_string(
                        offset_expr,
                        &name,
                        &cmd.delimiter,
                    );
                    let offset_str = {
                        offset_string
                            .strip_prefix('\'')
                            .and_then(|s| s.strip_suffix('\''))
                            .unwrap_or(&offset_string)
                    };

                    return Entry::new(name).with_line_number(
                        offset_str.parse::<usize>().unwrap(),
                    );
                }
            }
            Entry::new(name)
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
