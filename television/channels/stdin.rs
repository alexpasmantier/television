use std::{
    collections::HashSet,
    io::{stdin, BufRead},
    thread::spawn,
};

use rustc_hash::{FxBuildHasher, FxHashSet};
use tracing::debug;

use super::OnAir;
use crate::channels::entry::{Entry, PreviewType};
use crate::matcher::{config::Config, injector::Injector, Matcher};

pub struct Channel {
    matcher: Matcher<String>,
    preview_type: PreviewType,
    selected_entries: FxHashSet<Entry>,
}

impl Channel {
    pub fn new(preview_type: Option<PreviewType>) -> Self {
        let matcher = Matcher::new(Config::default());
        let injector = matcher.injector();

        spawn(move || stream_from_stdin(&injector));

        Self {
            matcher,
            preview_type: preview_type.unwrap_or_default(),
            selected_entries: HashSet::with_hasher(FxBuildHasher),
        }
    }
}

impl Default for Channel {
    fn default() -> Self {
        Self::new(None)
    }
}

const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

fn stream_from_stdin(injector: &Injector<String>) {
    let mut stdin = stdin().lock();
    let mut buffer = String::new();

    let instant = std::time::Instant::now();
    loop {
        match stdin.read_line(&mut buffer) {
            Ok(c) if c > 0 => {
                let buf = buffer.trim();
                if !buf.is_empty() {
                    injector.push(buf.to_string(), |e, cols| {
                        cols[0] = e.to_string().into();
                    });
                }
                buffer.clear();
            }
            Ok(0) => {
                debug!("EOF");
                break;
            }
            _ => {
                debug!("Error reading from stdin");
                if instant.elapsed() > TIMEOUT {
                    break;
                }
            }
        }
    }
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
                // NOTE: we're passing `PreviewType::Basic` here just as a placeholder
                // to avoid storing the preview command multiple times for each item.
                Entry::new(item.matched_string, PreviewType::Basic)
                    .with_name_match_ranges(&item.match_indices)
            })
            .collect()
    }

    fn get_result(&self, index: u32) -> Option<Entry> {
        self.matcher.get_result(index).map(|item| {
            Entry::new(item.matched_string, self.preview_type.clone())
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
        self.matcher.status.running
    }

    fn shutdown(&self) {}
}
