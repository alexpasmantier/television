use std::fmt::Display;

use lazy_regex::{regex, Lazy, Regex};

use super::entry::Entry;

static CMD_RE: &Lazy<Regex> = regex!(r"\{(\d+)\}");

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

    /// Format the command with the entry name and provided placeholders.
    ///
    /// # Example
    /// ```
    /// use television::channels::{preview::PreviewCommand, entry::Entry};
    ///
    /// let command = PreviewCommand {
    ///     command: "something {} {2} {0}".to_string(),
    ///     delimiter: ":".to_string(),
    /// };
    /// let entry = Entry::new("a:given:entry:to:preview".to_string());
    ///
    /// let formatted_command = command.format_with(&entry);
    ///
    /// assert_eq!(formatted_command, "something 'a:given:entry:to:preview' 'entry' 'a'");
    /// ```
    pub fn format_with(&self, entry: &Entry) -> String {
        let parts = entry.name.split(&self.delimiter).collect::<Vec<&str>>();

        let mut formatted_command = self
            .command
            .replace("{}", format!("'{}'", entry.name).as_str());

        formatted_command = CMD_RE
            .replace_all(&formatted_command, |caps: &regex::Captures| {
                let index =
                    caps.get(1).unwrap().as_str().parse::<usize>().unwrap();
                format!("'{}'", parts[index])
            })
            .to_string();

        formatted_command
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
        };
        let entry = Entry::new("an:entry:to:preview".to_string());
        let formatted_command = command.format_with(&entry);

        assert_eq!(formatted_command, "something 'an' -t 'to'");
    }
}
