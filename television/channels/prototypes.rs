use rustc_hash::FxHashMap;
use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

use crate::{cable::CableSpec, channels::preview::PreviewCommand};

/// A prototype for cable channels.
///
/// This can be seen as a cable channel specification, which is used to
/// create a cable channel.
///
/// The prototype contains the following fields:
/// - `name`: The name of the channel. This will be used to identify the
///     channel throughout the application and in UI menus.
/// - `source_command`: The command to run to get the source for the channel.
///     This is a shell command that will be run in the background.
/// - `interactive`: Whether the source command should be run in an interactive
///     shell. This is useful for commands that need the user's environment e.g.
///     `alias`.
/// - `preview_command`: The command to run on each entry to get the preview
///     for the channel. If this is not `None`, the channel will display a preview
///     pane with the output of this command.
/// - `preview_delimiter`: The delimiter to use to split an entry into
///     multiple parts that can then be referenced in the preview command (e.g.
///     `{1} + {2}`).
/// - `preview_offset`: a litteral expression that will be interpreted later on
///     in order to determine the vertical offset at which the preview should be
///     displayed.
///
/// # Example
/// The default files channel might look something like this:
/// ```toml
/// [[cable_channel]]
/// name = "files"
/// source_command = "fd -t f"
/// preview_command = "cat {}"
/// ```
#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
pub struct ChannelPrototype {
    pub name: String,
    pub source_command: String,
    #[serde(default)]
    pub interactive: bool,
    pub preview_command: Option<String>,
    #[serde(default = "default_delimiter")]
    pub preview_delimiter: Option<String>,
    pub preview_offset: Option<String>,
}

const STDIN_CHANNEL_NAME: &str = "stdin";
const STDIN_SOURCE_COMMAND: &str = "cat";

impl ChannelPrototype {
    pub fn new(
        name: &str,
        source_command: &str,
        interactive: bool,
        preview_command: Option<String>,
        preview_delimiter: Option<String>,
        preview_offset: Option<String>,
    ) -> Self {
        Self {
            name: name.to_string(),
            source_command: source_command.to_string(),
            interactive,
            preview_command,
            preview_delimiter,
            preview_offset,
        }
    }

    pub fn stdin(preview: Option<PreviewCommand>) -> Self {
        match preview {
            Some(PreviewCommand {
                command,
                delimiter,
                offset_expr,
            }) => Self {
                name: STDIN_CHANNEL_NAME.to_string(),
                source_command: STDIN_SOURCE_COMMAND.to_string(),
                interactive: false,
                preview_command: Some(command),
                preview_delimiter: Some(delimiter),
                preview_offset: offset_expr,
            },
            None => Self {
                name: STDIN_CHANNEL_NAME.to_string(),
                source_command: STDIN_SOURCE_COMMAND.to_string(),
                interactive: false,
                preview_command: None,
                preview_delimiter: None,
                preview_offset: None,
            },
        }
    }

    pub fn preview_command(&self) -> Option<PreviewCommand> {
        self.into()
    }
}

const DEFAULT_PROTOTYPE_NAME: &str = "files";
pub const DEFAULT_DELIMITER: &str = " ";

impl Default for ChannelPrototype {
    fn default() -> Self {
        Cable::default()
            .get(DEFAULT_PROTOTYPE_NAME)
            .cloned()
            .unwrap()
    }
}

/// The default delimiter to use for the preview command to use to split
/// entries into multiple referenceable parts.
#[allow(clippy::unnecessary_wraps)]
fn default_delimiter() -> Option<String> {
    Some(DEFAULT_DELIMITER.to_string())
}

impl Display for ChannelPrototype {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

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

/// A default cable channels specification that is compiled into the
/// application.
#[cfg(unix)]
const DEFAULT_CABLE_CHANNELS_FILE: &str =
    include_str!("../../cable/unix-channels.toml");
/// A default cable channels specification that is compiled into the
/// application.
#[cfg(not(unix))]
const DEFAULT_CABLE_CHANNELS_FILE: &str =
    include_str!("../../cable/windows-channels.toml");

impl Default for Cable {
    /// Fallback to the default cable channels specification (the template file
    /// included in the repo).
    fn default() -> Self {
        let s = toml::from_str::<CableSpec>(DEFAULT_CABLE_CHANNELS_FILE)
            .expect("Unable to parse default cable channels");
        let mut prototypes = FxHashMap::default();
        for prototype in s.prototypes {
            prototypes.insert(prototype.name.clone(), prototype);
        }
        Cable(prototypes)
    }
}
