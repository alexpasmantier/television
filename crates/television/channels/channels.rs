use std::sync::Arc;

use clap::ValueEnum;
use devicons::FileIcon;
use nucleo::{
    pattern::{CaseMatching, Normalization},
    Config, Nucleo,
};

use crate::{
    channels::{CliTvChannel, TelevisionChannel},
    entry::Entry,
    fuzzy::MATCHER,
    previewers::PreviewType,
};

pub struct SelectionChannel {
    matcher: Nucleo<CliTvChannel>,
    last_pattern: String,
    result_count: u32,
    total_count: u32,
    running: bool,
}

const NUM_THREADS: usize = 1;

const CHANNEL_BLACKLIST: [CliTvChannel; 1] = [CliTvChannel::Stdin];

impl SelectionChannel {
    pub fn new() -> Self {
        let matcher = Nucleo::new(
            Config::DEFAULT,
            Arc::new(|| {}),
            Some(NUM_THREADS),
            1,
        );
        let injector = matcher.injector();
        for variant in CliTvChannel::value_variants() {
            if CHANNEL_BLACKLIST.contains(variant) {
                continue;
            }
            let _ = injector.push(*variant, |e, cols| {
                cols[0] = (*e).to_string().into();
            });
        }
        SelectionChannel {
            matcher,
            last_pattern: String::new(),
            result_count: 0,
            total_count: 0,
            running: false,
        }
    }

    const MATCHER_TICK_TIMEOUT: u64 = 2;
}

const TV_ICON: FileIcon = FileIcon {
    icon: '📺',
    color: "#ffffff",
};

impl TelevisionChannel for SelectionChannel {
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

                let name = item.matcher_columns[0].to_string();
                Entry::new(name.clone(), PreviewType::Basic)
                    .with_name_match_ranges(
                        indices.map(|i| (i, i + 1)).collect(),
                    )
                    .with_icon(TV_ICON)
            })
            .collect()
    }

    fn get_result(&self, index: u32) -> Option<Entry> {
        let snapshot = self.matcher.snapshot();
        snapshot.get_matched_item(index).map(|item| {
            let name = item.matcher_columns[0].to_string();
            // TODO: Add new Previewer for Channel selection which displays a
            // short description of the channel
            Entry::new(name.clone(), PreviewType::Basic).with_icon(TV_ICON)
        })
    }

    fn running(&self) -> bool {
        self.running
    }
}
