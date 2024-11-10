use std::io::BufRead;
use std::path::Path;

use devicons::FileIcon;

use super::OnAir;
use crate::entry::{Entry, PreviewType};
use television_fuzzy::matcher::{config::Config, Matcher};
use television_utils::strings::preprocess_line;

pub struct Channel {
    matcher: Matcher<String>,
    icon: FileIcon,
}

const NUM_THREADS: usize = 2;

impl Channel {
    pub fn new() -> Self {
        let mut lines = Vec::new();
        for line in std::io::stdin().lock().lines().map_while(Result::ok) {
            lines.push(preprocess_line(&line));
        }
        let matcher = Matcher::new(Config::default().n_threads(NUM_THREADS));
        let injector = matcher.injector();
        for line in &lines {
            let () = injector.push(line.clone(), |e, cols| {
                cols[0] = e.clone().into();
            });
        }
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
