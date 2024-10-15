use std::path::Path;
use std::{io::BufRead, sync::Arc};

use devicons::FileIcon;
use nucleo::{Config, Nucleo};
use tracing::debug;

use crate::entry::Entry;
use crate::fuzzy::MATCHER;
use crate::previewers::PreviewType;

use super::TelevisionChannel;

pub(crate) struct Channel {
    matcher: Nucleo<String>,
    last_pattern: String,
    result_count: u32,
    total_count: u32,
    icon: FileIcon,
}

const NUM_THREADS: usize = 2;

impl Channel {
    pub(crate) fn new() -> Self {
        let mut lines = Vec::new();
        for line in std::io::stdin().lock().lines().map_while(Result::ok) {
            debug!("Read line: {:?}", line);
            lines.push(line);
        }
        let matcher = Nucleo::new(
            Config::DEFAULT,
            Arc::new(|| {}),
            Some(NUM_THREADS),
            1,
        );
        let injector = matcher.injector();
        for line in &lines {
            let _ = injector.push(line.clone(), |e, cols| {
                cols[0] = e.clone().into();
            });
        }
        Self {
            matcher,
            last_pattern: String::new(),
            result_count: 0,
            total_count: 0,
            icon: FileIcon::from("nu"),
        }
    }

    const MATCHER_TICK_TIMEOUT: u64 = 10;
}

impl TelevisionChannel for Channel {
    // maybe this could be sort of automatic with a blanket impl (making Finder generic over
    // its matcher type or something)
    fn find(&mut self, pattern: &str) {
        if pattern != self.last_pattern {
            self.matcher.pattern.reparse(
                0,
                pattern,
                nucleo::pattern::CaseMatching::Smart,
                nucleo::pattern::Normalization::Smart,
                pattern.starts_with(&self.last_pattern),
            );
            self.last_pattern = pattern.to_string();
        }
    }

    fn results(&mut self, num_entries: u32, offset: u32) -> Vec<Entry> {
        let status = self.matcher.tick(Self::MATCHER_TICK_TIMEOUT);
        let snapshot = self.matcher.snapshot();
        if status.changed {
            self.result_count = snapshot.matched_item_count();
            self.total_count = snapshot.item_count();
        }
        let mut indices = Vec::new();
        let mut matcher = MATCHER.lock();
        let icon = self.icon;

        snapshot
            .matched_items(
                offset
                    ..(num_entries + offset)
                        .min(snapshot.matched_item_count()),
            )
            .map(move |item| {
                snapshot.pattern().column_pattern(0).indices(
                    item.matcher_columns[0].slice(..),
                    &mut matcher,
                    &mut indices,
                );
                indices.sort_unstable();
                indices.dedup();
                let indices = indices.drain(..);

                let content = item.matcher_columns[0].to_string();
                let path = Path::new(&content);
                let icon = if path.try_exists().unwrap_or(false) {
                    FileIcon::from(path)
                } else {
                    icon
                };
                Entry::new(content.clone(), PreviewType::Basic)
                    .with_name_match_ranges(
                        indices.map(|i| (i, i + 1)).collect(),
                    )
                    .with_icon(icon)
            })
            .collect()
    }

    fn get_result(&self, index: u32) -> Option<super::Entry> {
        let snapshot = self.matcher.snapshot();
        snapshot.get_matched_item(index).map(|item| {
            let content = item.matcher_columns[0].to_string();
            // if we recognize a file path, use a file icon
            // and set the preview type to "Files"
            let path = Path::new(&content);
            if path.is_file() {
                Entry::new(content.clone(), PreviewType::Files)
                    .with_icon(FileIcon::from(path))
            } else if path.is_dir() {
                Entry::new(content.clone(), PreviewType::Directory)
                    .with_icon(FileIcon::from(path))
            } else {
                Entry::new(content.clone(), PreviewType::Basic)
                    .with_icon(self.icon)
            }
        })
    }

    fn result_count(&self) -> u32 {
        self.result_count
    }

    fn total_count(&self) -> u32 {
        self.total_count
    }
}
