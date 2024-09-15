use anyhow::{anyhow, Result};
use std::{env, path::PathBuf, sync::OnceLock};

use directories::ProjectDirs;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{de::Deserializer, ser::Serializer, Deserialize, Serialize};
use serde_with::{serde_as, NoneAsEmptyString};
use std::collections::HashMap;
use tracing::level_filters::LevelFilter;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use derive_deref::{Deref, DerefMut};
use itertools::Itertools;

use crate::{action::Action, app::Mode, command::Command};

use crate::cli::Cli;

static CONFIG: OnceLock<Config> = OnceLock::new();
pub const CONFIG_DEFAULT: &str = include_str!("../.config/config.default.toml");

/// Application configuration.
///
/// This is the main configuration struct for the application.
#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// The directory to use for storing application data (logs etc.).
    pub data_dir: PathBuf,

    /// The directory to use for storing application configuration (colors
    /// etc.).
    pub config_home: PathBuf,

    /// The directory to use for storing application configuration (colors
    /// etc.).
    pub config_file: PathBuf,

    /// The log level to use. Valid values are: error, warn, info, debug, trace,
    /// off. The default is info.
    #[serde_as(as = "NoneAsEmptyString")]
    pub log_level: Option<LevelFilter>,

    pub tick_rate: f64,

    pub frame_rate: f64,

    pub key_refresh_rate: f64,

    pub enable_mouse: bool,

    pub enable_paste: bool,

    pub prompt_padding: u16,

    pub key_bindings: KeyBindings,
    //pub color: Base16Palette,
}

impl Default for Config {
    fn default() -> Self {
        let key_bindings: KeyBindings = Default::default();
        //let rose_pine = Base16Palette::default();

        Self {
            data_dir: default_data_dir(),
            config_home: default_config_dir(),
            config_file: default_config_file(),
            log_level: None,
            tick_rate: 1.0,
            frame_rate: 15.0,
            key_refresh_rate: 0.5,
            enable_mouse: false,
            enable_paste: false,
            prompt_padding: 1,
            key_bindings,
            //color: rose_pine,
        }
    }
}

/// Initialize the application configuration.
///
/// This function should be called before any other function in the application.
/// It will initialize the application config from the following sources:
/// - default values
/// - a configuration file
/// - environment variables
/// - command line arguments
pub fn init(cli: &Cli) -> Result<()> {
    let config_file = cli.config_file.clone().unwrap_or_else(default_config_file);
    let color_file = cli.color_file.clone().unwrap_or_else(default_color_file);
    let mut config = Figment::new()
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::string(CONFIG_DEFAULT))
        .merge(Toml::file(config_file))
        .merge(Env::prefixed("CRATES_TUI_"))
        .merge(Serialized::defaults(cli))
        .extract::<Config>()?;
    //let base16 = Figment::new()
    //    .merge(Serialized::defaults(Base16Palette::default()))
    //    .merge(Yaml::file(color_file))
    //    .extract::<Base16Palette>()?;
    //config.color = base16;
    if let Some(data_dir) = cli.data_dir.clone() {
        config.data_dir = data_dir;
    }
    CONFIG
        .set(config)
        .map_err(|config| anyhow!("failed to set config {config:?}"))
}

/// Get the application configuration.
///
/// This function should only be called after [`init()`] has been called.
///
/// # Panics
///
/// This function will panic if [`init()`] has not been called.
pub fn get() -> &'static Config {
    CONFIG.get().expect("config not initialized")
}

/// Returns the path to the default configuration file.
pub fn default_config_file() -> PathBuf {
    default_config_dir().join("config.toml")
}

/// Returns the path to the default configuration file.
pub fn default_color_file() -> PathBuf {
    default_config_dir().join("color.yaml")
}

