use anyhow::Result;
use regex::Regex;
use tracing::debug;

use crate::channels::entry::{PreviewCommand, PreviewType};

#[derive(Debug, Clone, PartialEq)]
pub enum PreviewKind {
    Command(PreviewCommand),
    Builtin(PreviewType),
    None,
}

/// Parses the preview command to determine if it is a built-in (i.e. ":files:") or custom command.
///
/// # Example:
/// ```
/// use television::channels::entry::{PreviewCommand, PreviewType};
/// use television::channels::cable::preview::{parse_preview_kind, PreviewKind};
///
/// let command = PreviewCommand::new("cat {0}", ":");
/// let preview_kind = parse_preview_kind(&command).unwrap();
/// assert_eq!(preview_kind, PreviewKind::Command(command));
/// ```
pub fn parse_preview_kind(command: &PreviewCommand) -> Result<PreviewKind> {
    debug!("Parsing preview kind for command: {:?}", command);
    let re = Regex::new(r"^\:(\w+)\:$").unwrap();
    if let Some(captures) = re.captures(&command.command) {
        let preview_type = PreviewType::try_from(&captures[1])?;
        Ok(PreviewKind::Builtin(preview_type))
    } else {
        Ok(PreviewKind::Command(command.clone()))
    }
}
