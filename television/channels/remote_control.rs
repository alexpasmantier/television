use crate::{
    cable::Cable,
    channels::{entry::into_ranges, prototypes::ChannelPrototype},
    config::Binding,
    matcher::{Matcher, config::Config},
    screen::result_item::ResultItem,
};
use devicons::FileIcon;

#[derive(Debug, Clone)]
pub struct CableEntry {
    pub channel_name: String,
    pub match_ranges: Option<Vec<(u32, u32)>>,
    pub shortcut: Option<Binding>,
}

impl CableEntry {
    pub fn new(name: String, shortcut: Option<Binding>) -> Self {
        CableEntry {
            channel_name: name,
            match_ranges: None,
            shortcut,
        }
    }

    pub fn with_match_indices(mut self, indices: &[u32]) -> Self {
        self.match_ranges = Some(into_ranges(indices));
        self
    }
}

impl ResultItem for CableEntry {
    fn icon(&self) -> Option<&devicons::FileIcon> {
        // Remote control entries always share the same popcorn icon
        Some(&crate::channels::remote_control::CABLE_ICON)
    }

    fn display(&self) -> &str {
        &self.channel_name
    }

    fn match_ranges(&self) -> Option<&[(u32, u32)]> {
        self.match_ranges.as_deref()
    }

    fn shortcut(&self) -> Option<&crate::config::Binding> {
        self.shortcut.as_ref()
    }
}

pub struct RemoteControl {
    matcher: Matcher<String>,
    pub cable_channels: Cable,
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
}

pub const CABLE_ICON: FileIcon = FileIcon {
    icon: '🍿',
    color: "#000000",
};

impl RemoteControl {
    pub fn find(&mut self, pattern: &str) {
        self.matcher.find(pattern);
    }

    pub fn results(
        &mut self,
        num_entries: u32,
        offset: u32,
    ) -> Vec<CableEntry> {
        self.matcher.tick();
        self.matcher
            .results(num_entries, offset)
            .into_iter()
            .map(|item| {
                CableEntry::new(
                    item.matched_string.clone(),
                    self.cable_channels
                        .get_channel_shortcut(&item.matched_string),
                )
                .with_match_indices(&item.match_indices)
            })
            .collect()
    }

    pub fn get_result(&self, index: u32) -> Option<CableEntry> {
        self.matcher.get_result(index).map(|item| {
            CableEntry::new(
                item.matched_string.clone(),
                self.cable_channels
                    .get_channel_shortcut(&item.matched_string),
            )
            .with_match_indices(&item.match_indices)
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