/// Returns the directory to use for storing config files.
fn default_config_dir() -> PathBuf {
    env::var("CRATES_TUI_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|_| project_dirs().map(|dirs| dirs.config_local_dir().to_path_buf()))
        .unwrap_or(PathBuf::from(".").join(".config"))
}

/// Returns the directory to use for storing data files.
pub fn default_data_dir() -> PathBuf {
    env::var("CRATES_TUI_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|_| project_dirs().map(|dirs| dirs.data_local_dir().to_path_buf()))
        .unwrap_or(PathBuf::from(".").join(".data"))
}

/// Returns the project directories.
fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("rs", "ratatui", "crates-tui")
        .ok_or_else(|| anyhow!("project directories not found"))
}

#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct KeyBindings(pub HashMap<Mode, HashMap<Vec<KeyEvent>, Command>>);

impl KeyBindings {
    pub fn command_to_action(&self, command: Command) -> Action {
        match command {
            Command::Quit => Action::Quit,
            Command::NextTab => Action::NextTab,
            Command::PreviousTab => Action::PreviousTab,
            Command::ClosePopup => Action::ClosePopup,
            Command::SwitchMode(m) => Action::SwitchMode(m),
            Command::SwitchToLastMode => Action::SwitchToLastMode,
            Command::IncrementPage => Action::IncrementPage,
            Command::DecrementPage => Action::DecrementPage,
            Command::NextSummaryMode => Action::NextSummaryMode,
            Command::PreviousSummaryMode => Action::PreviousSummaryMode,
            Command::ToggleSortBy { reload, forward } => Action::ToggleSortBy { reload, forward },
            Command::ScrollBottom => Action::ScrollBottom,
            Command::ScrollTop => Action::ScrollTop,
            Command::ScrollDown => Action::ScrollDown,
            Command::ScrollUp => Action::ScrollUp,
            Command::ScrollCrateInfoDown => Action::ScrollCrateInfoDown,
            Command::ScrollCrateInfoUp => Action::ScrollCrateInfoUp,
            Command::ScrollSearchResultsDown => Action::ScrollSearchResultsDown,
            Command::ScrollSearchResultsUp => Action::ScrollSearchResultsUp,
            Command::SubmitSearch => Action::SubmitSearch,
            Command::ReloadData => Action::ReloadData,
            Command::ToggleShowCrateInfo => Action::ToggleShowCrateInfo,
            Command::CopyCargoAddCommandToClipboard => Action::CopyCargoAddCommandToClipboard,
            Command::OpenDocsUrlInBrowser => Action::OpenDocsUrlInBrowser,
            Command::OpenCratesIOUrlInBrowser => Action::OpenCratesIOUrlInBrowser,
        }
    }

    #[allow(dead_code)]
    pub fn insert(&mut self, mode: Mode, key_events: &[KeyEvent], command: Command) {
        // Convert the slice of `KeyEvent`(s) to a `Vec`.
        let key_events_vec = key_events.to_vec();

        // Retrieve or create the inner `HashMap` corresponding to the mode.
        let bindings_for_mode = self.0.entry(mode).or_default();

        // Insert the `Command` into the inner `HashMap` using the key events `Vec` as
        // the key.
        bindings_for_mode.insert(key_events_vec, command);
    }

    pub fn event_to_command(&self, mode: Mode, key_events: &[KeyEvent]) -> Option<Command> {
        if key_events.is_empty() {
            None
        } else if let Some(Some(command)) = self.0.get(&mode).map(|kb| kb.get(key_events)) {
            Some(*command)
        } else {
            self.event_to_command(mode, &key_events[1..])
        }
    }

    pub fn get_keybindings_for_command(&self, mode: Mode, command: Command) -> Vec<Vec<KeyEvent>> {
        let bindings_for_mode = self.0.get(&mode).cloned().unwrap_or_default();
        bindings_for_mode
            .into_iter()
            .filter(|(_, v)| *v == command)
            .map(|(k, _)| k)
            .collect_vec()
    }

    pub fn get_config_for_command(&self, mode: Mode, command: Command) -> Vec<String> {
        self.get_keybindings_for_command(mode, command)
            .iter()
            .map(|key_events| {
                key_events
                    .iter()
                    .map(key_event_to_string)
                    .collect_vec()
                    .join("")
            })
            .collect_vec()
    }
}

impl<'de> Deserialize<'de> for KeyBindings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let parsed_map = HashMap::<Mode, HashMap<String, Command>>::deserialize(deserializer)?;

        let keybindings = parsed_map
            .into_iter()
            .map(|(mode, inner_map)| {
                let converted_inner_map = inner_map
                    .into_iter()
                    .map(|(key_str, cmd)| (parse_key_sequence(&key_str).unwrap(), cmd))
                    .collect();
                (mode, converted_inner_map)
            })
            .collect();

        Ok(KeyBindings(keybindings))
    }
}

impl Serialize for KeyBindings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serialized_map: HashMap<Mode, HashMap<String, Command>> = HashMap::new();

        for (mode, key_event_map) in self.0.iter() {
            let mut string_event_map = HashMap::new();

            for (key_events, command) in key_event_map {
                let key_string = key_events
                    .iter()
                    .map(|key_event| format!("<{}>", key_event_to_string(key_event)))
                    .collect::<Vec<String>>()
                    .join("");

                string_event_map.insert(key_string, *command);
            }

            serialized_map.insert(*mode, string_event_map);
        }

        serialized_map.serialize(serializer)
    }
}

