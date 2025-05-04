use std::hash::{Hash, Hasher};

use devicons::FileIcon;

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
/// Convert a list of indices into a list of ranges, merging contiguous ranges.
///
/// # Example
/// ```
/// use television::channels::entry::into_ranges;
/// let indices = vec![1, 2, 7, 8];
/// let ranges = into_ranges(&indices);
/// assert_eq!(ranges, vec![(1, 3), (7, 9)]);
/// ```
pub fn into_ranges(indices: &[u32]) -> Vec<(u32, u32)> {
    indices
        .iter()
        .fold(Vec::new(), |mut acc: Vec<(u32, u32)>, x| {
            if let Some(last) = acc.last_mut() {
                if last.1 == *x {
                    last.1 = *x + 1;
                } else {
                    acc.push((*x, *x + 1));
                }
            } else {
                acc.push((*x, *x + 1));
            }
            return acc;
        })
}

impl Entry {
    /// Create a new entry with the given name and preview type.
    ///
    /// Additional fields can be set using the builder pattern.
    /// ```
    /// use television::channels::entry::Entry;
    /// use devicons::FileIcon;
    ///
    /// let entry = Entry::new("name".to_string())
    ///                 .with_value("value".to_string())
    ///                 .with_name_match_indices(&vec![0])
    ///                 .with_value_match_indices(&vec![0])
    ///                 .with_icon(FileIcon::default())
    ///                 .with_line_number(0);
    /// ```
    ///
    /// # Arguments
    /// * `name` - The name of the entry.
    ///
    /// # Returns
    /// A new entry with the given name and preview type.
    /// The other fields are set to `None` by default.
    pub fn new(name: String) -> Self {
        Self {
            name,
            value: None,
            name_match_ranges: None,
            value_match_ranges: None,
            icon: None,
            line_number: None,
        }
    }

    pub fn with_value(mut self, value: String) -> Self {
        self.value = Some(value);
        self
    }

    pub fn with_name_match_indices(mut self, indices: &[u32]) -> Self {
        self.name_match_ranges = Some(into_ranges(indices));
        self
    }

    pub fn with_value_match_indices(mut self, indices: &[u32]) -> Self {
        self.value_match_ranges = Some(into_ranges(indices));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let ranges: Vec<u32> = vec![];
        assert_eq!(into_ranges(&ranges), Vec::<(u32, u32)>::new());
    }

    #[test]
    fn test_single_range() {
        let ranges = vec![1, 2];
        assert_eq!(into_ranges(&ranges), vec![(1, 3)]);
    }

    #[test]
    fn test_contiguous_ranges() {
        let ranges = vec![1, 2, 3, 4];
        assert_eq!(into_ranges(&ranges), vec![(1, 5)]);
    }

    #[test]
    fn test_non_contiguous_ranges() {
        let ranges = vec![1, 3, 5];
        assert_eq!(into_ranges(&ranges), vec![(1, 2), (3, 4), (5, 6)]);
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
        };
        assert_eq!(entry.stdout_repr(), "test_file_name.rs:10");
    }
}
