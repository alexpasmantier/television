use crate::config::parse_key;
use crate::{
    cable::Cable,
    channels::{
        entry::Entry,
        prototypes::{ChannelKeyBindings, ChannelPrototype},
    },
    matcher::{Matcher, config::Config},
};
use devicons::FileIcon;
use rustc_hash::FxHashMap;
use unicode_width::UnicodeWidthStr;

pub struct RemoteControl {
    matcher: Matcher<String>,
    cable_channels: Cable,
    display_map: FxHashMap<String, String>,
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
        let max_name_width = cable_channels
            .keys()
            .map(|n| UnicodeWidthStr::width(n.as_str()))
            .max()
            .unwrap_or(0);

        // Precompute display strings for each channel
        let mut display_map: FxHashMap<String, String> = FxHashMap::default();
        for (channel_name, proto) in cable_channels.iter() {
            let formatted_shortcut = proto
                .keybindings
                .as_ref()
                .and_then(|kb| kb.shortcut_key())
                .and_then(|s| parse_key(s).ok().map(|k| k.to_string()))
                .unwrap_or_default();

            let padding = if UnicodeWidthStr::width(channel_name.as_str())
                < max_name_width
            {
                " ".repeat(
                    max_name_width
                        - UnicodeWidthStr::width(channel_name.as_str())
                        + 1,
                )
            } else {
                " ".to_string()
            };

            let display = if formatted_shortcut.is_empty() {
                channel_name.clone()
            } else {
                format!("{channel_name}{padding} {formatted_shortcut}")
            };

            display_map.insert(channel_name.clone(), display);
        }
        RemoteControl {
            matcher,
            cable_channels,
            display_map,
        }
    }

    pub fn zap(&self, channel_name: &str) -> ChannelPrototype {
        self.cable_channels.get_channel(channel_name)
    }

    /// Iterate over channels with their channel-level keybindings.
    /// Returns only those with keybindings present.
    pub fn shortcuts_iter(
        &self,
    ) -> impl Iterator<Item = (&String, &ChannelKeyBindings)> {
        self.cable_channels.iter().filter_map(|(name, proto)| {
            proto.keybindings.as_ref().map(|kb| (name, kb))
        })
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
                let channel_name = &item.matched_string;
                let display = self
                    .display_map
                    .get(channel_name)
                    .cloned()
                    .unwrap_or_else(|| channel_name.clone());

                Entry::new(channel_name.clone())
                    .with_match_indices(&item.match_indices)
                    .with_icon(CABLE_ICON)
                    .with_display(display)
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
