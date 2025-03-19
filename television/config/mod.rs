#![allow(clippy::module_name_repetitions, clippy::ref_option)]
use std::{
    env,
    hash::Hash,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use directories::ProjectDirs;
pub use keybindings::merge_keybindings;
pub use keybindings::{parse_key, Binding, KeyBindings};
use previewers::PreviewersConfig;
use serde::Deserialize;
use shell_integration::ShellIntegrationConfig;
pub use themes::Theme;
use tracing::{debug, warn};
pub use ui::UiConfig;

mod keybindings;
mod previewers;
pub mod shell_integration;
mod themes;
mod ui;

const DEFAULT_CONFIG: &str = include_str!("../../.config/config.toml");

#[allow(dead_code, clippy::module_name_repetitions)]
#[derive(Clone, Debug, Deserialize, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    #[serde(default = "get_data_dir")]
    pub data_dir: PathBuf,
    #[serde(default = "get_config_dir")]
    pub config_dir: PathBuf,
    #[serde(default = "default_frame_rate")]
    pub frame_rate: f64,
    #[serde(default = "default_tick_rate")]
    pub tick_rate: f64,
}

impl Hash for AppConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data_dir.hash(state);
        self.config_dir.hash(state);
        self.frame_rate.to_bits().hash(state);
        self.tick_rate.to_bits().hash(state);
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Default, PartialEq, Hash)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// General application configuration
    #[allow(clippy::struct_field_names)]
    #[serde(default, flatten)]
    pub application: AppConfig,
    /// Keybindings configuration
    #[serde(default)]
    pub keybindings: KeyBindings,
    /// UI configuration
    #[serde(default)]
    pub ui: UiConfig,
    /// Previewers configuration
    #[serde(default)]
    pub previewers: PreviewersConfig,
    /// Shell integration configuration
    #[serde(default)]
    pub shell_integration: ShellIntegrationConfig,
}

const PROJECT_NAME: &str = "television";
const CONFIG_FILE_NAME: &str = "config.toml";

pub struct ConfigEnv {
    _data_dir: PathBuf,
    config_dir: PathBuf,
}

impl ConfigEnv {
    pub fn init() -> Result<Self> {
        let data_dir = get_data_dir();
        let config_dir = get_config_dir();

        std::fs::create_dir_all(&config_dir)
            .context("Failed creating configuration directory")?;
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
    pub fn new(config_env: &ConfigEnv) -> Result<Self> {
        // Load the default_config values as base defaults
        let default_config: Config = default_config_from_file()?;

        // if a config file exists, load it and merge it with the default configuration
        if config_env.config_dir.join(CONFIG_FILE_NAME).is_file() {
            debug!("Found config file at {:?}", config_env.config_dir);

            let user_cfg: Config =
                Self::load_user_config(&config_env.config_dir)?;

            // merge the user configuration with the default configuration
            let final_cfg =
                Self::merge_user_with_default(default_config, user_cfg);

            debug!("Config: {:?}", final_cfg);
            Ok(final_cfg)
        } else {
            // otherwise, create the default configuration file
            warn!("No config file found at {:?}, creating default configuration file at that location.", config_env.config_dir);
            // create the default configuration file in the user's config directory
            std::fs::write(
                config_env.config_dir.join(CONFIG_FILE_NAME),
                DEFAULT_CONFIG,
            )?;
            Ok(default_config)
        }
    }

    fn load_user_config(config_dir: &Path) -> Result<Self> {
        let path = config_dir.join(CONFIG_FILE_NAME);
        let contents = std::fs::read_to_string(&path)?;
        let user_cfg: Config = toml::from_str(&contents).context(format!(
            "Error parsing configuration file: {}\n{}",
            path.display(),
            USER_CONFIG_ERROR_MSG,
        ))?;
        Ok(user_cfg)
    }

    fn merge_user_with_default(
        mut default: Config,
        mut user: Config,
    ) -> Config {
        // merge shell integration triggers with commands
        default.shell_integration.merge_triggers();
        user.shell_integration.merge_triggers();
        // merge shell integration commands with default commands
        let mut merged_commands = default.shell_integration.commands.clone();
        merged_commands.extend(user.shell_integration.commands.clone());
        user.shell_integration.commands = merged_commands;
        // merge shell integration keybindings with default keybindings
        let mut merged_keybindings =
            default.shell_integration.keybindings.clone();
        merged_keybindings.extend(user.shell_integration.keybindings.clone());
        user.shell_integration.keybindings = merged_keybindings;

        // merge keybindings with default keybindings
        let keybindings =
            merge_keybindings(default.keybindings.clone(), &user.keybindings);
        user.keybindings = keybindings;

        Config {
            application: user.application,
            keybindings: user.keybindings,
            ui: user.ui,
            previewers: user.previewers,
            shell_integration: user.shell_integration,
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

    let directory = if let Some(s) = data_folder {
        debug!("Using data directory: {:?}", s);
        s
    } else if let Some(proj_dirs) = project_directory() {
        debug!("Falling back to default data dir");
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        PathBuf::from("../../../../..").join(".data")
    };
    directory
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
    let directory = if let Some(s) = config_dir {
        debug!("Config directory: {:?}", s);
        s
    } else if cfg!(unix) {
        // default to ~/.config/television for unix systems
        if let Some(base_dirs) = directories::BaseDirs::new() {
            let cfg_dir =
                base_dirs.home_dir().join(".config").join("television");
            debug!("Config directory: {:?}", cfg_dir);
            cfg_dir
        } else {
            PathBuf::from("../../../../..").join(".config")
        }
    } else if let Some(proj_dirs) = project_directory() {
        debug!("Falling back to default config dir");
        proj_dirs.config_local_dir().to_path_buf()
    } else {
        PathBuf::from("../../../../..").join("../../../../../.config")
    };
    directory
}

fn project_directory() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "", env!("CARGO_PKG_NAME"))
}

fn default_frame_rate() -> f64 {
    60.0
}

fn default_tick_rate() -> f64 {
    50.0
}

#[cfg(test)]
mod tests {
    use crate::action::Action;
    use crate::event::Key;

