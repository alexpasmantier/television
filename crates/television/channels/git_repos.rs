use devicons::FileIcon;
use directories::BaseDirs;
use ignore::overrides::OverrideBuilder;
use nucleo::{
    pattern::{CaseMatching, Normalization},
    Config, Nucleo,
};
use std::{path::PathBuf, sync::Arc};
use tokio::task::JoinHandle;
use tracing::debug;

use crate::{
    entry::Entry,
    fuzzy::MATCHER,
    previewers::PreviewType,
    utils::files::{walk_builder, DEFAULT_NUM_THREADS},
};

use crate::channels::OnAir;
use crate::utils::strings::preprocess_line;

pub struct Channel {
    matcher: Nucleo<String>,
    last_pattern: String,
    result_count: u32,
    total_count: u32,
    running: bool,
    icon: FileIcon,
    crawl_handle: JoinHandle<()>,
}

impl Channel {
    pub fn new() -> Self {
        let matcher = Nucleo::new(
            Config::DEFAULT.match_paths(),
            Arc::new(|| {}),
            None,
            1,
        );
        let base_dirs = BaseDirs::new().unwrap();
        let crawl_handle = tokio::spawn(crawl_for_repos(
            base_dirs.home_dir().to_path_buf(),
            matcher.injector(),
        ));
        Channel {
            matcher,
            last_pattern: String::new(),
            result_count: 0,
            total_count: 0,
            running: false,
            icon: FileIcon::from("git"),
            crawl_handle,
        }
    }

    const MATCHER_TICK_TIMEOUT: u64 = 2;
}

impl Default for Channel {
    fn default() -> Self {
        Self::new()
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

                let path = item.matcher_columns[0].to_string();
                Entry::new(path.clone(), PreviewType::Directory)
                    .with_name_match_ranges(
                        indices.map(|i| (i, i + 1)).collect(),
                    )
                    .with_icon(icon)
            })
            .collect()
    }

    fn get_result(&self, index: u32) -> Option<Entry> {
        let snapshot = self.matcher.snapshot();
        snapshot.get_matched_item(index).map(|item| {
            let path = item.matcher_columns[0].to_string();
            Entry::new(path.clone(), PreviewType::Directory)
                .with_icon(self.icon)
        })
    }

    fn result_count(&self) -> u32 {
        self.result_count
    }

    fn total_count(&self) -> u32 {
        self.total_count
    }

    fn running(&self) -> bool {
        self.running
    }

    fn shutdown(&self) {
        debug!("Shutting down git repos channel");
        self.crawl_handle.abort();
    }
}

fn get_ignored_paths() -> Vec<PathBuf> {
    let mut ignored_paths = Vec::new();

    if let Some(base_dirs) = BaseDirs::new() {
        let home = base_dirs.home_dir();

        #[cfg(target_os = "macos")]
        {
            ignored_paths.push(home.join("Library"));
            ignored_paths.push(home.join("Applications"));
            ignored_paths.push(home.join("Music"));
            ignored_paths.push(home.join("Pictures"));
            ignored_paths.push(home.join("Movies"));
            ignored_paths.push(home.join("Downloads"));
            ignored_paths.push(home.join("Public"));
        }

        #[cfg(target_os = "linux")]
        {
            ignored_paths.push(home.join(".cache"));
            ignored_paths.push(home.join(".config"));
            ignored_paths.push(home.join(".local"));
            ignored_paths.push(home.join(".thumbnails"));
            ignored_paths.push(home.join("Downloads"));
            ignored_paths.push(home.join("Public"));
            ignored_paths.push(home.join("snap"));
            ignored_paths.push(home.join(".snap"));
        }

        #[cfg(target_os = "windows")]
        {
            ignored_paths.push(home.join("AppData"));
            ignored_paths.push(home.join("Downloads"));
            ignored_paths.push(home.join("Documents"));
            ignored_paths.push(home.join("Music"));
            ignored_paths.push(home.join("Pictures"));
            ignored_paths.push(home.join("Videos"));
        }

        // Common paths to ignore for all platforms
        ignored_paths.push(home.join("node_modules"));
        ignored_paths.push(home.join("venv"));
        ignored_paths.push(PathBuf::from("/tmp"));
    }

    ignored_paths
}
#[allow(clippy::unused_async)]
async fn crawl_for_repos(
    starting_point: PathBuf,
    injector: nucleo::Injector<String>,
) {
    let mut walker_overrides_builder = OverrideBuilder::new(&starting_point);
    walker_overrides_builder.add(".git").unwrap();
    let walker = walk_builder(
        &starting_point,
        *DEFAULT_NUM_THREADS,
        Some(walker_overrides_builder.build().unwrap()),
        Some(get_ignored_paths()),
    )
    .build_parallel();

    walker.run(|| {
        let injector = injector.clone();
        Box::new(move |result| {
            if let Ok(entry) = result {
                if entry.file_type().unwrap().is_dir() {
                    // if the entry is a .git directory, add its parent to the list of git repos
                    if entry.path().ends_with(".git") {
                        let parent_path = preprocess_line(
                            &entry.path().parent().unwrap().to_string_lossy(),
                        );
                        debug!("Found git repo: {:?}", parent_path);
                        let _ = injector.push(parent_path, |e, cols| {
                            cols[0] = e.clone().into();
                        });
                        return ignore::WalkState::Skip;
                    }
                }
            }
            ignore::WalkState::Continue
        })
    });
}
