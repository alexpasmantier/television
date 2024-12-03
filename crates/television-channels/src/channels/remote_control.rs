use crate::cable::{CableChannelPrototype, CableChannels};
use crate::channels::{CliTvChannel, OnAir, TelevisionChannel, UnitChannel};
use crate::entry::{Entry, PreviewType};
use clap::ValueEnum;
use color_eyre::Result;
use devicons::FileIcon;
use television_fuzzy::matcher::{config::Config, Matcher};
use tracing::debug;

use super::custom;

pub struct RemoteControl {
    matcher: Matcher<RCButton>,
    cable_channels: Option<CableChannels>,
}

#[derive(Clone)]
pub enum RCButton {
    Channel(UnitChannel),
    CableChannel(CableChannelPrototype),
}

impl ToString for RCButton {
    fn to_string(&self) -> String {
        match self {
            RCButton::Channel(channel) => channel.to_string(),
            RCButton::CableChannel(prototype) => prototype.to_string(),
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
                cols[0] = e.to_string().clone().into();
            });
        }
        RemoteControl {
            matcher,
            cable_channels,
        }
    }

    pub fn with_transitions_from(
        television_channel: &TelevisionChannel,
    ) -> Self {
        Self::new(television_channel.available_transitions(), None)
    }

    pub fn zap(&self, channel_name: &str) -> Result<TelevisionChannel> {
        if let Ok(channel) = UnitChannel::try_from(channel_name) {
            Ok(channel.into())
        } else {
            let maybe_prototype = self
                .cable_channels
                .as_ref()
                .and_then(|channels| channels.get(channel_name));
            match maybe_prototype {
                Some(prototype) => Ok(TelevisionChannel::Custom(
                    custom::Channel::from(prototype.clone()),
                )),
                None => Err(color_eyre::eyre::eyre!(
                    "No channel or cable channel prototype found for {}",
                    channel_name
                )),
            }
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

pub fn load_builtin_channels() -> Vec<UnitChannel> {
    CliTvChannel::value_variants()
        .iter()
        .flat_map(|v| UnitChannel::try_from(v.to_string().as_str()))
        .collect()
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
