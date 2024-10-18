use std::sync::Arc;

use devicons::FileIcon;
use ignore::{overrides::OverrideBuilder, DirEntry};
use nucleo::{
    pattern::{CaseMatching, Normalization},
    Config, Nucleo,
};
use tracing::debug;

use crate::{
    entry::Entry,
    fuzzy::MATCHER,
    previewers::PreviewType,
    utils::files::{walk_builder, DEFAULT_NUM_THREADS},
};

use crate::channels::TelevisionChannel;

pub struct Channel {
    matcher: Nucleo<DirEntry>,
    last_pattern: String,
    result_count: u32,
    total_count: u32,
    running: bool,
}

impl Channel {
    pub fn new() -> Self {
        let matcher = Nucleo::new(
            Config::DEFAULT.match_paths(),
            Arc::new(|| {}),
            None,
            1,
        );
        // start loading files in the background
        tokio::spawn(crawl_for_repos(
            std::env::home_dir().expect("Could not get home directory"),
            matcher.injector(),
        ));
        Channel {
            matcher,
            last_pattern: String::new(),
            result_count: 0,
            total_count: 0,
            running: false,
        }
    }

    const MATCHER_TICK_TIMEOUT: u64 = 10;
}

impl TelevisionChannel for Channel {
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
                Entry::new(path.clone(), PreviewType::Directory)
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
            Entry::new(path.clone(), PreviewType::Directory)
                .with_icon(FileIcon::from(&path))
        })
    }

    fn running(&self) -> bool {
        self.running
    }
}

#[allow(clippy::unused_async)]
async fn crawl_for_repos(
    starting_point: std::path::PathBuf,
    injector: nucleo::Injector<DirEntry>,
) {
    let mut walker_overrides_builder = OverrideBuilder::new(&starting_point);
    walker_overrides_builder.add(".git").unwrap();
    let walker = walk_builder(
        &starting_point,
        *DEFAULT_NUM_THREADS,
        Some(walker_overrides_builder.build().unwrap()),
    )
    .build_parallel();

    walker.run(|| {
        let injector = injector.clone();
        Box::new(move |result| {
            if let Ok(entry) = result {
                if entry.file_type().unwrap().is_dir()
                    && entry.path().ends_with(".git")
                {
                    debug!("Found git repo: {:?}", entry.path());
                    let _ = injector.push(entry, |e, cols| {
                        cols[0] = e
                            .path()
                            .parent()
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
