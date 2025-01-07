use crate::channels::{OnAir, TelevisionChannel};
use crate::entry::{Entry, PreviewType};
use devicons::FileIcon;
use rustc_hash::{FxBuildHasher, FxHashSet};
use std::collections::HashSet;
use std::path::PathBuf;
use television_fuzzy::matcher::{config::Config, injector::Injector, Matcher};
use television_utils::files::{walk_builder, DEFAULT_NUM_THREADS};

pub struct Channel {
    matcher: Matcher<String>,
    crawl_handle: tokio::task::JoinHandle<()>,
    // PERF: cache results (to make deleting characters smoother) with
    // a shallow stack of sub-patterns as keys (e.g. "a", "ab", "abc")
    selected_entries: FxHashSet<Entry>,
}

impl Channel {
    pub fn new(paths: Vec<PathBuf>) -> Self {
        let matcher = Matcher::new(Config::default().match_paths(true));
        // start loading files in the background
        let crawl_handle = tokio::spawn(load_files(paths, matcher.injector()));
        Channel {
            matcher,
            crawl_handle,
            selected_entries: HashSet::with_hasher(FxBuildHasher),
        }
    }
}

impl Default for Channel {
    fn default() -> Self {
        Self::new(vec![std::env::current_dir().unwrap()])
    }
}

impl From<&mut TelevisionChannel> for Channel {
    fn from(value: &mut TelevisionChannel) -> Self {
        match value {
            c @ TelevisionChannel::GitRepos(_) => {
                let entries = if c.selected_entries().is_empty() {
                    c.results(c.result_count(), 0)
                } else {
                    c.selected_entries().iter().cloned().collect()
                };
                Self::new(
                    entries
                        .iter()
                        .map(|entry| PathBuf::from(entry.name.clone()))
                        .collect(),
                )
            }
            c @ TelevisionChannel::Files(_) => {
                let entries = if c.selected_entries().is_empty() {
                    c.results(c.result_count(), 0)
                } else {
                    c.selected_entries().iter().cloned().collect()
                };
                Self::new(
                    entries
                        .iter()
                        .map(|entry| PathBuf::from(entry.name.clone()))
                        .collect(),
                )
            }
            c @ TelevisionChannel::Text(_) => {
                let entries = if c.selected_entries().is_empty() {
                    c.results(c.result_count(), 0)
                } else {
                    c.selected_entries().iter().cloned().collect()
                };
                Self::new(
                    entries
                        .iter()
                        .map(|entry| PathBuf::from(&entry.name))
                        .collect::<FxHashSet<_>>()
                        .into_iter()
                        .collect(),
                )
            }
            c @ TelevisionChannel::Dirs(_) => {
                let entries = c.results(c.result_count(), 0);
                Self::new(
                    entries
                        .iter()
                        .map(|entry| PathBuf::from(&entry.name))
                        .collect::<FxHashSet<_>>()
                        .into_iter()
                        .collect(),
                )
            }
            _ => unreachable!(),
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
                let path = item.matched_string;
                Entry::new(path.clone(), PreviewType::Files)
                    .with_name_match_ranges(&item.match_indices)
                    .with_icon(FileIcon::from(&path))
            })
            .collect()
    }

    fn get_result(&self, index: u32) -> Option<Entry> {
        self.matcher.get_result(index).map(|item| {
            let path = item.matched_string;
            Entry::new(path.clone(), PreviewType::Files)
                .with_icon(FileIcon::from(&path))
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
        self.crawl_handle.abort();
    }
}

#[allow(clippy::unused_async)]
async fn load_files(paths: Vec<PathBuf>, injector: Injector<String>) {
    if paths.is_empty() {
        return;
    }
    let current_dir = std::env::current_dir().unwrap();
    let mut builder =
        walk_builder(&paths[0], *DEFAULT_NUM_THREADS, None, None);
    paths[1..].iter().for_each(|path| {
        builder.add(path);
    });
    let walker = builder.build_parallel();

    walker.run(|| {
        let injector = injector.clone();
        let current_dir = current_dir.clone();
        Box::new(move |result| {
            if let Ok(entry) = result {
                if entry.file_type().unwrap().is_file() {
                    let file_path = &entry
                        .path()
                        .strip_prefix(&current_dir)
                        .unwrap_or(entry.path())
                        .to_string_lossy();
                    let () =
                        injector.push(file_path.to_string(), |e, cols| {
                            cols[0] = e.clone().into();
                        });
                }
            }
            ignore::WalkState::Continue
        })
    });
}
