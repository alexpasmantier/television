use std::{
    io::{stdin, BufRead},
    path::Path,
    thread::spawn,
};

use devicons::FileIcon;
use tracing::debug;

use super::OnAir;
use crate::entry::{Entry, PreviewType};
use television_fuzzy::matcher::{config::Config, injector::Injector, Matcher};

pub struct Channel {
    matcher: Matcher<String>,
    icon: FileIcon,
}

impl Channel {
    pub fn new() -> Self {
        let matcher = Matcher::new(Config::default());
        let injector = matcher.injector();

        spawn(move || stream_from_stdin(injector));

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

fn stream_from_stdin(injector: Injector<String>) {
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
                let path = Path::new(&item.matched_string);
                let icon = if path.try_exists().unwrap_or(false) {
                    FileIcon::from(path)
                } else {
                    self.icon
                };
                Entry::new(item.matched_string, PreviewType::Basic)
                    .with_name_match_ranges(item.match_indices)
                    .with_icon(icon)
            })
            .collect()
    }

    fn get_result(&self, index: u32) -> Option<Entry> {
        self.matcher.get_result(index).map(|item| {
            let path = Path::new(&item.matched_string);
            // if we recognize a file path, use a file icon
            // and set the preview type to "Files"
            if path.is_file() {
                Entry::new(item.matched_string.clone(), PreviewType::Files)
                    .with_icon(FileIcon::from(path))
            } else if path.is_dir() {
                Entry::new(item.matched_string.clone(), PreviewType::Directory)
                    .with_icon(FileIcon::from(path))
            } else {
                Entry::new(item.matched_string.clone(), PreviewType::Basic)
                    .with_icon(self.icon)
            }
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
