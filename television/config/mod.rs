use crate::{
    action::Action,
    cable::CABLE_DIR_NAME,
    channels::prototypes::{DEFAULT_PROTOTYPE_NAME, Template, UiSpec},
    cli::PostProcessedCli,
    features::FeatureFlags,
    history::DEFAULT_HISTORY_SIZE,
    screen::keybindings::remove_action_bindings,
};
use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use shell_integration::ShellIntegrationConfig;
use std::{
    env,
    hash::Hash,
    path::{Path, PathBuf},
};
use tracing::{debug, warn};

pub use keybindings::{EventBindings, EventType, KeyBindings, merge_bindings};
pub use themes::Theme;
pub use ui::UiConfig;

mod themes;

pub mod keybindings;
pub mod shell_integration;
pub mod ui;

const DEFAULT_CONFIG: &str = include_str!("../../.config/config.toml");

#[allow(dead_code, clippy::module_name_repetitions)]
#[derive(Clone, Debug, Deserialize, Default, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    #[serde(default = "get_data_dir")]
    pub data_dir: PathBuf,
    #[serde(default = "get_config_dir")]
    pub config_dir: PathBuf,
    #[serde(default = "default_cable_dir")]
    pub cable_dir: PathBuf,
    #[serde(default = "default_tick_rate")]
    pub tick_rate: f64,
    /// The default channel to use when no channel is specified
    #[serde(default = "default_channel")]
    pub default_channel: String,
    /// Maximum number of entries to keep in the global history
    #[serde(default = "default_history_size")]
    pub history_size: usize,
    /// Whether to use global history (all channels) or channel-specific history (default)
    #[serde(default = "default_global_history")]
    pub global_history: bool,
}

fn default_channel() -> String {
    DEFAULT_PROTOTYPE_NAME.to_string()
}

fn default_history_size() -> usize {
    DEFAULT_HISTORY_SIZE
}

fn default_global_history() -> bool {
    false
}

impl Hash for AppConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data_dir.hash(state);
        self.config_dir.hash(state);
        self.tick_rate.to_bits().hash(state);
        self.history_size.hash(state);
        self.global_history.hash(state);
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Hash)]
pub struct Config {
    /// General application configuration
    #[allow(clippy::struct_field_names)]
    #[serde(default, flatten)]
    pub application: AppConfig,
    /// Keybindings configuration
    #[serde(default)]
    pub keybindings: KeyBindings,
    /// Event bindings configuration
    #[serde(default)]
    pub events: EventBindings,
    /// UI configuration
    #[serde(default)]
    pub ui: UiConfig,
    /// Shell integration configuration
    #[serde(default)]
    pub shell_integration: ShellIntegrationConfig,
}

const PROJECT_NAME: &str = "television";
pub const CONFIG_FILE_NAME: &str = "config.toml";

pub struct ConfigEnv {
    _data_dir: PathBuf,
    config_dir: PathBuf,
}

impl ConfigEnv {
    pub fn init() -> Result<Self> {
        let data_dir = get_data_dir();
        let config_dir = get_config_dir();
        let cable_dir = config_dir.join(CABLE_DIR_NAME);

        std::fs::create_dir_all(&config_dir)
            .context("Failed creating configuration directory")?;
        std::fs::create_dir_all(&cable_dir)
            .context("Failed creating cable directory")?;
        std::fs::create_dir_all(&data_dir)
            .context("Failed creating data directory")?;

        Ok(Self {
            _data_dir: data_dir,
            config_dir,
        })
    }
}

pub fn default_config_from_file() -> Result<Config> {
    let default_config: Config = toml::from_str(DEFAULT_CONFIG)
        .context("Error parsing default config")?;
    Ok(default_config)
}

const USER_CONFIG_ERROR_MSG: &str = "
╔══════════════════════════════════════════════════════════════════════════════╗
║                                                                              ║
║  If this follows a recent update, it is likely due to a breaking change in   ║
║  the configuration format.                                                   ║
║                                                                              ║
║  Check https://github.com/alexpasmantier/television/releases/latest for the  ║
║  latest release notes.                                                       ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝";

