use devicons::FileIcon;
use directories::BaseDirs;
use ignore::overrides::OverrideBuilder;
use rustc_hash::{FxBuildHasher, FxHashSet};
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::task::JoinHandle;
use tracing::debug;

use crate::channels::entry::{Entry, PreviewCommand, PreviewType};
use crate::channels::OnAir;
use crate::matcher::{config::Config, injector::Injector, Matcher};
use crate::utils::files::{get_default_num_threads, walk_builder};

pub struct Channel {
    matcher: Matcher<String>,
    icon: FileIcon,
    crawl_handle: JoinHandle<()>,
    selected_entries: FxHashSet<Entry>,
    preview_command: PreviewCommand,
}

impl Channel {
    pub fn new() -> Self {
        let matcher = Matcher::new(Config::default().match_paths(true));
        let base_dirs = BaseDirs::new().unwrap();
        let crawl_handle = tokio::spawn(crawl_for_repos(
            base_dirs.home_dir().to_path_buf(),
            matcher.injector(),
        ));

        let preview_command = PreviewCommand {
            command: String::from(
                "cd {} && git log -n 200 --pretty=medium --all --graph --color",
            ),
            delimiter: ":".to_string(),
        };

        Channel {
            matcher,
            icon: FileIcon::from("git"),
            crawl_handle,
            selected_entries: HashSet::with_hasher(FxBuildHasher),
            preview_command,
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
                let path = item.matched_string;
                Entry::new(
                    path,
                    PreviewType::Command(self.preview_command.clone()),
                )
                .with_name_match_ranges(&item.match_indices)
                .with_icon(self.icon)
            })
            .collect()
    }

    fn get_result(&self, index: u32) -> Option<Entry> {
        self.matcher.get_result(index).map(|item| {
            let path = item.matched_string;
            Entry::new(
                path,
                PreviewType::Command(self.preview_command.clone()),
            )
            .with_icon(self.icon)
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
async fn crawl_for_repos(starting_point: PathBuf, injector: Injector<String>) {
    let mut walker_overrides_builder = OverrideBuilder::new(&starting_point);
    walker_overrides_builder.add(".git").unwrap();
    let walker = walk_builder(
        &starting_point,
        get_default_num_threads(),
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
                        let parent_path =
                            &entry.path().parent().unwrap().to_string_lossy();
                        debug!("Found git repo: {:?}", parent_path);
                        let () = injector.push(
                            parent_path.to_string(),
                            |e, cols| {
                                cols[0] = e.clone().into();
                            },
                        );
                        return ignore::WalkState::Skip;
                    }
                }
            }
            ignore::WalkState::Continue
        })
    });
}
