use std::path::PathBuf;

use rustc_hash::FxHashMap;

use anyhow::Result;
use tracing::{debug, error};

use crate::{
    channels::cable::prototypes::{CableChannelPrototype, CableChannels},
    config::get_config_dir,
};

/// Just a proxy struct to deserialize prototypes
#[derive(Debug, serde::Deserialize, Default)]
pub struct SerializedChannelPrototypes {
    #[serde(rename = "cable_channel")]
    pub prototypes: Vec<CableChannelPrototype>,
}

const CABLE_FILE_NAME_SUFFIX: &str = "channels";
const CABLE_FILE_FORMAT: &str = "toml";

#[cfg(unix)]
const DEFAULT_CABLE_CHANNELS: &str =
    include_str!("../cable/unix-channels.toml");

#[cfg(not(unix))]
const DEFAULT_CABLE_CHANNELS: &str =
    include_str!("../cable/windows-channels.toml");

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
    let file_paths: Vec<PathBuf> = files
        .filter_map(|f| f.ok().map(|f| f.path()))
        .filter(|p| is_cable_file_format(p) && p.is_file())
        .collect();

    debug!("Found cable channel files: {:?}", file_paths);
    if file_paths.is_empty() {
        debug!("No user defined cable channels found");
    }

    let default_prototypes =
        toml::from_str::<SerializedChannelPrototypes>(DEFAULT_CABLE_CHANNELS)
            .expect("Failed to parse default cable channels");

    let prototypes = file_paths.iter().fold(
        Vec::<CableChannelPrototype>::new(),
        |mut acc, p| {
            match toml::from_str::<SerializedChannelPrototypes>(
                &std::fs::read_to_string(p)
                    .expect("Unable to read configuration file"),
            ) {
                Ok(pts) => acc.extend(pts.prototypes),
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

    debug!(
        "Loaded {} default and {} custom prototypes",
        default_prototypes.prototypes.len(),
        prototypes.len()
    );

    let mut cable_channels = FxHashMap::default();
    // custom prototypes take precedence over default ones
    for prototype in default_prototypes
        .prototypes
        .into_iter()
        .chain(prototypes.into_iter())
    {
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
    }
}
