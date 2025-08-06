use crate::{
    channels::prototypes::Template, event::Key,
    screen::result_item::ResultItem,
};
use anyhow::Result;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Eq)]
pub struct Entry {
    /// The raw entry (as captured from the source)
    pub raw: String,
    /// The actual entry string that will be displayed in the UI.
    pub display: Option<String>,
    /// The output string that will be used when the entry is selected.
    pub output: Option<Template>,
    /// The optional ranges for matching characters (based on `self.display`).
    pub match_ranges: Option<Vec<(u32, u32)>>,
    /// Whether the entry contains ANSI escape sequences.
    pub ansi: bool,
}

impl Hash for Entry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl PartialEq<Entry> for &Entry {
    fn eq(&self, other: &Entry) -> bool {
        self.raw == other.raw
    }
}

impl PartialEq<Entry> for Entry {
    fn eq(&self, other: &Entry) -> bool {
        self.raw == other.raw
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
    ///                 .with_display("Display Name".to_string())
    ///                 .with_match_indices(&vec![0]);
    /// ```
    ///
    /// # Arguments
    /// * `name` - The name of the entry.
    ///
    /// # Returns
    /// A new entry with the given name and preview type.
    /// The other fields are set to `None` by default.
    pub fn new(raw: String) -> Self {
        Self {
            raw,
            display: None,
            output: None,
            match_ranges: None,
            ansi: false,
        }
    }

    pub fn with_display(mut self, display: String) -> Self {
        self.display = Some(display);
        self
    }

    pub fn with_output(mut self, output: Template) -> Self {
        self.output = Some(output);
        self
    }

    pub fn with_match_indices(mut self, indices: &[u32]) -> Self {
        self.match_ranges = Some(into_ranges(indices));
        self
    }

    pub fn display(&self) -> &str {
        self.display.as_deref().unwrap_or(&self.raw)
    }

    pub fn output(&self) -> Result<String> {
        if let Some(output) = &self.output {
            output.format(&self.raw)
        } else {
            Ok(self.raw.clone())
        }
    }

    /// Sets whether the entry contains ANSI escape sequences.
    pub fn ansi(mut self, ansi: bool) -> Self {
        self.ansi = ansi;
        self
    }
}

impl ResultItem for Entry {
    fn raw(&self) -> &str {
        &self.raw
    }

    fn display(&self) -> &str {
        self.display()
    }

    fn output(&self) -> Result<String> {
        self.output()
    }

    fn match_ranges(&self) -> Option<&[(u32, u32)]> {
        self.match_ranges.as_deref()
    }

    fn shortcut(&self) -> Option<&Key> {
        None
    }

    fn ansi(&self) -> bool {
        self.ansi
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
            raw: "test name with spaces".to_string(),
            display: None,
            output: None,
            match_ranges: None,
            ansi: false,
        };
        assert_eq!(entry.output().unwrap(), "test name with spaces");
    }
}