impl Config {
    #[allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]
    pub fn new(
        config_env: &ConfigEnv,
        custom_config_file: Option<&Path>,
    ) -> Result<Self> {
        // Load the default_config values as base defaults
        let default_config: Config = default_config_from_file()?;

        // if a config file exists, load it and merge it with the default configuration
        if config_env.config_dir.join(CONFIG_FILE_NAME).is_file()
            || custom_config_file.is_some()
        {
            let config_file = if let Some(path) = custom_config_file {
                debug!("Using custom configuration file at: {:?}", path);
                path.to_path_buf()
            } else {
                let config_file = config_env.config_dir.join(CONFIG_FILE_NAME);
                debug!(
                    "Using default configuration file at: {:?}",
                    config_file
                );
                config_file
            };

            let user_cfg: Config = Self::load_user_config(&config_file)?;

            // merge the user configuration with the default configuration
            let final_cfg = Self::merge_with_default(default_config, user_cfg);

            debug!(
                "Configuration: \n{}",
                toml::to_string(&final_cfg).unwrap()
            );
            Ok(final_cfg)
        } else {
            // otherwise, create the default configuration file
            warn!(
                "No config file found at {:?}, creating default configuration file at that location.",
                config_env.config_dir
            );
            // create the default configuration file in the user's config directory
            std::fs::write(
                config_env.config_dir.join(CONFIG_FILE_NAME),
                DEFAULT_CONFIG,
            )?;
            Ok(default_config)
        }
    }

    fn load_user_config(config_file: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(config_file)?;
        let user_cfg: Config = toml::from_str(&contents).context(format!(
            "Error parsing configuration file: {}\n{}",
            config_file.display(),
            USER_CONFIG_ERROR_MSG,
        ))?;
        Ok(user_cfg)
    }

    fn merge_with_default(mut default: Config, mut new: Config) -> Config {
        // use default fallback channel as a fallback if user hasn't specified one
        if new.shell_integration.fallback_channel.is_empty() {
            new.shell_integration
                .fallback_channel
                .clone_from(&default.shell_integration.fallback_channel);
        }

        // merge shell integration triggers with commands
        default.shell_integration.merge_triggers();
        new.shell_integration.merge_triggers();
        // merge shell integration commands with default commands
        if new.shell_integration.commands.is_empty() {
            new.shell_integration
                .commands
                .clone_from(&default.shell_integration.commands);
        }

        // merge shell integration keybindings with default keybindings
        let mut merged_keybindings =
            default.shell_integration.keybindings.clone();
        merged_keybindings.extend(new.shell_integration.keybindings.clone());
        new.shell_integration.keybindings = merged_keybindings;

        // merge keybindings with default keybindings
        let keybindings =
            merge_bindings(default.keybindings.clone(), &new.keybindings);
        new.keybindings = keybindings;

        // merge event bindings with default event bindings
        let events = merge_bindings(default.events.clone(), &new.events);
        new.events = events;

        Config {
            application: new.application,
            keybindings: new.keybindings,
            events: new.events,
            ui: new.ui,
            shell_integration: new.shell_integration,
        }
    }

    pub fn merge_channel_keybindings(&mut self, other: &KeyBindings) {
        self.keybindings = merge_bindings(self.keybindings.clone(), other);
    }

    /// Apply CLI keybinding overrides.
    pub fn apply_cli_keybinding_overrides(
        &mut self,
        cli_keybindings: &KeyBindings,
    ) {
        debug!("keybindings before: {:?}", self.keybindings);

        for (key, actions) in &cli_keybindings.bindings {
            // Update the keybinding
            self.keybindings.insert(*key, actions.clone());
        }

        debug!("keybindings after: {:?}", self.keybindings);
    }

    pub fn merge_event_bindings(&mut self, other: &EventBindings) {
        self.events = merge_bindings(self.events.clone(), other);
    }

