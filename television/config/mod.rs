#![allow(clippy::module_name_repetitions)]
use std::{env, path::PathBuf};

use color_eyre::{eyre::Context, Result};
use directories::ProjectDirs;
use keybindings::merge_keybindings;
pub use keybindings::{parse_key, Binding, KeyBindings};
use lazy_static::lazy_static;
use previewers::PreviewersConfig;
use serde::Deserialize;
use shell_integration::ShellIntegrationConfig;
pub use themes::Theme;
use tracing::{debug, warn};
use ui::UiConfig;

mod keybindings;
mod previewers;
mod shell_integration;
mod themes;
mod ui;

const DEFAULT_CONFIG: &str = include_str!("../../.config/config.toml");

#[allow(dead_code, clippy::module_name_repetitions)]
#[derive(Clone, Debug, Deserialize, Default)]
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

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// General application configuration
    #[allow(clippy::struct_field_names)]
    #[serde(default, flatten)]
    pub config: AppConfig,
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

lazy_static! {
    pub static ref PROJECT_NAME: String = String::from("television");
    pub static ref PROJECT_NAME_UPPER: String = PROJECT_NAME.to_uppercase();
    pub static ref DATA_FOLDER: Option<PathBuf> =
        // if `TELEVISION_DATA` is set, use that as the data directory
        env::var_os(format!("{}_DATA", PROJECT_NAME_UPPER.clone())).map(PathBuf::from).or_else(|| {
            // otherwise, use the XDG data directory
            env::var_os("XDG_DATA_HOME").map(PathBuf::from).map(|p| p.join(PROJECT_NAME.as_str())).filter(|p| p.is_absolute())
        });
    pub static ref CONFIG_FOLDER: Option<PathBuf> =
        // if `TELEVISION_CONFIG` is set, use that as the television config directory
        env::var_os(format!("{}_CONFIG", PROJECT_NAME_UPPER.clone())).map(PathBuf::from).or_else(|| {
            // otherwise, use the XDG config directory + 'television'
            env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).map(|p| p.join(PROJECT_NAME.as_str())).filter(|p| p.is_absolute())
        });
}

const CONFIG_FILE_NAME: &str = "config.toml";

impl Config {
    // FIXME: default management is a bit of a mess right now
    #[allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]
    pub fn new() -> Result<Self> {
        // Load the default_config values as base defaults
        let default_config: Config = toml::from_str(DEFAULT_CONFIG)
            .wrap_err("Error parsing default config")?;

        // initialize the config builder
        let data_dir = get_data_dir();
        let config_dir = get_config_dir();

        std::fs::create_dir_all(&config_dir)
            .expect("Failed creating configuration directory");
        std::fs::create_dir_all(&data_dir)
            .expect("Failed creating data directory");

        if config_dir.join(CONFIG_FILE_NAME).is_file() {
            debug!("Found config file at {:?}", config_dir);

            let path = config_dir.join(CONFIG_FILE_NAME);
            let contents = std::fs::read_to_string(&path)?;

            let cfg: Config = toml::from_str(&contents)
                .wrap_err(format!("error parsing config: {path:?}"))?;

            // merge keybindings with default keybindings
            let keybindings = merge_keybindings(
                default_config.keybindings,
                &cfg.keybindings,
            );
            let cfg = Config { keybindings, ..cfg };

            debug!("Config: {:?}", cfg);
            Ok(cfg)
        } else {
            warn!("No config file found at {:?}, creating default configuration file at that location.", config_dir);
            // create the default configuration file in the user's config directory
            std::fs::write(config_dir.join(CONFIG_FILE_NAME), DEFAULT_CONFIG)?;
            Ok(default_config)
        }
    }
}

pub fn get_data_dir() -> PathBuf {
    let directory = if let Some(s) = DATA_FOLDER.clone() {
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
    let directory = if let Some(s) = CONFIG_FOLDER.clone() {
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
