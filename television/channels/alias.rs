use std::collections::HashSet;

use crate::channels::entry::Entry;
use crate::channels::entry::PreviewType;
use crate::channels::OnAir;
use crate::matcher::{config::Config, injector::Injector, Matcher};
use crate::utils::indices::sep_name_and_value_indices;
use devicons::FileIcon;
use rustc_hash::FxBuildHasher;
use rustc_hash::FxHashSet;
use tracing::debug;

#[derive(Debug, Clone)]
struct Alias {
    name: String,
    value: String,
}

impl Alias {
    fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}

pub struct Channel {
    matcher: Matcher<Alias>,
    file_icon: FileIcon,
    selected_entries: FxHashSet<Entry>,
}

const NUM_THREADS: usize = 1;

const FILE_ICON_STR: &str = "nu";
const SHELL_ENV_VAR: &str = "SHELL";

fn get_current_shell() -> Option<String> {
    std::env::var(SHELL_ENV_VAR).ok()
}

fn get_raw_aliases(shell: &str) -> Vec<String> {
    let output = std::process::Command::new(shell)
        .arg("-i")
        .arg("-c")
        .arg("alias")
        .output()
        .expect("failed to execute process");
    let aliases = String::from_utf8_lossy(&output.stdout);
    aliases.lines().map(ToString::to_string).collect()
}

impl Channel {
    pub fn new() -> Self {
        let matcher = Matcher::new(Config::default().n_threads(NUM_THREADS));
        let injector = matcher.injector();
        tokio::spawn(load_aliases(injector));

        Self {
            matcher,
            file_icon: FileIcon::from(FILE_ICON_STR),
            selected_entries: HashSet::with_hasher(FxBuildHasher),
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
                let (
                    name_indices,
                    value_indices,
                    should_add_name_indices,
                    should_add_value_indices,
                ) = sep_name_and_value_indices(
                    &mut item.match_indices.iter().map(|i| i.0).collect(),
                    u32::try_from(item.inner.name.len()).unwrap(),
                );

                let mut entry =
                    Entry::new(item.inner.name.clone(), PreviewType::EnvVar)
                        .with_value(item.inner.value)
                        .with_icon(self.file_icon);

                if should_add_name_indices {
                    let name_indices: Vec<(u32, u32)> =
                        name_indices.into_iter().map(|i| (i, i + 1)).collect();
                    entry = entry.with_name_match_ranges(&name_indices);
                }

                if should_add_value_indices {
                    let value_indices: Vec<(u32, u32)> = value_indices
                        .into_iter()
                        .map(|i| (i, i + 1))
                        .collect();
                    entry = entry.with_value_match_ranges(&value_indices);
                }

                entry
            })
            .collect()
    }

    fn get_result(&self, index: u32) -> Option<Entry> {
        self.matcher.get_result(index).map(|item| {
            Entry::new(item.inner.name.clone(), PreviewType::EnvVar)
                .with_value(item.inner.value)
                .with_icon(self.file_icon)
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

    fn shutdown(&self) {}

    fn supports_preview(&self) -> bool {
        true
    }
}

#[allow(clippy::unused_async)]
async fn load_aliases(injector: Injector<Alias>) {
    let raw_shell = get_current_shell().unwrap_or("bash".to_string());
    let shell = raw_shell.split('/').last().unwrap();
    debug!("Current shell: {}", shell);
    let raw_aliases = get_raw_aliases(shell);

    raw_aliases
        .iter()
        .filter_map(|alias| {
            let mut parts = alias.split('=');
            if let Some(name) = parts.next() {
                if let Some(value) = parts.next() {
                    return Some(Alias::new(
                        name.to_string(),
                        value.to_string(),
                    ));
                }
            }
            None
        })
        .for_each(|alias| {
            let () = injector.push(alias, |e, cols| {
                cols[0] = (e.name.clone() + &e.value).into();
            });
        });
}