    /// Apply CLI overrides to this config
    pub fn apply_cli_overrides(&mut self, args: &PostProcessedCli) {
        debug!("Applying CLI overrides to config after channel merging");

        if let Some(cable_dir) = &args.cable_dir {
            self.application.cable_dir.clone_from(cable_dir);
        }
        if let Some(tick_rate) = args.tick_rate {
            self.application.tick_rate = tick_rate;
        }
        if args.global_history {
            self.application.global_history = true;
        }
        // Handle preview panel flags
        if args.no_preview {
            self.ui.features.disable(FeatureFlags::PreviewPanel);
            remove_action_bindings(
                &mut self.keybindings,
                &Action::TogglePreview.into(),
            );
        } else if args.hide_preview {
            self.ui.features.hide(FeatureFlags::PreviewPanel);
        } else if args.show_preview {
            self.ui.features.enable(FeatureFlags::PreviewPanel);
        }

        if let Some(ps) = args.preview_size {
            self.ui.preview_panel.size = ps;
        }

        // Handle status bar flags
        if args.no_status_bar {
            self.ui.features.disable(FeatureFlags::StatusBar);
            remove_action_bindings(
                &mut self.keybindings,
                &Action::ToggleStatusBar.into(),
            );
        } else if args.hide_status_bar {
            self.ui.features.hide(FeatureFlags::StatusBar);
        } else if args.show_status_bar {
            self.ui.features.enable(FeatureFlags::StatusBar);
        }

        // Handle remote control flags
        if args.no_remote {
            self.ui.features.disable(FeatureFlags::RemoteControl);
            remove_action_bindings(
                &mut self.keybindings,
                &Action::ToggleRemoteControl.into(),
            );
        } else if args.hide_remote {
            self.ui.features.hide(FeatureFlags::RemoteControl);
        } else if args.show_remote {
            self.ui.features.enable(FeatureFlags::RemoteControl);
        }

        // Handle help panel flags
        if args.no_help_panel {
            self.ui.features.disable(FeatureFlags::HelpPanel);
            remove_action_bindings(
                &mut self.keybindings,
                &Action::ToggleHelp.into(),
            );
        } else if args.hide_help_panel {
            self.ui.features.hide(FeatureFlags::HelpPanel);
        } else if args.show_help_panel {
            self.ui.features.enable(FeatureFlags::HelpPanel);
        }

        // Apply CLI keybinding overrides
        if let Some(keybindings) = &args.keybindings {
            self.apply_cli_keybinding_overrides(keybindings);
        }

        self.ui.ui_scale = args.ui_scale.unwrap_or(self.ui.ui_scale);
        if let Some(input_header) = &args.input_header {
            if let Ok(t) = Template::parse(input_header) {
                self.ui.input_bar.header = Some(t);
            }
        }
        if let Some(input_prompt) = &args.input_prompt {
            self.ui.input_bar.prompt.clone_from(input_prompt);
        }
        if let Some(preview_header) = &args.preview_header {
            if let Ok(t) = Template::parse(preview_header) {
                self.ui.preview_panel.header = Some(t);
            }
        }
        if let Some(preview_footer) = &args.preview_footer {
            if let Ok(t) =
                crate::channels::prototypes::Template::parse(preview_footer)
            {
                self.ui.preview_panel.footer = Some(t);
            }
        }
        if let Some(layout) = args.layout {
            self.ui.orientation = layout;
        }
        if let Some(input_border) = args.input_border {
            self.ui.input_bar.border_type = input_border;
        }
        if let Some(preview_border) = args.preview_border {
            self.ui.preview_panel.border_type = preview_border;
        }
        if let Some(results_border) = args.results_border {
            self.ui.results_panel.border_type = results_border;
        }
        if let Some(input_padding) = args.input_padding {
            self.ui.input_bar.padding = input_padding;
        }
        if let Some(preview_padding) = args.preview_padding {
            self.ui.preview_panel.padding = preview_padding;
        }
        if let Some(results_padding) = args.results_padding {
            self.ui.results_panel.padding = results_padding;
        }
    }

    pub fn apply_prototype_ui_spec(&mut self, ui_spec: &UiSpec) {
        // Apply simple copy fields (Copy types)
        if let Some(value) = ui_spec.ui_scale {
            self.ui.ui_scale = value;
        }
        if let Some(value) = ui_spec.orientation {
            self.ui.orientation = value;
        }

        // Apply clone fields
        if let Some(value) = &ui_spec.features {
            self.ui.features = value.clone();
        }
        if let Some(value) = &ui_spec.status_bar {
            self.ui.status_bar = value.clone();
        }
        if let Some(value) = &ui_spec.help_panel {
            self.ui.help_panel = value.clone();
        }
        if let Some(value) = &ui_spec.remote_control {
            self.ui.remote_control = value.clone();
        }

        // Handle input, results, and preview with field merging
        if let Some(input_bar) = &ui_spec.input_bar {
            self.ui.input_bar.position = input_bar.position;
            if input_bar.header.is_some() {
                self.ui.input_bar.header.clone_from(&input_bar.header);
            }
            self.ui.input_bar.prompt.clone_from(&input_bar.prompt);
            self.ui
                .input_bar
                .border_type
                .clone_from(&input_bar.border_type);
            self.ui.input_bar.padding = input_bar.padding;
        }
        if let Some(results_panel) = &ui_spec.results_panel {
            self.ui
                .results_panel
                .border_type
                .clone_from(&results_panel.border_type);
            self.ui.results_panel.padding = results_panel.padding;
        }
        if let Some(preview_panel) = &ui_spec.preview_panel {
            self.ui.preview_panel.size = preview_panel.size;
            if let Some(header) = &preview_panel.header {
                self.ui.preview_panel.header = Some(header.clone());
            }
            if let Some(footer) = &preview_panel.footer {
                self.ui.preview_panel.footer = Some(footer.clone());
            }
            self.ui.preview_panel.scrollbar = preview_panel.scrollbar;
            self.ui
                .preview_panel
                .border_type
                .clone_from(&preview_panel.border_type);
            self.ui.preview_panel.padding = preview_panel.padding;
        }
    }
}

