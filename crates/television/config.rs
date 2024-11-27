#![allow(clippy::module_name_repetitions)]
use std::{env, path::PathBuf};

use color_eyre::{eyre::Context, Result};
use directories::ProjectDirs;
pub use keybindings::parse_key;
pub use keybindings::KeyBindings;
use lazy_static::lazy_static;
use previewers::PreviewersConfig;
use serde::Deserialize;
use styles::Styles;
use tracing::{debug, warn};
use ui::UiConfig;

mod keybindings;
mod previewers;
mod styles;
mod ui;

const CONFIG: &str = include_str!("../../.config/config.toml");

#[allow(dead_code, clippy::module_name_repetitions)]
#[derive(Clone, Debug, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub data_dir: PathBuf,
    #[serde(default)]
    pub config_dir: PathBuf,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    #[allow(clippy::struct_field_names)]
    #[serde(default, flatten)]
    pub config: AppConfig,
    #[serde(default)]
    pub keybindings: KeyBindings,
    #[serde(default)]
    pub styles: Styles,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub previewers: PreviewersConfig,
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
    #[allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]
    pub fn new() -> Result<Self> {
        // Load the default_config values as base defaults
        let default_config: Config =
            toml::from_str(CONFIG).expect("default config should be valid");

        // initialize the config builder
        let data_dir = get_data_dir();
        let config_dir = get_config_dir();
        let mut builder = config::Config::builder()
            .set_default("data_dir", data_dir.to_str().unwrap())?
            .set_default("config_dir", config_dir.to_str().unwrap())?
            .set_default("ui", UiConfig::default())?
            .set_default("previewers", PreviewersConfig::default())?;

        // Load the user's config file
        let source = config::File::from(config_dir.join(CONFIG_FILE_NAME))
            .format(config::FileFormat::Toml)
            .required(false);
        builder = builder.add_source(source);

        if config_dir.join(CONFIG_FILE_NAME).is_file() {
            debug!("Found config file at {:?}", config_dir);
            let mut cfg: Self =
                builder.build()?.try_deserialize().with_context(|| {
                    format!(
                        "Error parsing config file at {:?}",
                        config_dir.join(CONFIG_FILE_NAME)
                    )
                })?;

            for (mode, default_bindings) in default_config.keybindings.iter() {
                let user_bindings = cfg.keybindings.entry(*mode).or_default();
                for (command, key) in default_bindings {
                    user_bindings
                        .entry(command.clone())
                        .or_insert_with(|| *key);
                }
            }

            for (mode, default_styles) in default_config.styles.iter() {
                let user_styles = cfg.styles.entry(*mode).or_default();
                for (style_key, style) in default_styles {
                    user_styles.entry(style_key.clone()).or_insert(*style);
                }
            }

            debug!("Config: {:?}", cfg);
            Ok(cfg)
        } else {
            warn!("No config file found at {:?}", config_dir);
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
        PathBuf::from(".").join(".data")
    };
    directory
}

pub fn get_config_dir() -> PathBuf {
    let directory = if let Some(s) = CONFIG_FOLDER.clone() {
        debug!("Using config directory: {:?}", s);
        s
    } else if let Some(proj_dirs) = project_directory() {
        debug!("Falling back to default config dir");
        proj_dirs.config_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".config")
    };
    directory
}

fn project_directory() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "", env!("CARGO_PKG_NAME"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::config::keybindings::parse_key;
    use crate::television::Mode;

    #[test]
    fn test_config() -> Result<()> {
        let c = Config::new()?;
        assert_eq!(
            c.keybindings
                .get(&Mode::Channel)
                .unwrap()
                .get(&Action::Quit),
            Some(&parse_key("esc").unwrap())
        );
        Ok(())
    }
}
