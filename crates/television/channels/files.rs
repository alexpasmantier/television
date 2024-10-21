use devicons::FileIcon;
use nucleo::{
    pattern::{CaseMatching, Normalization},
    Config, Injector, Nucleo,
};
use std::{
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    sync::Arc,
};

use ignore::DirEntry;

use super::OnAir;
use crate::previewers::PreviewType;
use crate::utils::files::{walk_builder, DEFAULT_NUM_THREADS};
use crate::{
    entry::Entry, utils::strings::proportion_of_printable_ascii_characters,
};
use crate::{fuzzy::MATCHER, utils::strings::PRINTABLE_ASCII_THRESHOLD};

pub struct Channel {
    matcher: Nucleo<DirEntry>,
    last_pattern: String,
    result_count: u32,
    total_count: u32,
    running: bool,
    crawl_handle: tokio::task::JoinHandle<()>,
    // PERF: cache results (to make deleting characters smoother) but like
    // a shallow cache (maybe more like a stack actually? so we just pop result sets)
}

impl Channel {
    pub fn new(starting_dir: &Path) -> Self {
        let matcher = Nucleo::new(
            Config::DEFAULT.match_paths(),
            Arc::new(|| {}),
            None,
            1,
        );
        // start loading files in the background
        let crawl_handle = tokio::spawn(load_files(
            starting_dir.to_path_buf(),
            matcher.injector(),
        ));
        Channel {
            matcher,
            last_pattern: String::new(),
            result_count: 0,
            total_count: 0,
            running: false,
            crawl_handle,
        }
    }

    const MATCHER_TICK_TIMEOUT: u64 = 2;
}

impl Default for Channel {
    fn default() -> Self {
        Self::new(&std::env::current_dir().unwrap())
    }
}

impl OnAir for Channel {
    fn find(&mut self, pattern: &str) {
        if pattern != self.last_pattern {
            self.matcher.pattern.reparse(
                0,
                pattern,
                CaseMatching::Smart,
                Normalization::Smart,
                pattern.starts_with(&self.last_pattern),
            );
            self.last_pattern = pattern.to_string();
        }
    }

    fn result_count(&self) -> u32 {
        self.result_count
    }

    fn total_count(&self) -> u32 {
        self.total_count
    }

    fn results(&mut self, num_entries: u32, offset: u32) -> Vec<Entry> {
        let status = self.matcher.tick(Self::MATCHER_TICK_TIMEOUT);
        let snapshot = self.matcher.snapshot();
        if status.changed {
            self.result_count = snapshot.matched_item_count();
            self.total_count = snapshot.item_count();
        }
        self.running = status.running;
        let mut indices = Vec::new();
        let mut matcher = MATCHER.lock();

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

                let path = item.matcher_columns[0].to_string();
                Entry::new(path.clone(), PreviewType::Files)
                    .with_name_match_ranges(
                        indices.map(|i| (i, i + 1)).collect(),
                    )
                    .with_icon(FileIcon::from(&path))
            })
            .collect()
    }

    fn get_result(&self, index: u32) -> Option<Entry> {
        let snapshot = self.matcher.snapshot();
        snapshot.get_matched_item(index).map(|item| {
            let path = item.matcher_columns[0].to_string();
            Entry::new(path.clone(), PreviewType::Files)
                .with_icon(FileIcon::from(&path))
        })
    }

    fn running(&self) -> bool {
        self.running
    }

    fn shutdown(&self) {
        self.crawl_handle.abort();
    }
}

#[allow(clippy::unused_async)]
async fn load_files(path: PathBuf, injector: Injector<DirEntry>) {
    let current_dir = std::env::current_dir().unwrap();
    let walker =
        walk_builder(&path, *DEFAULT_NUM_THREADS, None).build_parallel();

    walker.run(|| {
        let injector = injector.clone();
        let current_dir = current_dir.clone();
        Box::new(move |result| {
            if let Ok(entry) = result {
                if entry.file_type().unwrap().is_file() {
                    let file_name = entry.file_name();
                    if proportion_of_printable_ascii_characters(
                        file_name.as_bytes(),
                    ) < PRINTABLE_ASCII_THRESHOLD
                    {
                        return ignore::WalkState::Continue;
                    }
                    let _ = injector.push(entry, |e, cols| {
                        cols[0] = e
                            .path()
                            .strip_prefix(&current_dir)
                            .unwrap()
                            .to_string_lossy()
                            .into();
                    });
                }
            }
            ignore::WalkState::Continue
        })
    });
}