fn parse_key_event(raw: &str) -> Result<KeyEvent, String> {
    let (remaining, modifiers) = extract_modifiers(raw);
    parse_key_code_with_modifiers(remaining, modifiers)
}

fn extract_modifiers(raw: &str) -> (&str, KeyModifiers) {
    let mut modifiers = KeyModifiers::empty();
    let mut current = raw;

    loop {
        match current {
            rest if rest.to_lowercase().starts_with("ctrl-") => {
                modifiers.insert(KeyModifiers::CONTROL);
                current = &rest[5..];
            }
            rest if rest.to_lowercase().starts_with("alt-") => {
                modifiers.insert(KeyModifiers::ALT);
                current = &rest[4..];
            }
            rest if rest.to_lowercase().starts_with("shift-") => {
                modifiers.insert(KeyModifiers::SHIFT);
                current = &rest[6..];
            }
            _ => break, // break out of the loop if no known prefix is detected
        };
    }

    (current, modifiers)
}

// FIXME - seems excessively verbose. Use strum to simplify?
fn parse_key_code_with_modifiers(
    raw: &str,
    mut modifiers: KeyModifiers,
) -> Result<KeyEvent, String> {
    let c = match raw.to_lowercase().as_str() {
        "esc" => KeyCode::Esc,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "backtab" => {
            modifiers.insert(KeyModifiers::SHIFT);
            KeyCode::BackTab
        }
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        "space" => KeyCode::Char(' '),
        "hyphen" => KeyCode::Char('-'),
        "minus" => KeyCode::Char('-'),
        "tab" => KeyCode::Tab,
        c if c.len() == 1 => {
            let mut c = raw.chars().next().unwrap();
            if modifiers.contains(KeyModifiers::SHIFT) {
                c = c.to_ascii_uppercase();
            }
            KeyCode::Char(c)
        }
        _ => return Err(format!("Unable to parse {raw}")),
    };
    Ok(KeyEvent::new(c, modifiers))
}

pub fn key_event_to_string(key_event: &KeyEvent) -> String {
    let char;
    let key_code = match key_event.code {
        KeyCode::Backspace => "Backspace",
        KeyCode::Enter => "Enter",
        KeyCode::Left => "Left",
        KeyCode::Right => "Right",
        KeyCode::Up => "Up",
        KeyCode::Down => "Down",
        KeyCode::Home => "Home",
        KeyCode::End => "End",
        KeyCode::PageUp => "PageUp",
        KeyCode::PageDown => "PageDown",
        KeyCode::Tab => "Tab",
        KeyCode::BackTab => "Backtab",
        KeyCode::Delete => "Delete",
        KeyCode::Insert => "Insert",
        KeyCode::F(c) => {
            char = format!("F({c})");
            &char
        }
        KeyCode::Char(' ') => "Space",
        KeyCode::Char(c) => {
            char = c.to_string();
            &char
        }
        KeyCode::Esc => "Esc",
        KeyCode::Null => "",
        KeyCode::CapsLock => "",
        KeyCode::Menu => "",
        KeyCode::ScrollLock => "",
        KeyCode::Media(_) => "",
        KeyCode::NumLock => "",
        KeyCode::PrintScreen => "",
        KeyCode::Pause => "",
        KeyCode::KeypadBegin => "",
        KeyCode::Modifier(_) => "",
    };

    let mut modifiers = Vec::with_capacity(3);

    if key_event.modifiers.intersects(KeyModifiers::CONTROL) {
        modifiers.push("Ctrl");
    }

    if key_event.modifiers.intersects(KeyModifiers::SHIFT) {
        modifiers.push("Shift");
    }

    if key_event.modifiers.intersects(KeyModifiers::ALT) {
        modifiers.push("Alt");
    }

    let mut key = modifiers.join("-");

    if !key.is_empty() {
        key.push('-');
    }
    key.push_str(key_code);

    key
}

pub fn parse_key_sequence(raw: &str) -> Result<Vec<KeyEvent>, String> {
    if raw.chars().filter(|c| *c == '>').count() != raw.chars().filter(|c| *c == '<').count() {
        return Err(format!("Unable to parse `{}`", raw));
    }
    let raw = if !raw.contains("><") {
        let raw = raw.strip_prefix('<').unwrap_or(raw);
        let raw = raw.strip_prefix('>').unwrap_or(raw);
        raw
    } else {
        raw
    };
    let sequences = raw
        .split("><")
        .map(|seq| {
            if let Some(s) = seq.strip_prefix('<') {
                s
            } else if let Some(s) = seq.strip_suffix('>') {
                s
            } else {
                seq
            }
        })
        .collect::<Vec<_>>();

    sequences.into_iter().map(parse_key_event).collect()
}
