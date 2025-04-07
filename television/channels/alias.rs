use std::collections::HashSet;

use crate::channels::entry::Entry;
use crate::channels::entry::PreviewType;
use crate::channels::OnAir;
use crate::matcher::{config::Config, injector::Injector, Matcher};
use crate::utils::command::shell_command;
use crate::utils::indices::sep_name_and_value_indices;
use crate::utils::shell::Shell;
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
    crawl_handle: tokio::task::JoinHandle<()>,
}

const NUM_THREADS: usize = 1;

const FILE_ICON_STR: &str = "nu";

fn get_raw_aliases(shell: Shell) -> Vec<String> {
    // this needs to be run in an interactive shell in order to get the aliases
    let mut command = shell_command(true);

    let output = match shell {
        Shell::PowerShell => {
            command.arg("Get-Alias | Format-List -Property Name, Definition")
        }
        Shell::Cmd => command.arg("doskey /macros"),
        _ => command.arg("-i").arg("alias").arg("2>/dev/null"),
    }
    .output()
    .expect("failed to execute process");

    let aliases = String::from_utf8_lossy(&output.stdout);
    aliases.lines().map(ToString::to_string).collect()
}

impl Channel {
    pub fn new() -> Self {
        let matcher = Matcher::new(Config::default().n_threads(NUM_THREADS));
        let injector = matcher.injector();
        let crawl_handle = tokio::spawn(load_aliases(injector));

        Self {
            matcher,
            file_icon: FileIcon::from(FILE_ICON_STR),
            selected_entries: HashSet::with_hasher(FxBuildHasher),
            crawl_handle,
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
                    item.match_indices,
                    u32::try_from(item.inner.name.len()).unwrap(),
                );

                let mut entry =
                    Entry::new(item.inner.name.clone(), PreviewType::EnvVar)
                        .with_value(item.inner.value)
                        .with_icon(self.file_icon);

                if should_add_name_indices {
                    entry = entry.with_name_match_indices(&name_indices);
                }

                if should_add_value_indices {
                    entry = entry.with_value_match_indices(&value_indices);
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
        self.matcher.status.running || !self.crawl_handle.is_finished()
    }

    fn shutdown(&self) {}

    fn supports_preview(&self) -> bool {
        true
    }
}

#[allow(clippy::unused_async)]
async fn load_aliases(injector: Injector<Alias>) {
    let shell = Shell::from_env().unwrap_or_default();
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
            } else {
                debug!("Invalid alias format: {}", alias);
            }
            None
        })
        .for_each(|alias| {
            let () = injector.push(alias, |e, cols| {
                cols[0] = (e.name.clone() + &e.value).into();
            });
        });
}
