use crate::channels::cable::prototypes::Cable;
use crate::channels::entry::Entry;
use crate::matcher::{config::Config, Matcher};
use anyhow::Result;
use devicons::FileIcon;

use super::cable::prototypes::ChannelPrototype;

pub struct RemoteControl {
    matcher: Matcher<String>,
    cable_channels: Option<Cable>,
}

const NUM_THREADS: usize = 1;

impl RemoteControl {
    pub fn new(cable_channels: Option<Cable>) -> Self {
        let matcher = Matcher::new(Config::default().n_threads(NUM_THREADS));
        let injector = matcher.injector();
        for c in cable_channels.as_ref().unwrap_or(&Cable::default()).keys() {
            let () = injector.push(c.clone(), |e, cols| {
                cols[0] = e.to_string().into();
            });
        }
        RemoteControl {
            matcher,
            cable_channels,
        }
    }

    pub fn zap(&self, channel_name: &str) -> Result<ChannelPrototype> {
        match self
            .cable_channels
            .as_ref()
            .and_then(|channels| channels.get(channel_name).cloned())
        {
            Some(prototype) => Ok(prototype),
            None => Err(anyhow::anyhow!(
                "No channel or cable channel prototype found for {}",
                channel_name
            )),
        }
    }
}

impl Default for RemoteControl {
    fn default() -> Self {
        Self::new(None)
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
                    .with_name_match_indices(&item.match_indices)
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
