use std::path::PathBuf;

use rustc_hash::FxHashMap;

use crate::channels::cable::{CableChannelPrototype, CableChannels};
use color_eyre::Result;
use tracing::{debug, error};

use crate::config::get_config_dir;

/// Just a proxy struct to deserialize prototypes
#[derive(Debug, serde::Deserialize, Default)]
struct ChannelPrototypes {
    #[serde(rename = "cable_channel")]
    prototypes: Vec<CableChannelPrototype>,
}

const CABLE_FILE_NAME_SUFFIX: &str = "channels";
const CABLE_FILE_FORMAT: &str = "toml";

#[cfg(unix)]
const DEFAULT_CABLE_CHANNELS: &str =
    include_str!("../cable/unix-channels.toml");

#[cfg(not(unix))]
const DEFAULT_CABLE_CHANNELS: &str =
    include_str!("../cable/windows-channels.toml");

const DEFAULT_CABLE_CHANNELS_FILE_NAME: &str = "default_channels.toml";

/// Load the cable configuration from the config directory.
///
/// Cable is loaded by compiling all files that match the following
/// pattern in the config directory: `*channels.toml`.
///
/// # Example:
/// ```ignore
///   config_folder/
///   ├── cable_channels.toml
///   ├── my_channels.toml
///   └── windows_channels.toml
/// ```
pub fn load_cable_channels() -> Result<CableChannels> {
    let config_dir = get_config_dir();

    // list all files in the config directory
    let files = std::fs::read_dir(&config_dir)?;

    // filter the files that match the pattern
    let mut file_paths: Vec<PathBuf> = files
        .filter_map(|f| f.ok().map(|f| f.path()))
        .filter(|p| is_cable_file_format(p) && p.is_file())
        .collect();

    debug!("Found cable channel files: {:?}", file_paths);

    // If no cable provider files are found, write the default provider for the current
    // platform to the config directory
    if file_paths.is_empty() {
        debug!("No user defined cable channels found");
        // write the default cable channels to the config directory
        let default_channels_path =
            config_dir.join(DEFAULT_CABLE_CHANNELS_FILE_NAME);
        std::fs::write(&default_channels_path, DEFAULT_CABLE_CHANNELS)?;
        file_paths.push(default_channels_path);
    }

    let user_defined_prototypes = file_paths.iter().fold(
        Vec::<CableChannelPrototype>::new(),
        |mut acc, p| {
            match toml::from_str::<ChannelPrototypes>(
                &std::fs::read_to_string(p)
                    .expect("Unable to read configuration file"),
            ) {
                Ok(prototypes) => acc.extend(prototypes.prototypes),
                Err(e) => {
                    error!(
                        "Failed to parse cable channel file {:?}: {}",
                        p, e
                    );
                }
            }
            acc
        },
    );

    debug!("Loaded cable channels: {:?}", user_defined_prototypes);

    let mut cable_channels = FxHashMap::default();
    for prototype in user_defined_prototypes {
        cable_channels.insert(prototype.name.clone(), prototype);
    }
    Ok(CableChannels(cable_channels))
}

fn is_cable_file_format<P>(p: P) -> bool
where
    P: AsRef<std::path::Path>,
{
    let p = p.as_ref();
    p.file_stem()
        .and_then(|s| s.to_str())
        .map_or(false, |s| s.ends_with(CABLE_FILE_NAME_SUFFIX))
        && p.extension()
            .and_then(|e| e.to_str())
            .map_or(false, |e| e.to_lowercase() == CABLE_FILE_FORMAT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_cable_file() {
        let path = std::path::Path::new("cable_channels.toml");
        assert!(is_cable_file_format(path));

        let path = std::path::Path::new(DEFAULT_CABLE_CHANNELS_FILE_NAME);
        assert!(is_cable_file_format(path));
    }
}
