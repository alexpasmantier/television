use std::{
    io::{stdin, BufRead},
    thread::spawn,
};

use tracing::debug;

use super::OnAir;
use crate::entry::{Entry, PreviewType};
use television_fuzzy::matcher::{config::Config, injector::Injector, Matcher};

pub struct Channel {
    matcher: Matcher<String>,
    preview_type: PreviewType,
}

impl Channel {
    pub fn new(preview_type: Option<PreviewType>) -> Self {
        let matcher = Matcher::new(Config::default());
        let injector = matcher.injector();

        spawn(move || stream_from_stdin(&injector));

        Self {
            matcher,
            preview_type: preview_type.unwrap_or_default(),
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
                if !buffer.trim().is_empty() {
                    injector.push(buffer.clone(), |e, cols| {
                        cols[0] = e.clone().into();
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
                Entry::new(item.matched_string, self.preview_type.clone())
                    .with_name_match_ranges(item.match_indices)
            })
            .collect()
    }

    fn get_result(&self, index: u32) -> Option<Entry> {
        self.matcher.get_result(index).map(|item| {
            Entry::new(item.matched_string.clone(), self.preview_type.clone())
        })
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
