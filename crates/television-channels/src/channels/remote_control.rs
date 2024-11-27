use crate::channels::{CliTvChannel, OnAir, TelevisionChannel, UnitChannel};
use crate::entry::{Entry, PreviewType};
use clap::ValueEnum;
use devicons::FileIcon;
use television_fuzzy::matcher::{config::Config, Matcher};

pub struct RemoteControl {
    matcher: Matcher<String>,
}

const NUM_THREADS: usize = 1;

impl RemoteControl {
    pub fn new(channels: Vec<UnitChannel>) -> Self {
        let matcher = Matcher::new(Config::default().n_threads(NUM_THREADS));
        let injector = matcher.injector();
        for channel in channels {
            let () = injector.push(channel.to_string(), |e, cols| {
                cols[0] = e.clone().into();
            });
        }
        RemoteControl { matcher }
    }

    pub fn with_transitions_from(
        television_channel: &TelevisionChannel,
    ) -> Self {
        Self::new(television_channel.available_transitions())
    }
}

impl Default for RemoteControl {
    fn default() -> Self {
        Self::new(
            CliTvChannel::value_variants()
                .iter()
                .map(|v| v.to_string().as_str().into())
                .collect(),
        )
    }
}

const TV_ICON: FileIcon = FileIcon {
    icon: '📺',
    color: "#000000",
};

impl OnAir for RemoteControl {
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
                Entry::new(path, PreviewType::Basic)
                    .with_name_match_ranges(item.match_indices)
                    .with_icon(TV_ICON)
            })
            .collect()
    }

    fn get_result(&self, index: u32) -> Option<Entry> {
        self.matcher.get_result(index).map(|item| {
            let path = item.matched_string;
            Entry::new(path, PreviewType::Basic).with_icon(TV_ICON)
        })
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
}