    use super::*;
    use rustc_hash::FxHashMap;
    use std::fs::File;
    use std::io::Write;
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

        let config = Config::load_user_config(config_dir).unwrap();
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
        let config = Config::new(&config_env).unwrap();
        let mut default_config: Config =
            toml::from_str(DEFAULT_CONFIG).unwrap();
        default_config.shell_integration.merge_triggers();

        assert_eq!(config.application, default_config.application);
        assert_eq!(config.keybindings, default_config.keybindings);
        assert_eq!(config.ui, default_config.ui);
        assert_eq!(config.previewers, default_config.previewers);
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
        frame_rate = 30.0
        
        [ui]
        ui_scale = 40
        theme = "television"

        [previewers.file]
        theme = "Visual Studio Dark"

        [keybindings]
        toggle_help = ["ctrl-a", "ctrl-b"]
        confirm_selection = "ctrl-enter"

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
        let config = Config::new(&config_env).unwrap();

        let mut default_config: Config =
            toml::from_str(DEFAULT_CONFIG).unwrap();
        default_config.application.frame_rate = 30.0;
        default_config.ui.ui_scale = 40;
        default_config.ui.theme = "television".to_string();
        default_config.previewers.file.theme =
            "Visual Studio Dark".to_string();
        default_config.keybindings.extend({
            let mut map = FxHashMap::default();
            map.insert(
                Action::ToggleHelp,
                Binding::MultipleKeys(vec![Key::Ctrl('a'), Key::Ctrl('b')]),
            );
            map.insert(
                Action::ConfirmSelection,
                Binding::SingleKey(Key::CtrlEnter),
            );
            map
        });

        default_config
            .shell_integration
            .commands
            .extend(vec![("git add".to_string(), "git-diff".to_string())]);
        default_config
            .shell_integration
            .keybindings
            .insert("command_history".to_string(), "ctrl-h".to_string());
        default_config.shell_integration.merge_triggers();

        assert_eq!(config.application, default_config.application);
        assert_eq!(config.keybindings, default_config.keybindings);
        assert_eq!(config.ui, default_config.ui);
        assert_eq!(config.previewers, default_config.previewers);
        assert_eq!(
            config.shell_integration.commands,
            default_config.shell_integration.commands
        );
        assert_eq!(
            config.shell_integration.keybindings,
            default_config.shell_integration.keybindings
        );
    }
}
