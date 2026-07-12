use crate::{
    channels::{entry::Entry, prototypes::Template},
    matcher::matched_item::MatchedItem,
};
use fast_strip_ansi::strip_ansi_string;
use std::borrow::Cow;

/// Implementors of this trait define two things:
/// - how to process raw lines into matcher entries, including any preprocessing steps (e.g.
///   stripping ANSI codes, applying templates, etc.)
/// - how to construct the final Entry objects used by the rest of the application from the matched
///   items
///
/// The associated type `Data` defines the type of data stored in the matcher for each line.
pub trait EntryProcessor: Send + Sync + Clone + 'static {
    type Data: Send + Sync + Clone + 'static;

    /// Process a raw line into the data stored in the matcher and the haystack
    /// string the line will be matched against.
    fn process(&self, line: String) -> (Self::Data, String);

    fn make_entry(
        &self,
        item: MatchedItem<Self::Data>,
        source_output: Option<&Template>,
    ) -> Entry;

    fn has_ansi(&self) -> bool;

    /// Extract the frecency key for lookup.
    ///
    /// This should return the same value that becomes `Entry.raw` so that
    /// frecency lookups match correctly.
    ///
    /// Returns `Cow<str>` to avoid allocations when possible (e.g., when the data is
    /// already a String).
    fn frecency_key<'a>(
        data: &'a Self::Data,
        haystack: &'a str,
    ) -> Cow<'a, str>;
}

/// A processor that does no special processing: matches the raw lines as-is.
///
/// Uses `Matcher<()>` since no extra data is needed which reduces memory usage.
#[derive(Clone, Debug)]
pub struct PlainProcessor;

impl EntryProcessor for PlainProcessor {
    type Data = ();

    fn process(&self, line: String) -> ((), String) {
        ((), line)
    }

    fn make_entry(
        &self,
        item: MatchedItem<()>,
        source_output: Option<&Template>,
    ) -> Entry {
        let mut entry = Entry::new(item.matched_string)
            .with_match_indices(&item.match_indices);
        if let Some(output) = source_output {
            entry = entry.with_output(output.clone());
        }
        entry
    }

    fn has_ansi(&self) -> bool {
        false
    }

    fn frecency_key<'a>(
        (): &'a Self::Data,
        haystack: &'a str,
    ) -> Cow<'a, str> {
        Cow::Borrowed(haystack)
    }
}

/// A processor that preserves ANSI codes in the matched lines by storing two versions of each
/// line in the matcher:
///
/// - the original line with ANSI codes (the matcher data)
/// - a stripped version without ANSI codes for matching (the haystack)
///
/// Uses `Matcher<String>` to store original with ANSI codes.
#[derive(Clone, Debug)]
pub struct AnsiProcessor;

impl EntryProcessor for AnsiProcessor {
    type Data = String;

    fn process(&self, line: String) -> (String, String) {
        let stripped = strip_ansi_string(&line).into_owned();
        (line, stripped)
    }

    fn make_entry(
        &self,
        item: MatchedItem<String>,
        source_output: Option<&Template>,
    ) -> Entry {
        let mut entry = Entry::new(item.inner)
            .with_display(item.matched_string)
            .with_match_indices(&item.match_indices)
            .ansi(true);
        if let Some(output) = source_output {
            entry = entry.with_output(output.clone());
        }
        entry
    }

    fn has_ansi(&self) -> bool {
        true
    }

    fn frecency_key<'a>(
        data: &'a Self::Data,
        _haystack: &'a str,
    ) -> Cow<'a, str> {
        // data is the original String, borrow it directly without allocation
        Cow::Borrowed(data.as_str())
    }
}

/// A processor that applies a display template to each line before matching, also storing the
/// original line into the matcher for further uses (e.g. output templates).
///
/// Uses `Matcher<String>` to store original lines.
#[derive(Clone, Debug)]
pub struct DisplayProcessor {
    pub template: Template,
}

impl EntryProcessor for DisplayProcessor {
    type Data = String;

    fn process(&self, line: String) -> (String, String) {
        let display = self.template.format(&line).unwrap_or_else(|_| {
            panic!(
                "Failed to format display expression '{}' with entry '{}'",
                self.template.raw(),
                line
            )
        });
        (line, display)
    }

    fn make_entry(
        &self,
        item: MatchedItem<String>,
        source_output: Option<&Template>,
    ) -> Entry {
        let mut entry = Entry::new(item.inner)
            .with_display(item.matched_string)
            .with_match_indices(&item.match_indices)
            .ansi(false);
        if let Some(output) = source_output {
            entry = entry.with_output(output.clone());
        }
        entry
    }

    fn has_ansi(&self) -> bool {
        false
    }

    fn frecency_key<'a>(
        data: &'a Self::Data,
        _haystack: &'a str,
    ) -> Cow<'a, str> {
        // data is the original String, borrow it directly without allocation
        Cow::Borrowed(data.as_str())
    }
}
