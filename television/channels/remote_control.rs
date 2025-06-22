use crate::{
    cable::Cable,
    channels::{
        entry::into_ranges,
        prototypes::{BinaryRequirement, ChannelPrototype},
    },
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
    pub description: Option<String>,
    pub requirements: Vec<BinaryRequirement>,
}

impl CableEntry {
    pub fn new(name: String, shortcut: Option<&Binding>) -> Self {
        CableEntry {
            channel_name: name,
            match_ranges: None,
            shortcut: shortcut.cloned(),
            description: None,
            requirements: Vec::new(),
        }
    }

    pub fn with_match_indices(mut self, indices: &[u32]) -> Self {
        self.match_ranges = Some(into_ranges(indices));
        self
    }

    pub fn with_description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    pub fn with_requirements(
        mut self,
        requirements: Vec<BinaryRequirement>,
    ) -> Self {
        self.requirements = requirements;
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
    matcher: Matcher<CableEntry>,
    pub cable_channels: Cable,
}

const NUM_THREADS: usize = 1;

impl RemoteControl {
    pub fn new(cable_channels: Cable) -> Self {
        let matcher =
            Matcher::new(&Config::default().n_threads(Some(NUM_THREADS)));
        let injector = matcher.injector();
        for (channel_name, prototype) in cable_channels.iter() {
            let channel_shortcut = prototype
                .keybindings
                .as_ref()
                .and_then(|kb| kb.channel_shortcut());
            let cable_entry =
                CableEntry::new(channel_name.to_string(), channel_shortcut)
                    .with_description(prototype.metadata.description.clone())
                    .with_requirements(
                        // check if the prototype has binary requirements
                        // and whether they are met
                        prototype
                            .metadata
                            .requirements
                            .iter()
                            .cloned()
                            .map(|mut r| {
                                r.init();
                                r
                            })
                            .collect(),
                    );
            let () = injector.push(cable_entry, |e, cols| {
                cols[0] = e.channel_name.clone().into();
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
            .map(|item| item.inner.with_match_indices(&item.match_indices))
            .collect()
    }

    pub fn get_result(&self, index: u32) -> CableEntry {
        let item = self.matcher.get_result(index).expect("Invalid index");
        item.inner.with_match_indices(&item.match_indices)
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
