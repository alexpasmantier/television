use crate::channels::{CliTvChannel, OnAir, TelevisionChannel, UnitChannel};
use crate::entry::{Entry, PreviewType};
use crate::recipes::{load_cook_book, ChannelCookBook, ChannelRecipe};
use clap::ValueEnum;
use color_eyre::Result;
use devicons::FileIcon;
use television_fuzzy::matcher::{config::Config, Matcher};

use super::custom;

pub struct RemoteControl {
    matcher: Matcher<RCButton>,
    cookbook: Option<ChannelCookBook>,
}

#[derive(Clone)]
enum RCButton {
    Channel(UnitChannel),
    ChannelRecipe(ChannelRecipe),
}

impl ToString for RCButton {
    fn to_string(&self) -> String {
        match self {
            RCButton::Channel(channel) => channel.to_string(),
            RCButton::ChannelRecipe(recipe) => recipe.to_string(),
        }
    }
}

const NUM_THREADS: usize = 1;

impl RemoteControl {
    pub fn new(
        buttons: Vec<RCButton>,
        cookbook: Option<ChannelCookBook>,
    ) -> Self {
        let matcher = Matcher::new(Config::default().n_threads(NUM_THREADS));
        let injector = matcher.injector();
        for button in buttons {
            let () = injector.push(button.clone(), |e, cols| {
                cols[0] = e.to_string().clone().into();
            });
        }
        RemoteControl { matcher, cookbook }
    }

    pub fn with_transitions_from(
        television_channel: &TelevisionChannel,
    ) -> Self {
        Self::new(
            television_channel
                .available_transitions()
                .into_iter()
                .map(RCButton::Channel)
                .collect(),
            None,
        )
    }

    pub fn zap(&self, channel_name: &str) -> Result<TelevisionChannel> {
        match UnitChannel::try_from(channel_name) {
            Ok(channel) => Ok(channel.into()),
            Err(_) => {
                let recipe = self
                    .cookbook
                    .as_ref()
                    .and_then(|cookbook| cookbook.get(channel_name));
                match recipe {
                    Some(recipe) => Ok(TelevisionChannel::Custom(
                        custom::Channel::from(recipe.clone()),
                    )),
                    None => Err(color_eyre::eyre::eyre!(
                        "No channel or recipe found for {}",
                        channel_name
                    )),
                }
            }
        }
    }
}

impl Default for RemoteControl {
    fn default() -> Self {
        let cookbook = load_cook_book().expect("Failed to load cookbook");
        Self::new(
            CliTvChannel::value_variants()
                .iter()
                .flat_map(|v| UnitChannel::try_from(v.to_string().as_str()))
                .map(RCButton::Channel)
                .chain(
                    cookbook
                        .iter()
                        .map(|(_, v)| RCButton::ChannelRecipe(v.clone())),
                )
                .collect(),
            Some(cookbook),
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
