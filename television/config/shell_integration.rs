use crate::config::parse_key;
use crate::event::Key;
use rustc_hash::FxHashMap;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(default)]
pub struct ShellIntegrationConfig {
    pub commands: FxHashMap<String, String>,
    pub keybindings: FxHashMap<String, String>,
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
                _ => DEFAULT_COMMAND_HISTORY_KEY,
            },
            None => DEFAULT_COMMAND_HISTORY_KEY,
        }
    }
}
