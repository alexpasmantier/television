use std::{
    io::{stdin, BufRead},
    thread::spawn,
};

use devicons::FileIcon;
use tracing::debug;

use super::OnAir;
use crate::entry::{Entry, PreviewType};
use television_fuzzy::{SimdInjector, SimdMatcher};

pub struct Channel {
    matcher: SimdMatcher<String>,
    icon: FileIcon,
}

impl Channel {
    pub fn new() -> Self {
        let matcher = SimdMatcher::new(|s: &String| s.trim_end());
        let injector = matcher.injector();

        spawn(move || stream_from_stdin(injector.clone()));

        Self {
            matcher,
            icon: FileIcon::from("nu"),
        }
    }
}

impl Default for Channel {
    fn default() -> Self {
        Self::new()
    }
}

const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

fn stream_from_stdin(injector: SimdInjector<String>) {
    let mut stdin = stdin().lock();
    let mut buffer = String::new();

    let instant = std::time::Instant::now();
    loop {
        match stdin.read_line(&mut buffer) {
            Ok(c) if c > 0 => {
                if !buffer.trim().is_empty() {
                    injector.push(buffer.clone());
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
            .map(|s| {
                Entry::new(s.inner, PreviewType::Basic)
                    .with_icon(self.icon.clone())
                    .with_name_match_ranges(s.match_indices)
            })
            .collect()
    }

    fn get_result(&self, index: u32) -> Option<Entry> {
        self.matcher
            .get_result(index as usize)
            .map(|s| Entry::new(s.matched_string.clone(), PreviewType::Basic))
    }

    fn result_count(&self) -> u32 {
        self.matcher.result_count() as u32
    }

    fn total_count(&self) -> u32 {
        self.matcher.total_count() as u32
    }

    fn running(&self) -> bool {
        self.matcher.running()
    }

    fn shutdown(&self) {}
}
