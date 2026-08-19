use crate::{
    channels::{entry::Entry, prototypes::Template},
    matcher::matched_item::MatchedItem,
    utils::ansi::{AnsiParser, SharedPalette, StyleRuns},
};
use std::{borrow::Cow, sync::Arc};

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
    ///
    /// Takes `&mut self` so processors can keep per-worker state across the
    /// lines of a batch (see [`AnsiProcessor`]).
    fn process(&mut self, line: String) -> (Self::Data, String);

    fn make_entry(
        &self,
        item: MatchedItem<Self::Data>,
        source_output: Option<&Template>,
    ) -> Entry;

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

    fn process(&mut self, line: String) -> ((), String) {
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

    fn frecency_key<'a>(
        (): &'a Self::Data,
        haystack: &'a str,
    ) -> Cow<'a, str> {
        Cow::Borrowed(haystack)
    }
}

/// A processor for sources that emit ANSI escape codes.
///
/// The line is parsed once here: the matcher stores the stripped text and the
/// styling is reduced to a few interned runs (see [`crate::utils::ansi`]).
/// Keeping the styling instead of the original line costs a fraction of the
/// memory, and results render from resolved runs rather than re-parsing
/// escape codes on every frame.
#[derive(Clone, Debug)]
pub struct AnsiProcessor {
    parser: AnsiParser,
    palette: SharedPalette,
}

impl AnsiProcessor {
    pub fn new() -> Self {
        let palette = SharedPalette::default();
        Self {
            parser: AnsiParser::new(Arc::clone(&palette)),
            palette,
        }
    }
}

impl Default for AnsiProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl EntryProcessor for AnsiProcessor {
    type Data = StyleRuns;

    fn process(&mut self, line: String) -> (StyleRuns, String) {
        let (stripped, runs) = self.parser.parse(&line);
        (runs, stripped)
    }

    fn make_entry(
        &self,
        item: MatchedItem<StyleRuns>,
        source_output: Option<&Template>,
    ) -> Entry {
        let mut entry = Entry::new(item.matched_string)
            .with_match_indices(&item.match_indices);
        if !item.inner.is_empty() {
            // Resolving the palette here (rather than storing styles per
            // entry) keeps it to the handful of rows actually on screen
            let palette = self.palette.read();
            entry = entry.with_styles(
                item.inner
                    .iter()
                    .map(|&(at, id)| (at, palette.resolve(id)))
                    .collect(),
            );
        }
        if let Some(output) = source_output {
            entry = entry.with_output(output.clone());
        }
        entry
    }

    fn frecency_key<'a>(
        _data: &'a Self::Data,
        haystack: &'a str,
    ) -> Cow<'a, str> {
        // the stripped line is what `Entry.raw` ends up being
        Cow::Borrowed(haystack)
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

    fn process(&mut self, line: String) -> (String, String) {
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
            .with_match_indices(&item.match_indices);
        if let Some(output) = source_output {
            entry = entry.with_output(output.clone());
        }
        entry
    }

    fn frecency_key<'a>(
        data: &'a Self::Data,
        _haystack: &'a str,
    ) -> Cow<'a, str> {
        // data is the original String, borrow it directly without allocation
        Cow::Borrowed(data.as_str())
    }
}
