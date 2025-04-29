use rustc_hash::FxHashMap;
use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

use crate::cable::ChannelPrototypes;

#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
pub struct CableChannelPrototype {
    pub name: String,
    pub source_command: String,
    #[serde(default)]
    pub interactive: bool,
    pub preview_command: Option<String>,
    #[serde(default = "default_delimiter")]
    pub preview_delimiter: Option<String>,
}

impl CableChannelPrototype {
    pub fn new(
        name: &str,
        source_command: &str,
        interactive: bool,
        preview_command: Option<String>,
        preview_delimiter: Option<String>,
    ) -> Self {
        Self {
            name: name.to_string(),
            source_command: source_command.to_string(),
            interactive,
            preview_command,
            preview_delimiter,
        }
    }
}

const DEFAULT_PROTOTYPE_NAME: &str = "files";
const DEFAULT_SOURCE_COMMAND: &str = "fd -t f";
const DEFAULT_PREVIEW_COMMAND: &str = ":files:";

impl Default for CableChannelPrototype {
    fn default() -> Self {
        Self {
            name: DEFAULT_PROTOTYPE_NAME.to_string(),
            source_command: DEFAULT_SOURCE_COMMAND.to_string(),
            interactive: false,
            preview_command: Some(DEFAULT_PREVIEW_COMMAND.to_string()),
            preview_delimiter: Some(DEFAULT_DELIMITER.to_string()),
        }
    }
}

pub const DEFAULT_DELIMITER: &str = " ";

#[allow(clippy::unnecessary_wraps)]
fn default_delimiter() -> Option<String> {
    Some(DEFAULT_DELIMITER.to_string())
}

impl Display for CableChannelPrototype {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct CableChannels(pub FxHashMap<String, CableChannelPrototype>);

impl Deref for CableChannels {
    type Target = FxHashMap<String, CableChannelPrototype>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(unix)]
const DEFAULT_CABLE_CHANNELS_FILE: &str =
    include_str!("../../../cable/unix-channels.toml");
#[cfg(not(unix))]
const DEFAULT_CABLE_CHANNELS_FILE: &str =
    include_str!("../../cable/windows-channels.toml");

impl Default for CableChannels {
    /// Fallback to the default cable channels specification (the template file
    /// included in the repo).
    fn default() -> Self {
        let pts =
            toml::from_str::<ChannelPrototypes>(DEFAULT_CABLE_CHANNELS_FILE)
                .expect("Unable to parse default cable channels");
        let mut channels = FxHashMap::default();
        for prototype in pts.prototypes {
            channels.insert(prototype.name.clone(), prototype);
        }
        CableChannels(channels)
    }
}
