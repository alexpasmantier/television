use devicons::FileIcon;
use nucleo::{
    pattern::{CaseMatching, Normalization},
    Config, Nucleo,
};
use std::sync::Arc;

use super::TelevisionChannel;
use crate::entry::Entry;
use crate::fuzzy::MATCHER;
use crate::previewers::PreviewType;
use crate::utils::indices::sep_name_and_value_indices;

struct EnvVar {
    name: String,
    value: String,
}

#[allow(clippy::module_name_repetitions)]
pub(crate) struct Channel {
    matcher: Nucleo<EnvVar>,
    last_pattern: String,
    file_icon: FileIcon,
    result_count: u32,
    total_count: u32,
    running: bool,
}

const NUM_THREADS: usize = 1;
const FILE_ICON_STR: &str = "config";

impl Channel {
    pub(crate) fn new() -> Self {
        let matcher = Nucleo::new(
            Config::DEFAULT,
            Arc::new(|| {}),
            Some(NUM_THREADS),
            1,
        );
        let injector = matcher.injector();
        for (name, value) in std::env::vars() {
            let _ = injector.push(EnvVar { name, value }, |e, cols| {
                cols[0] = (e.name.clone() + &e.value).into();
            });
        }
        Channel {
            matcher,
            last_pattern: String::new(),
            file_icon: FileIcon::from(FILE_ICON_STR),
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
            let name = item.data.name.clone();
            let value = item.data.value.clone();
            Entry::new(name, PreviewType::EnvVar)
                .with_value(value)
                .with_icon(self.file_icon)
        })
    }

    fn running(&self) -> bool {
        self.running
    }
}
