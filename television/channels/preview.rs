use std::fmt::Display;

use anyhow::Result;
use regex::Regex;
use strum::EnumString;
use tracing::debug;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct PreviewCommand {
    pub command: String,
    pub delimiter: String,
}

impl PreviewCommand {
    pub fn new(command: &str, delimiter: &str) -> Self {
        Self {
            command: command.to_string(),
            delimiter: delimiter.to_string(),
        }
    }
}

impl Display for PreviewCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum PreviewType {
    Basic,
    EnvVar,
    Files,
    #[strum(disabled)]
    Command(PreviewCommand),
    #[default]
    None,
}

/// Parses the preview command to determine the preview type.
///
/// This checks if the command matches the builtin pattern `:{preview_type}:`
/// and then falls back to the command type if it doesn't.
///
/// # Example:
/// ```
/// use television::channels::preview::{parse_preview_type, PreviewCommand, PreviewType};
///
/// let command = PreviewCommand::new("cat {0}", ":");
/// let preview_type = parse_preview_type(&command).unwrap();
/// assert_eq!(preview_type, PreviewType::Command(command));
/// ```
pub fn parse_preview_type(command: &PreviewCommand) -> Result<PreviewType> {
    debug!("Parsing preview kind for command: {:?}", command);
    let re = Regex::new(r"^\:(\w+)\:$").unwrap();
    if let Some(captures) = re.captures(&command.command) {
        let preview_type = PreviewType::try_from(&captures[1])?;
        Ok(preview_type)
    } else {
        Ok(PreviewType::Command(command.clone()))
    }
}
