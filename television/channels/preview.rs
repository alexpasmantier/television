use std::fmt::Display;

use serde::Deserialize;

use crate::{channels::entry::Entry, utils::strings::format_string};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, Deserialize)]
pub struct PreviewCommand {
    pub command: String,
    #[serde(default = "default_delimiter")]
    pub delimiter: String,
    #[serde(rename = "offset")]
    pub offset_expr: Option<String>,
}

pub const DEFAULT_DELIMITER: &str = " ";

/// The default delimiter to use for the preview command to use to split
/// entries into multiple referenceable parts.
#[allow(clippy::unnecessary_wraps)]
fn default_delimiter() -> String {
    DEFAULT_DELIMITER.to_string()
}

impl PreviewCommand {
    pub fn new(
        command: &str,
        delimiter: &str,
        offset_expr: Option<String>,
    ) -> Self {
        Self {
            command: command.to_string(),
            delimiter: delimiter.to_string(),
            offset_expr,
        }
    }

    /// Format the command with the entry name and provided placeholders.
    ///
    /// # Example
    /// ```
    /// use television::channels::{preview::PreviewCommand, entry::Entry};
    ///
    /// let command = PreviewCommand {
    ///     command: "something {} {2} {0}".to_string(),
    ///     delimiter: ":".to_string(),
    ///     offset_expr: None,
    /// };
    /// let entry = Entry::new("a:given:entry:to:preview".to_string());
    ///
    /// let formatted_command = command.format_with(&entry);
    ///
    /// assert_eq!(formatted_command, "something 'a:given:entry:to:preview' 'entry' 'a'");
    /// ```
    pub fn format_with(&self, entry: &Entry) -> String {
        format_string(&self.command, &entry.name, &self.delimiter)
    }
}

impl Display for PreviewCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channels::entry::Entry;

    #[test]
    fn test_format_command() {
        let command = PreviewCommand {
            command: "something {} {2} {0}".to_string(),
            delimiter: ":".to_string(),
            offset_expr: None,
        };
        let entry = Entry::new("an:entry:to:preview".to_string());
        let formatted_command = command.format_with(&entry);

        assert_eq!(
            formatted_command,
            "something 'an:entry:to:preview' 'to' 'an'"
        );
    }

    #[test]
    fn test_format_command_no_placeholders() {
        let command = PreviewCommand {
            command: "something".to_string(),
            delimiter: ":".to_string(),
            offset_expr: None,
        };
        let entry = Entry::new("an:entry:to:preview".to_string());
        let formatted_command = command.format_with(&entry);

        assert_eq!(formatted_command, "something");
    }

    #[test]
    fn test_format_command_with_global_placeholder_only() {
        let command = PreviewCommand {
            command: "something {}".to_string(),
            delimiter: ":".to_string(),
            offset_expr: None,
        };
        let entry = Entry::new("an:entry:to:preview".to_string());
        let formatted_command = command.format_with(&entry);

        assert_eq!(formatted_command, "something 'an:entry:to:preview'");
    }

    #[test]
    fn test_format_command_with_positional_placeholders_only() {
        let command = PreviewCommand {
            command: "something {0} -t {2}".to_string(),
            delimiter: ":".to_string(),
            offset_expr: None,
        };
        let entry = Entry::new("an:entry:to:preview".to_string());
        let formatted_command = command.format_with(&entry);

        assert_eq!(formatted_command, "something 'an' -t 'to'");
    }
}