pub fn get_data_dir() -> PathBuf {
    // if `TELEVISION_DATA` is set, use that as the data directory
    let data_folder =
        env::var_os(format!("{}_DATA", PROJECT_NAME.to_uppercase()))
            .map(PathBuf::from)
            .or_else(|| {
                // otherwise, use the XDG data directory
                env::var_os("XDG_DATA_HOME")
                    .map(PathBuf::from)
                    .map(|p| p.join(PROJECT_NAME))
                    .filter(|p| p.is_absolute())
            });

    if let Some(s) = data_folder {
        s
    } else if let Some(proj_dirs) = project_directory() {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        PathBuf::from("../../../../..").join(".data")
    }
}

pub fn get_config_dir() -> PathBuf {
    // if `TELEVISION_CONFIG` is set, use that as the television config directory
    let config_dir =
        env::var_os(format!("{}_CONFIG", PROJECT_NAME.to_uppercase()))
            .map(PathBuf::from)
            .or_else(|| {
                // otherwise, use the XDG config directory + 'television'
                env::var_os("XDG_CONFIG_HOME")
                    .map(PathBuf::from)
                    .map(|p| p.join(PROJECT_NAME))
                    .filter(|p| p.is_absolute())
            });
    if let Some(s) = config_dir {
        s
    } else if cfg!(unix) {
        // default to ~/.config/television for unix systems
        if let Some(base_dirs) = directories::BaseDirs::new() {
            base_dirs.home_dir().join(".config").join("television")
        } else {
            PathBuf::from("../../../../..").join(".config")
        }
    } else if let Some(proj_dirs) = project_directory() {
        proj_dirs.config_local_dir().to_path_buf()
    } else {
        PathBuf::from("../../../../..").join("../../../../../.config")
    }
}

fn default_cable_dir() -> PathBuf {
    get_config_dir().join(CABLE_DIR_NAME)
}

fn project_directory() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "", env!("CARGO_PKG_NAME"))
}

pub fn default_tick_rate() -> f64 {
    50.0
}

pub use ui::{DEFAULT_PREVIEW_SIZE, DEFAULT_UI_SCALE};

#[cfg(test)]
mod tests {
    use crate::action::Action;
    use crate::config::ui::Padding;
    use crate::event::Key;

    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::str::FromStr;
    use tempfile::tempdir;

    #[test]
    fn test_get_data_dir() {
        let data_dir = get_data_dir();
        assert!(data_dir.is_absolute());
    }

    #[test]
    fn test_get_config_dir() {
        let config_dir = get_config_dir();
        assert!(config_dir.is_absolute());
    }

    #[test]
    fn test_load_user_config() {
        let dir = tempdir().unwrap();
        let config_dir = dir.path();
        let config_file = config_dir.join(CONFIG_FILE_NAME);
        let mut file = File::create(&config_file).unwrap();
        file.write_all(DEFAULT_CONFIG.as_bytes()).unwrap();

        let config = Config::load_user_config(&config_file).unwrap();
        assert_eq!(config.application.data_dir, get_data_dir());
        assert_eq!(config.application.config_dir, get_config_dir());
        assert_eq!(config, toml::from_str(DEFAULT_CONFIG).unwrap());
    }

