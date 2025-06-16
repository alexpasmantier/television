use colored::Colorize;
use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

use rustc_hash::FxHashMap;
use tracing::{debug, error};
use walkdir::WalkDir;

use crate::{
    channels::prototypes::ChannelPrototype, cli::unknown_channel_exit,
};

/// A neat `HashMap` of channel prototypes indexed by their name.
///
/// This is used to store cable channel prototypes throughout the application
/// in a way that facilitates answering questions like "what's the prototype
/// for `files`?" or "does this channel exist?".
#[derive(Debug, serde::Deserialize, Clone)]
pub struct Cable(pub FxHashMap<String, ChannelPrototype>);

impl Deref for Cable {
    type Target = FxHashMap<String, ChannelPrototype>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Cable {
    pub fn get_channel(&self, name: &str) -> ChannelPrototype {
        self.get(name)
            .cloned()
            .unwrap_or_else(|| unknown_channel_exit(name))
    }

    pub fn has_channel(&self, name: &str) -> bool {
        self.contains_key(name)
    }

    pub fn from_prototypes(prototypes: Vec<ChannelPrototype>) -> Self {
        let mut map = FxHashMap::default();
        for prototype in prototypes {
            map.insert(prototype.metadata.name.clone(), prototype);
        }
        Cable(map)
    }
}

/// Just a proxy struct to deserialize prototypes
#[derive(Debug, serde::Deserialize, Default)]
pub struct CableSpec {
    #[serde(rename = "cable_channel")]
    pub prototypes: Vec<ChannelPrototype>,
}

pub const CHANNEL_FILE_FORMAT: &str = "toml";
/// ```ignore
///   config_folder/
///   ├── config.toml
///   └── cable/
///    ├── channel_1.toml
///    ├── channel_2.toml
///    └── ...
/// ```
pub const CABLE_DIR_NAME: &str = "cable";

fn get_cable_files<P>(cable_dir: P) -> Vec<PathBuf>
where
    P: AsRef<Path>,
{
    WalkDir::new(cable_dir)
        .into_iter()
        .map(|e| e.unwrap().path().to_owned())
        .filter(|p| {
            p.is_file()
                && p.extension().is_some()
                && p.extension().unwrap() == CHANNEL_FILE_FORMAT
        })
        .collect::<Vec<_>>()
}

fn load_prototypes<I>(cable_files: I) -> Vec<ChannelPrototype>
where
    I: IntoIterator<Item = PathBuf>,
{
    cable_files
        .into_iter()
        .filter_map(|p| match std::fs::read_to_string(&p) {
            Ok(content) => {
                match toml::from_str::<ChannelPrototype>(&content) {
                    Ok(prototype) => {
                        debug!(
                            "Loaded cable channel prototype from {:?}: {}",
                            p, prototype.metadata.name
                        );
                        Some(prototype)
                    }
                    Err(e) => {
                        error!(
                            "Failed to parse cable channel file {:?}: {}",
                            p, e
                        );
                        None
                    }
                }
            }
            Err(e) => {
                error!("Failed to read cable channel file {:?}: {}", p, e);
                None
            }
        })
        .collect()
}

/// Load cable channels from the config directory.
///
/// Cable is loaded by compiling all files located in the `cable/` subdirectory
/// of the user's configuration directory, unless a custom directory is provided.
///
/// # Example:
/// ```ignore
///   config_folder/
///   ├── config.toml
///   └── cable/
///    ├── channel_1.toml
///    ├── channel_2.toml
///    └── ...
/// ```
pub fn load_cable<P>(cable_dir: P) -> Option<Cable>
where
    P: AsRef<Path>,
{
    let cable_dir = cable_dir.as_ref();
    debug!("Using cable directory: {}", cable_dir.to_string_lossy());
    let cable_files = get_cable_files(cable_dir);
    debug!("Found cable channel files: {:?}", cable_files);

    if cable_files.is_empty() {
        println!(
            "{}",
            "It seems you don't have any cable channels configured yet.\n"
                .blue()
                .bold()
        );
        println!(
            "Run {} to get the latest default cable channels and/or add your own in `{}`.\n",
            "`tv update-channels`".green().bold(),
            cable_dir.to_string_lossy().yellow().bold()
        );
        println!(
            "More info: {}",
            "https://github.com/alexpasmantier/television/blob/main/README.md"
                .blue()
                .bold()
        );
        return None;
    }

    let prototypes = load_prototypes(cable_files);

    debug!("Loaded {} cable channels", prototypes.len());

    Some(Cable::from_prototypes(prototypes))
}
