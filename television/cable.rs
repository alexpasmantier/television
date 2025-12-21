use crate::{
    action::Action, channels::prototypes::ChannelPrototype,
    config::Keybindings, errors::unknown_channel_exit, event::Key,
};
use rustc_hash::FxHashMap;
use std::{
    ops::Deref,
    path::{Path, PathBuf},
};
use tracing::{debug, error};
use walkdir::WalkDir;

/// A neat `HashMap` of channel prototypes indexed by their name.
///
/// This is used to store cable channel prototypes throughout the application
/// in a way that facilitates answering questions like "what's the prototype
/// for `files`?" or "does this channel exist?".
#[derive(Debug, serde::Deserialize, Clone, Default)]
pub struct Cable(pub FxHashMap<String, ChannelPrototype>);

impl Deref for Cable {
    type Target = FxHashMap<String, ChannelPrototype>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Cable {
    /// Get a channel prototype by its name.
    ///
    /// # Panics
    /// If the channel does not exist.
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

    /// Get a hash map of channel names and their related shortcut bindings.
    ///
    /// (e.g. "files" -> "F1", "dirs" -> "F2", etc.)
    pub fn get_channels_shortcut_keybindings(&self) -> Keybindings {
        let bindings: Vec<(Key, Action)> = self
            .iter()
            .filter_map(|(name, prototype)| {
                if let Some(keybindings) = &prototype.keybindings {
                    keybindings.shortcut.as_ref().map(|key| {
                        (*key, Action::SwitchToChannel(name.clone()))
                    })
                } else {
                    None
                }
            })
            .collect();

        Keybindings::from(bindings)
    }

    /// Get a channel prototype's shortcut binding.
    ///
    /// E.g. if the channel is "files" and the shortcut is "F1",
    /// this will return `Some(Key::F(1))`.
    pub fn get_channel_shortcut(&self, channel_name: &str) -> Option<Key> {
        self.get(channel_name)
            .and_then(|prototype| prototype.keybindings.as_ref())
            .and_then(|keybindings| keybindings.shortcut.as_ref())
            .copied()
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

fn load_prototypes(
    toml_prototypes: FxHashMap<PathBuf, String>,
) -> Vec<ChannelPrototype> {
    toml_prototypes
        .into_iter()
        .filter_map(|(path, content)| {
            match toml::from_str::<ChannelPrototype>(&content) {
                Ok(prototype) => {
                    debug!(
                        "Loaded cable channel prototype from {}: {}",
                        path.display(),
                        prototype.metadata.name
                    );
                    Some(prototype)
                }
                Err(e) => {
                    eprintln!(
                        "Failed to parse cable channel file {}: {}",
                        path.display(),
                        e
                    );
                    None
                }
            }
        })
        .collect()
}

/// Load cable channels from the provided directory.
///
/// The resulting cable channels are a combination of the default cable channels
/// merged with any user-defined channels found in the specified directory, custom
/// channels taking precedence over defaults. For a list of default cable channels,
/// see `DEFAULT_CABLE_FILES`.
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
pub fn load_cable<P>(cable_dir: P) -> Cable
where
    P: AsRef<Path>,
{
    let cable_dir = cable_dir.as_ref();
    debug!("Using cable directory: {}", cable_dir.to_string_lossy());
    let cable_files = get_cable_files(cable_dir);
    debug!("Found cable channel files: {:?}", cable_files);

    let mut cable_map: FxHashMap<PathBuf, String> = DEFAULT_CABLE_FILES
        .iter()
        .map(|(name, content)| (PathBuf::from(*name), (*content).to_string()))
        .collect();

    cable_map.extend(
        cable_files
            .into_iter()
            .filter_map(|path| match std::fs::read_to_string(&path) {
                Ok(content) => {
                    Some((path.file_name().unwrap().into(), content))
                }
                Err(e) => {
                    error!(
                        "Failed to read cable channel file {}: {}",
                        path.display(),
                        e
                    );
                    None
                }
            })
            .collect::<FxHashMap<_, _>>(),
    );

    let prototypes = load_prototypes(cable_map);

    debug!("Loaded {} cable channels", prototypes.len());

    Cable::from_prototypes(prototypes)
}

#[cfg(unix)]
const DEFAULT_CABLE_FILES: &[(&str, &str)] = &[
    ("alias.toml", include_str!("../cable/unix/alias.toml")),
    (
        "bash-history.toml",
        include_str!("../cable/unix/bash-history.toml"),
    ),
    ("dirs.toml", include_str!("../cable/unix/dirs.toml")),
    (
        "docker-images.toml",
        include_str!("../cable/unix/docker-images.toml"),
    ),
    ("env.toml", include_str!("../cable/unix/env.toml")),
    ("files.toml", include_str!("../cable/unix/files.toml")),
    (
        "git-branch.toml",
        include_str!("../cable/unix/git-branch.toml"),
    ),
    ("git-diff.toml", include_str!("../cable/unix/git-diff.toml")),
    ("git-log.toml", include_str!("../cable/unix/git-log.toml")),
    (
        "git-repos.toml",
        include_str!("../cable/unix/git-repos.toml"),
    ),
    ("text.toml", include_str!("../cable/unix/text.toml")),
];

#[cfg(windows)]
const DEFAULT_CABLE_FILES: &[(&str, &str)] = &[
    ("alias.toml", include_str!("../cable/windows/alias.toml")),
    ("dirs.toml", include_str!("../cable/windows/dirs.toml")),
    (
        "docker-images.toml",
        include_str!("../cable/windows/docker-images.toml"),
    ),
    ("env.toml", include_str!("../cable/windows/env.toml")),
    ("files.toml", include_str!("../cable/windows/files.toml")),
    (
        "git-branch.toml",
        include_str!("../cable/windows/git-branch.toml"),
    ),
    (
        "git-diff.toml",
        include_str!("../cable/windows/git-diff.toml"),
    ),
    (
        "git-log.toml",
        include_str!("../cable/windows/git-log.toml"),
    ),
    (
        "git-repos.toml",
        include_str!("../cable/windows/git-repos.toml"),
    ),
    ("text.toml", include_str!("../cable/windows/text.toml")),
];
