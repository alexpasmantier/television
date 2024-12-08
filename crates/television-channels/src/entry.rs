use std::{fmt::Display, path::PathBuf};

use devicons::FileIcon;

// NOTE: having an enum for entry types would be nice since it would allow
// having a nicer implementation for transitions between channels. This would
// permit implementing `From<EntryType>` for channels which would make the
// channel convertible from any other that yields `EntryType`.
// This needs pondering since it does bring another level of abstraction and
// adds a layer of complexity.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Entry {
    /// The name of the entry.
    pub name: String,
    /// An optional value associated with the entry.
    pub value: Option<String>,
    /// The optional ranges for matching characters in the name.
    pub name_match_ranges: Option<Vec<(u32, u32)>>,
    /// The optional ranges for matching characters in the value.
    pub value_match_ranges: Option<Vec<(u32, u32)>>,
    /// The optional icon associated with the entry.
    pub icon: Option<FileIcon>,
    /// The optional line number associated with the entry.
    pub line_number: Option<usize>,
    /// The type of preview associated with the entry.
    pub preview_type: PreviewType,
}

impl Entry {
    /// Create a new entry with the given name and preview type.
    ///
    /// Additional fields can be set using the builder pattern.
    /// ```
    /// use television_channels::entry::{Entry, PreviewType};
    /// use devicons::FileIcon;
    ///
    /// let entry = Entry::new("name".to_string(), PreviewType::EnvVar)
    ///                 .with_value("value".to_string())
    ///                 .with_name_match_ranges(vec![(0, 1)])
    ///                 .with_value_match_ranges(vec![(0, 1)])
    ///                 .with_icon(FileIcon::default())
    ///                 .with_line_number(0);
    /// ```
    ///
    /// # Arguments
    /// * `name` - The name of the entry.
    /// * `preview_type` - The type of preview associated with the entry.
    ///
    /// # Returns
    /// A new entry with the given name and preview type.
    /// The other fields are set to `None` by default.
    pub fn new(name: String, preview_type: PreviewType) -> Self {
        Self {
            name,
            value: None,
            name_match_ranges: None,
            value_match_ranges: None,
            icon: None,
            line_number: None,
            preview_type,
        }
    }

    pub fn with_value(mut self, value: String) -> Self {
        self.value = Some(value);
        self
    }
    #[allow(clippy::needless_return)]
    pub fn minimal_name_match_ranges(self) -> Option<Vec<(u32, u32)>> {
        // This method takes the existing `name_match_ranges`
        // and merges contiguous ranges into the minimal equivalent
        //   set of ranges. If no ranges exist, it returns `None`.
        if let Some(name_match_ranges) = self.name_match_ranges {
            let minimal_name_match_ranges: Vec<(u32, u32)> = name_match_ranges
                .into_iter()
                .fold(Vec::new(), |mut acc, x| {
                    if let Some(last) = acc.last_mut() {
                        if last.1 == x.0 {
                            last.1 = x.1
                        } else {
                            acc.push(x);
                        }
                    } else {
                        acc.push(x);
                    }
                    return acc;
                });
            Some(minimal_name_match_ranges)
        } else {
            None
        }
    }

    pub fn with_name_match_ranges(
        mut self,
        name_match_ranges: Vec<(u32, u32)>,
    ) -> Self {
        self.name_match_ranges = Some(name_match_ranges);
        self
    }

    pub fn with_value_match_ranges(
        mut self,
        value_match_ranges: Vec<(u32, u32)>,
    ) -> Self {
        self.value_match_ranges = Some(value_match_ranges);
        self
    }

    pub fn with_icon(mut self, icon: FileIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn with_line_number(mut self, line_number: usize) -> Self {
        self.line_number = Some(line_number);
        self
    }

    pub fn stdout_repr(&self) -> String {
        let mut repr = self.name.clone();
        if PathBuf::from(&repr).exists()
            && repr.contains(|c| char::is_ascii_whitespace(&c))
        {
            repr.insert(0, '\'');
            repr.push('\'');
        }
        if let Some(line_number) = self.line_number {
            repr.push_str(&format!(":{line_number}"));
        }
        repr
    }
}

pub const ENTRY_PLACEHOLDER: Entry = Entry {
    name: String::new(),
    value: None,
    name_match_ranges: None,
    value_match_ranges: None,
    icon: None,
    line_number: None,
    preview_type: PreviewType::EnvVar,
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum PreviewType {
    #[default]
    Basic,
    EnvVar,
    Files,
    Command(PreviewCommand),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_name_match_ranges_none() {
        let entry = Entry {
            name: "test".to_string(),
            value: None,
            name_match_ranges: None,
            value_match_ranges: None,
            icon: None,
            line_number: None,
            preview_type: PreviewType::Files,
        };
        assert_eq!(entry.minimal_name_match_ranges(), None);
    }
    #[test]
    fn test_minimal_name_match_ranges_empty() {
        let entry = Entry {
            name: "test".to_string(),
            value: None,
            name_match_ranges: Some(vec![]),
            value_match_ranges: None,
            icon: None,
            line_number: None,
            preview_type: PreviewType::Files,
        };

        assert_eq!(entry.minimal_name_match_ranges(), Some(vec![]));
    }
    #[test]
    fn test_minimal_name_match_ranges_non_contiguous() {
        let entry = Entry {
            name: "test".to_string(),
            value: None,
            name_match_ranges: Some(vec![(0, 1), (2, 4), (6, 9)]),
            value_match_ranges: None,
            icon: None,
            line_number: None,
            preview_type: PreviewType::Files,
        };
        assert_eq!(
            entry.minimal_name_match_ranges(),
            Some(vec![(0, 1), (2, 4), (6, 9)])
        );
    }

    #[test]
    fn test_minimal_name_match_ranges_contiguous() {
        let entry = Entry {
            name: "test".to_string(),
            value: None,
            name_match_ranges: Some(vec![(0, 1), (1, 2), (2, 3), (3, 4)]),
            value_match_ranges: None,
            icon: None,
            line_number: None,
            preview_type: PreviewType::Files,
        };

        assert_eq!(entry.minimal_name_match_ranges(), Some(vec![(0, 4)]));
    }

    #[test]
    fn test_minimal_name_match_ranges_both_contiguous_and_non_contiguous() {
        let entry = Entry {
            name: "test".to_string(),
            value: None,
            name_match_ranges: Some(vec![(0, 1), (2, 3), (3, 4)]),
            value_match_ranges: None,
            icon: None,
            line_number: None,
            preview_type: PreviewType::Files,
        };

        assert_eq!(
            entry.minimal_name_match_ranges(),
            Some(vec![(0, 1), (2, 4)])
        );
    }
}