    #[test]
    fn test_config_new_empty_user_cfg() {
        // write user config to a file
        let dir = tempdir().unwrap();
        let config_dir = dir.path();
        let config_file = config_dir.join(CONFIG_FILE_NAME);
        let _ = File::create(&config_file).unwrap();

        let config_env = ConfigEnv {
            _data_dir: get_data_dir(),
            config_dir: config_dir.to_path_buf(),
        };
        let config = Config::new(&config_env, None).unwrap();
        let mut default_config: Config =
            toml::from_str(DEFAULT_CONFIG).unwrap();
        default_config.shell_integration.merge_triggers();

        assert_eq!(config.application, default_config.application);
        assert_eq!(config.keybindings, default_config.keybindings);
        assert_eq!(config.events, default_config.events);
        assert_eq!(config.ui, default_config.ui);
        // backwards compatibility
        assert_eq!(
            config.shell_integration.commands,
            default_config.shell_integration.commands
        );
        assert_eq!(
            config.shell_integration.keybindings,
            default_config.shell_integration.keybindings
        );
    }

    const USER_CONFIG_1: &str = r#"
        [ui]
        ui_scale = 40
        theme = "television"

        [previewers.file]
        theme = "something"

        [keybindings]
        ctrl-enter = "confirm_selection"

        [shell_integration.commands]
        "git add" = "git-diff"

        [shell_integration.keybindings]
        "command_history" = "ctrl-h"

    "#;

    #[test]
    fn test_config_new_with_user_cfg() {
        // write user config to a file
        let dir = tempdir().unwrap();
        let config_dir = dir.path();
        let config_file = config_dir.join(CONFIG_FILE_NAME);
        let mut file = File::create(&config_file).unwrap();
        file.write_all(USER_CONFIG_1.as_bytes()).unwrap();

        let config_env = ConfigEnv {
            _data_dir: get_data_dir(),
            config_dir: config_dir.to_path_buf(),
        };
        let config = Config::new(&config_env, None).unwrap();

        let mut default_config: Config =
            toml::from_str(DEFAULT_CONFIG).unwrap();
        default_config.ui.ui_scale = 40;
        default_config.ui.theme = "television".to_string();
        // With new architecture, we add directly to the bindings map
        default_config
            .keybindings
            .bindings
            .insert(Key::CtrlEnter, Action::ConfirmSelection.into());

        default_config.shell_integration.keybindings.insert(
            "command_history".to_string(),
            Key::from_str("ctrl-h").unwrap(),
        );
        default_config.shell_integration.merge_triggers();

        assert_eq!(config.application, default_config.application);
        assert_eq!(config.keybindings, default_config.keybindings);
        assert_eq!(config.events, default_config.events);
        assert_eq!(config.ui, default_config.ui);
        assert_eq!(
            config.shell_integration.commands,
            [(&String::from("git add"), &String::from("git-diff"))]
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect()
        );
        assert_eq!(
            config.shell_integration.keybindings,
            default_config.shell_integration.keybindings
        );
    }

    const USER_CONFIG_INPUT_PROMPT: &str = r#"
        [ui.input_bar]
        prompt = "❯"
    "#;

    #[test]
    fn test_config_input_prompt_from_user_cfg() {
        // write user config to a file
        let dir = tempdir().unwrap();
        let config_dir = dir.path();
        let config_file = config_dir.join(CONFIG_FILE_NAME);
        let mut file = File::create(&config_file).unwrap();
        file.write_all(USER_CONFIG_INPUT_PROMPT.as_bytes()).unwrap();

        let config_env = ConfigEnv {
            _data_dir: get_data_dir(),
            config_dir: config_dir.to_path_buf(),
        };
        let config = Config::new(&config_env, None).unwrap();

        // Verify that input_prompt was loaded from user config
        assert_eq!(config.ui.input_bar.prompt, "❯");
    }

    #[test]
    fn test_setting_user_shell_integration_triggers_overrides_default() {
        let user_config = r#"
            [shell_integration.channel_triggers]
            "files" = ["some command"]
        "#;

        let dir = tempdir().unwrap();
        let config_dir = dir.path();
        let config_file = config_dir.join(CONFIG_FILE_NAME);
        let mut file = File::create(&config_file).unwrap();
        file.write_all(user_config.as_bytes()).unwrap();

        let config_env = ConfigEnv {
            _data_dir: get_data_dir(),
            config_dir: config_dir.to_path_buf(),
        };

        let config = Config::new(&config_env, None).unwrap();

        assert_eq!(
            config.shell_integration.commands.iter().collect::<Vec<_>>(),
            vec![(&String::from("some command"), &String::from("files"))]
        );
    }

