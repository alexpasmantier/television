use std::hash::Hash;

use crate::config::parse_key;
use crate::event::Key;
use crate::utils::hashmaps;
use rustc_hash::FxHashMap;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct ShellIntegrationConfig {
    /// DEPRECATED: This is a legacy configuration option that is no longer used.
    /// It is kept here for backwards compatibility.
    /// {command: channel}
    pub commands: FxHashMap<String, String>,
    /// {channel: [commands]}
    pub channel_triggers: FxHashMap<String, Vec<String>>,
    pub keybindings: FxHashMap<String, String>,
}

impl Hash for ShellIntegrationConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // we're not actually using this for hashing, so this really only is a placeholder
        state.write_u8(0);
    }
}

impl ShellIntegrationConfig {
    /// Merge the channel triggers into the commands hashmap
    /// This is done to maintain backwards compatibility with the old configuration
    /// format.
    ///
    /// {command: channel} + {channel: [commands]} => {command: channel}
    pub fn merge_triggers(&mut self) {
        // invert the hashmap to get {command: channel}
        let inverted_triggers =
            hashmaps::invert_hashmap(&self.channel_triggers);
        // merge the inverted hashmap with the existing commands hashmap
        self.commands.extend(inverted_triggers);
    }
}

const SMART_AUTOCOMPLETE_CONFIGURATION_KEY: &str = "smart_autocomplete";
const COMMAND_HISTORY_CONFIGURATION_KEY: &str = "command_history";
const DEFAULT_SHELL_AUTOCOMPLETE_KEY: char = 'T';
const DEFAULT_COMMAND_HISTORY_KEY: char = 'R';

impl ShellIntegrationConfig {
    // based on the keybindings configuration provided in the configuration file
    // (if any), extract the character triggers shell autocomplete
    pub fn get_shell_autocomplete_keybinding_character(&self) -> char {
        match self.keybindings.get(SMART_AUTOCOMPLETE_CONFIGURATION_KEY) {
            Some(s) => match parse_key(s) {
                Ok(Key::Ctrl(c)) => c.to_uppercase().next().unwrap(),
                Ok(Key::CtrlSpace) => ' ',
                _ => DEFAULT_SHELL_AUTOCOMPLETE_KEY,
            },
            None => DEFAULT_SHELL_AUTOCOMPLETE_KEY,
        }
    }
    // based on the keybindings configuration provided in the configuration file
    // (if any), extract the character triggers command history management
    // through tv
    pub fn get_command_history_keybinding_character(&self) -> char {
        match self.keybindings.get(COMMAND_HISTORY_CONFIGURATION_KEY) {
            Some(s) => match parse_key(s) {
                Ok(Key::Ctrl(c)) => c.to_uppercase().next().unwrap(),
                Ok(Key::CtrlSpace) => ' ',
                _ => DEFAULT_COMMAND_HISTORY_KEY,
            },
            None => DEFAULT_COMMAND_HISTORY_KEY,
        }
    }
}
