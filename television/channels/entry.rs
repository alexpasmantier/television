use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

use devicons::FileIcon;
use strum::EnumString;

// NOTE: having an enum for entry types would be nice since it would allow
// having a nicer implementation for transitions between channels. This would
// permit implementing `From<EntryType>` for channels which would make the
// channel convertible from any other that yields `EntryType`.
// This needs pondering since it does bring another level of abstraction and
// adds a layer of complexity.
#[derive(Clone, Debug, Eq)]
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

impl Hash for Entry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        if let Some(line_number) = self.line_number {
            line_number.hash(state);
        }
    }
}

impl PartialEq<Entry> for &Entry {
    fn eq(&self, other: &Entry) -> bool {
        self.name == other.name
            && (self.line_number.is_none() && other.line_number.is_none()
                || self.line_number == other.line_number)
    }
}

impl PartialEq<Entry> for Entry {
    fn eq(&self, other: &Entry) -> bool {
        self.name == other.name
            && (self.line_number.is_none() && other.line_number.is_none()
                || self.line_number == other.line_number)
    }
}

#[allow(clippy::needless_return)]
pub fn merge_ranges(ranges: &[(u32, u32)]) -> Vec<(u32, u32)> {
    ranges.iter().fold(
        Vec::new(),
        |mut acc: Vec<(u32, u32)>, x: &(u32, u32)| {
            if let Some(last) = acc.last_mut() {
                if last.1 == x.0 {
                    last.1 = x.1;
                } else {
                    acc.push(*x);
                }
            } else {
                acc.push(*x);
            }
            return acc;
        },
    )
}

impl Entry {
    /// Create a new entry with the given name and preview type.
    ///
    /// Additional fields can be set using the builder pattern.
    /// ```
    /// use television::channels::entry::{Entry, PreviewType};
    /// use devicons::FileIcon;
    ///
    /// let entry = Entry::new("name".to_string(), PreviewType::EnvVar)
    ///                 .with_value("value".to_string())
    ///                 .with_name_match_ranges(&vec![(0, 1)])
    ///                 .with_value_match_ranges(&vec![(0, 1)])
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

    pub fn with_name_match_ranges(
        mut self,
        name_match_ranges: &[(u32, u32)],
    ) -> Self {
        self.name_match_ranges = Some(merge_ranges(name_match_ranges));
        self
    }

    pub fn with_value_match_ranges(
        mut self,
        value_match_ranges: &[(u32, u32)],
    ) -> Self {
        self.value_match_ranges = Some(merge_ranges(value_match_ranges));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let ranges: Vec<(u32, u32)> = vec![];
        assert_eq!(merge_ranges(&ranges), Vec::<(u32, u32)>::new());
    }

    #[test]
    fn test_single_range() {
        let ranges = vec![(1, 3)];
        assert_eq!(merge_ranges(&ranges), vec![(1, 3)]);
    }

    #[test]
    fn test_contiguous_ranges() {
        let ranges = vec![(1, 2), (2, 3), (3, 4), (4, 5)];
        assert_eq!(merge_ranges(&ranges), vec![(1, 5)]);
    }

    #[test]
    fn test_non_contiguous_ranges() {
        let ranges = vec![(1, 2), (3, 4), (5, 6)];
        assert_eq!(merge_ranges(&ranges), vec![(1, 2), (3, 4), (5, 6)]);
    }

    #[test]
    fn test_leaves_name_intact() {
        let entry = Entry {
            name: "test name with spaces".to_string(),
            value: None,
            name_match_ranges: None,
            value_match_ranges: None,
            icon: None,
            line_number: None,
            preview_type: PreviewType::Basic,
        };
        assert_eq!(entry.stdout_repr(), "test name with spaces");
    }
    #[test]
    fn test_uses_line_number_information() {
        let a: usize = 10;
        let entry = Entry {
            name: "test_file_name.rs".to_string(),
            value: None,
            name_match_ranges: None,
            value_match_ranges: None,
            icon: None,
            line_number: Some(a),
            preview_type: PreviewType::Basic,
        };
        assert_eq!(entry.stdout_repr(), "test_file_name.rs:10");
    }
}