    #[test]
    fn test_shell_integration_keybindings_are_overwritten_by_user() {
        let user_config = r#"
            [shell_integration.keybindings]
            "smart_autocomplete" = "ctrl-t"
            "command_history" = "ctrl-["
        "#;

        let dir = tempdir().unwrap();
        let config_dir = dir.path();
        let config_file = config_dir.join(CONFIG_FILE_NAME);
        let mut file = File::create(&config_file).unwrap();
        file.write_all(user_config.as_bytes()).unwrap();

        let config_env = ConfigEnv {
            _data_dir: get_data_dir(),
            config_dir: config_dir.to_path_buf(),
        };

        let config = Config::new(&config_env, None).unwrap();

        let expected: rustc_hash::FxHashMap<String, Key> = [
            (
                "command_history".to_string(),
                Key::from_str("ctrl-[").unwrap(),
            ),
            (
                "smart_autocomplete".to_string(),
                Key::from_str("ctrl-t").unwrap(),
            ),
        ]
        .iter()
        .cloned()
        .collect();

        assert_eq!(config.shell_integration.keybindings, expected);
    }

    #[test]
    fn test_apply_prototype_ui_spec() {
        use crate::channels::prototypes::Template;
        use crate::features::Features;
        use crate::screen::layout::{InputPosition, Orientation};
        use ui::{
            BorderType, HelpPanelConfig, InputBarConfig, PreviewPanelConfig,
            RemoteControlConfig, ResultsPanelConfig, StatusBarConfig,
        };

        let mut features = Features::default();
        features.help_panel.disable();

        let mut config = Config::default();
        config.ui.preview_panel.header =
            Some(Template::Raw("cow".to_string()));

        let ui_spec = UiSpec {
            ui_scale: Some(12),
            features: Some(features),
            orientation: Some(Orientation::Portrait),
            input_bar: Some(InputBarConfig {
                position: InputPosition::Bottom,
                header: Some(Template::Raw("hello".to_string())),
                prompt: "world".to_string(),
                border_type: BorderType::Thick,
                padding: Padding::uniform(2),
            }),
            results_panel: Some(ResultsPanelConfig {
                border_type: BorderType::None,
                padding: Padding::uniform(2),
            }),
            preview_panel: Some(PreviewPanelConfig {
                size: 42,
                header: None, // does not overwrite "cow"
                footer: Some(Template::Raw("moo".to_string())),
                scrollbar: true,
                border_type: BorderType::Plain,
                padding: Padding::uniform(2),
            }),
            status_bar: Some(StatusBarConfig {
                separator_open: "open".to_string(),
                separator_close: "close".to_string(),
            }),
            help_panel: Some(HelpPanelConfig {
                show_categories: true,
            }),
            remote_control: Some(RemoteControlConfig {
                show_channel_descriptions: true,
                sort_alphabetically: true,
            }),
        };
        config.apply_prototype_ui_spec(&ui_spec);

        assert_eq!(config.ui.ui_scale, 12);
        assert!(!config.ui.features.help_panel.enabled);
        assert_eq!(config.ui.input_bar.position, InputPosition::Bottom);
        assert_eq!(
            config.ui.input_bar.header.as_ref().unwrap().raw(),
            "hello"
        );
        assert_eq!(config.ui.input_bar.prompt, "world");
        assert_eq!(config.ui.input_bar.border_type, BorderType::Thick);
        assert_eq!(config.ui.input_bar.padding, Padding::uniform(2));

        assert_eq!(config.ui.results_panel.border_type, BorderType::None);
        assert_eq!(config.ui.results_panel.padding, Padding::uniform(2));

        assert_eq!(config.ui.preview_panel.size, 42);
        assert_eq!(
            config.ui.preview_panel.header.as_ref().unwrap().raw(),
            "cow"
        );
        assert_eq!(
            config.ui.preview_panel.footer.as_ref().unwrap().raw(),
            "moo"
        );
        assert!(config.ui.preview_panel.scrollbar);
        assert_eq!(config.ui.preview_panel.border_type, BorderType::Plain);
        assert_eq!(config.ui.preview_panel.padding, Padding::uniform(2));

        assert_eq!(config.ui.status_bar.separator_open, "open");
        assert_eq!(config.ui.status_bar.separator_close, "close");
        assert!(config.ui.remote_control.show_channel_descriptions);
        assert!(config.ui.remote_control.sort_alphabetically);
    }
}
