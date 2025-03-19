use std::collections::HashSet;
use std::fmt::Display;

use crate::channels::cable::{CableChannelPrototype, CableChannels};
use crate::channels::entry::{Entry, PreviewType};
use crate::channels::{CliTvChannel, OnAir, TelevisionChannel, UnitChannel};
use crate::matcher::{config::Config, Matcher};
use anyhow::Result;
use clap::ValueEnum;
use devicons::FileIcon;
use rustc_hash::{FxBuildHasher, FxHashSet};

use super::cable;

pub struct RemoteControl {
    matcher: Matcher<RCButton>,
    cable_channels: Option<CableChannels>,
    selected_entries: FxHashSet<Entry>,
}

#[derive(Clone)]
pub enum RCButton {
    Channel(UnitChannel),
    CableChannel(CableChannelPrototype),
}

impl Display for RCButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RCButton::Channel(channel) => write!(f, "{channel}"),
            RCButton::CableChannel(prototype) => write!(f, "{prototype}"),
        }
    }
}

const NUM_THREADS: usize = 1;

impl RemoteControl {
    pub fn new(
        builtin_channels: Vec<UnitChannel>,
        cable_channels: Option<CableChannels>,
    ) -> Self {
        let matcher = Matcher::new(Config::default().n_threads(NUM_THREADS));
        let injector = matcher.injector();
        let buttons =
            builtin_channels.into_iter().map(RCButton::Channel).chain(
                cable_channels
                    .as_ref()
                    .map(|channels| {
                        channels.iter().map(|(_, prototype)| {
                            RCButton::CableChannel(prototype.clone())
                        })
                    })
                    .into_iter()
                    .flatten(),
            );
        for button in buttons {
            let () = injector.push(button.clone(), |e, cols| {
                cols[0] = e.to_string().into();
            });
        }
        RemoteControl {
            matcher,
            cable_channels,
            selected_entries: HashSet::with_hasher(FxBuildHasher),
        }
    }

    pub fn with_transitions_from(
        television_channel: &TelevisionChannel,
    ) -> Self {
        Self::new(television_channel.available_transitions(), None)
    }

    pub fn zap(&self, channel_name: &str) -> Result<TelevisionChannel> {
        match self
            .cable_channels
            .as_ref()
            .and_then(|channels| channels.get(channel_name).cloned())
        {
            Some(prototype) => {
                Ok(TelevisionChannel::Cable(cable::Channel::from(prototype)))
            }
            None => match UnitChannel::try_from(channel_name) {
                Ok(channel) => Ok(channel.into()),
                Err(_) => Err(anyhow::anyhow!(
                    "No channel or cable channel prototype found for {}",
                    channel_name
                )),
            },
        }
    }
}

impl Default for RemoteControl {
    fn default() -> Self {
        Self::new(
            CliTvChannel::value_variants()
                .iter()
                .flat_map(|v| UnitChannel::try_from(v.to_string().as_str()))
                .collect(),
            None,
        )
    }
}

pub fn load_builtin_channels(
    filter_out_cable_names: Option<&[&String]>,
) -> Vec<UnitChannel> {
    let mut value_variants = CliTvChannel::value_variants()
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>();

    if let Some(f) = filter_out_cable_names {
        value_variants.retain(|v| !f.iter().any(|c| *c == v));
    }

    value_variants
        .iter()
        .flat_map(|v| UnitChannel::try_from(v.as_str()))
        .collect()
}

const TV_ICON: FileIcon = FileIcon {
    icon: '📺',
    color: "#000000",
};

const CABLE_ICON: FileIcon = FileIcon {
    icon: '🍿',
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
                    .with_name_match_ranges(&item.match_indices)
                    .with_icon(match item.inner {
                        RCButton::Channel(_) => TV_ICON,
                        RCButton::CableChannel(_) => CABLE_ICON,
                    })
            })
            .collect()
    }

    fn get_result(&self, index: u32) -> Option<Entry> {
        self.matcher.get_result(index).map(|item| {
            let path = item.matched_string;
            Entry::new(path, PreviewType::Basic).with_icon(TV_ICON)
        })
    }

    fn selected_entries(&self) -> &FxHashSet<Entry> {
        &self.selected_entries
    }

    #[allow(unused_variables)]
    fn toggle_selection(&mut self, entry: &Entry) {}

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

    fn supports_preview(&self) -> bool {
        false
    }
}
