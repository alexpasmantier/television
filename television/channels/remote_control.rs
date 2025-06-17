use crate::{
    cable::Cable,
    channels::{entry::Entry, prototypes::ChannelPrototype},
    matcher::{Matcher, config::Config},
};
use devicons::FileIcon;

pub struct RemoteControl {
    matcher: Matcher<String>,
    cable_channels: Cable,
}

const NUM_THREADS: usize = 1;

impl RemoteControl {
    pub fn new(cable_channels: Cable) -> Self {
        let matcher = Matcher::new(Config::default().n_threads(NUM_THREADS));
        let injector = matcher.injector();
        for c in cable_channels.keys() {
            let () = injector.push(c.clone(), |e, cols| {
                cols[0] = e.to_string().into();
            });
        }
        RemoteControl {
            matcher,
            cable_channels,
        }
    }

    pub fn zap(&self, channel_name: &str) -> ChannelPrototype {
        self.cable_channels.get_channel(channel_name)
    }

    pub fn has_channel(&self, channel_name: &str) -> bool {
        self.cable_channels.has_channel(channel_name)
    }
}

const TV_ICON: FileIcon = FileIcon {
    icon: '📺',
    color: "#000000",
};

const CABLE_ICON: FileIcon = FileIcon {
    icon: '🍿',
    color: "#000000",
};

impl RemoteControl {
    pub fn find(&mut self, pattern: &str) {
        self.matcher.find(pattern);
    }

    pub fn results(&mut self, num_entries: u32, offset: u32) -> Vec<Entry> {
        self.matcher.tick();
        self.matcher
            .results(num_entries, offset)
            .into_iter()
            .map(|item| {
                let path = item.matched_string;
                Entry::new(path)
                    .with_match_indices(&item.match_indices)
                    .with_icon(CABLE_ICON)
            })
            .collect()
    }

    pub fn get_result(&self, index: u32) -> Option<Entry> {
        self.matcher.get_result(index).map(|item| {
            let path = item.matched_string;
            Entry::new(path).with_icon(TV_ICON)
        })
    }

    pub fn result_count(&self) -> u32 {
        self.matcher.matched_item_count
    }

    pub fn total_count(&self) -> u32 {
        self.matcher.total_item_count
    }

    pub fn running(&self) -> bool {
        self.matcher.status.running
    }

    pub fn shutdown(&self) {}

    pub fn supports_preview(&self) -> bool {
        false
    }
}
