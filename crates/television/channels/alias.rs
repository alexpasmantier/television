use std::sync::Arc;

use devicons::FileIcon;
use nucleo::{Config, Injector, Nucleo};
use tracing::debug;

use crate::channels::OnAir;
use crate::entry::Entry;
use crate::fuzzy::MATCHER;
use crate::previewers::PreviewType;
use crate::utils::indices::sep_name_and_value_indices;
use crate::utils::strings::preprocess_line;

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
    matcher: Nucleo<Alias>,
    last_pattern: String,
    file_icon: FileIcon,
    result_count: u32,
    total_count: u32,
    running: bool,
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
    let aliases = String::from_utf8(output.stdout).unwrap();
    aliases.lines().map(ToString::to_string).collect()
}

impl Channel {
    pub fn new() -> Self {
        let matcher = Nucleo::new(
            Config::DEFAULT,
            Arc::new(|| {}),
            Some(NUM_THREADS),
            1,
        );
        let injector = matcher.injector();
        tokio::spawn(load_aliases(injector));

        Self {
            matcher,
            last_pattern: String::new(),
            file_icon: FileIcon::from(FILE_ICON_STR),
            result_count: 0,
            total_count: 0,
            running: false,
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
        self.running = status.running;
        let mut col_indices = Vec::new();
        let mut matcher = MATCHER.lock();
        let icon = self.file_icon;

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
                    &mut col_indices,
                );
                col_indices.sort_unstable();
                col_indices.dedup();

                let (
                    name_indices,
                    value_indices,
                    should_add_name_indices,
                    should_add_value_indices,
                ) = sep_name_and_value_indices(
                    &mut col_indices,
                    u32::try_from(item.data.name.len()).unwrap(),
                );

                let mut entry =
                    Entry::new(item.data.name.clone(), PreviewType::EnvVar)
                        .with_value(item.data.value.clone())
                        .with_icon(icon);

                if should_add_name_indices {
                    entry = entry.with_name_match_ranges(
                        name_indices.into_iter().map(|i| (i, i + 1)).collect(),
                    );
                }

                if should_add_value_indices {
                    entry = entry.with_value_match_ranges(
                        value_indices
                            .into_iter()
                            .map(|i| (i, i + 1))
                            .collect(),
                    );
                }

                entry
            })
            .collect()
    }

    fn get_result(&self, index: u32) -> Option<Entry> {
        let snapshot = self.matcher.snapshot();
        snapshot.get_matched_item(index).map(|item| {
            Entry::new(item.data.name.clone(), PreviewType::EnvVar)
                .with_value(item.data.value.clone())
                .with_icon(self.file_icon)
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

    fn shutdown(&self) {}
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
                        preprocess_line(name),
                        preprocess_line(value),
                    ));
                }
            }
            None
        })
        .for_each(|alias| {
            let _ = injector.push(alias.clone(), |_, cols| {
                cols[0] = (alias.name.clone() + &alias.value).into();
            });
        });
}
