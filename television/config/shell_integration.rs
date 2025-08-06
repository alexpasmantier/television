use std::hash::Hash;

use crate::{event::Key, utils::hashmaps};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(default)]
pub struct ShellIntegrationConfig {
    /// {command: channel}
    #[serde(skip)]
    pub commands: FxHashMap<String, String>,
    pub fallback_channel: String,
    pub keybindings: FxHashMap<String, Key>,
}

impl<'de> Deserialize<'de> for ShellIntegrationConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct DeSerHelper {
            channel_triggers: Option<FxHashMap<String, Vec<String>>>,
            fallback_channel: Option<String>,
            keybindings: Option<FxHashMap<String, Key>>,
        }

        let helper = DeSerHelper::deserialize(deserializer)?;

        let mut config = Self::default();

        if let Some(triggers) = helper.channel_triggers {
            let commands = hashmaps::invert_nested_hashmap(&triggers);
            config.commands = commands;
        }
        if let Some(fallback) = helper.fallback_channel {
            config.fallback_channel = fallback;
        }
        if let Some(keybindings) = helper.keybindings {
            config.keybindings = keybindings;
        }

        Ok(config)
    }
}

const DEFAULT_FALLBACK_CHANNEL: &str = "files";

static DEFAULT_CHANNEL_TRIGGERS: &[(&str, &[&str])] = &[
    ("alias", &["alias", "unalias"]),
    ("env", &["export", "unset"]),
    ("dirs", &["cd", "ls", "rmdir"]),
    (
        "files",
        &[
            "cat", "less", "head", "tail", "vim", "nano", "bat", "cp", "mv",
            "rm", "touch", "chmod", "chown", "ln", "tar", "zip", "unzip",
            "gzip", "gunzip", "xz",
        ],
    ),
    ("git-diff", &["git add", "git restore"]),
    (
        "git-branch",
        &[
            "git checkout",
            "git branch",
            "git merge",
            "git rebase",
            "git pull",
            "git push",
            "git switch",
        ],
    ),
    ("git-log", &["git log", "git show"]),
    ("docker-images", &["docker run"]),
    ("git-repos", &["nvim", "code", "hx", "git clone"]),
];

static DEFAULT_KEYBINDINGS: &[(&str, Key)] = &[
    ("smart_autocomplete", Key::Ctrl('t')),
    ("command_history", Key::Ctrl('r')),
];

impl Default for ShellIntegrationConfig {
    fn default() -> Self {
        let mut triggers: FxHashMap<String, Vec<String>> =
            FxHashMap::default();
        triggers.extend(DEFAULT_CHANNEL_TRIGGERS.iter().map(
            |(channel, commands)| {
                (
                    (*channel).to_string(),
                    commands
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect(),
                )
            },
        ));

        Self {
            commands: hashmaps::invert_nested_hashmap(&triggers),
            fallback_channel: DEFAULT_FALLBACK_CHANNEL.to_string(),
            keybindings: DEFAULT_KEYBINDINGS
                .iter()
                .map(|(name, key)| ((*name).to_string(), *key))
                .collect(),
        }
    }
}

impl Hash for ShellIntegrationConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // we're not actually using this for hashing, so this really only is a placeholder
        state.write_u8(0);
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
            Some(&key) => extract_ctrl_char(key)
                .unwrap_or(DEFAULT_SHELL_AUTOCOMPLETE_KEY),
            None => DEFAULT_SHELL_AUTOCOMPLETE_KEY,
        }
    }
    // based on the keybindings configuration provided in the configuration file
    // (if any), extract the character triggers command history management
    // through tv
    pub fn get_command_history_keybinding_character(&self) -> char {
        match self.keybindings.get(COMMAND_HISTORY_CONFIGURATION_KEY) {
            Some(&key) => {
                extract_ctrl_char(key).unwrap_or(DEFAULT_COMMAND_HISTORY_KEY)
            }
            None => DEFAULT_COMMAND_HISTORY_KEY,
        }
    }
}

/// Extract an upper-case character from a `Key` if it is a single CTRL key
/// (or CTRL-Space).  Returns `None` otherwise.
fn extract_ctrl_char(key: Key) -> Option<char> {
    match key {
        Key::Ctrl(c) => Some(c.to_ascii_uppercase()),
        Key::CtrlSpace => Some(' '),
        _ => None,
    }
}
