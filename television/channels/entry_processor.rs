use crate::{
    channels::{entry::Entry, prototypes::Template},
    matcher::{injector::Injector, matched_item::MatchedItem},
};
use fast_strip_ansi::strip_ansi_string;
use nucleo::Utf32Str;
use std::borrow::Cow;
use std::sync::Arc;

/// Implementors of this trait define two things:
/// - how to push lines into the matcher, including any preprocessing steps (e.g. stripping ANSI
///   codes, applying templates, etc.)
/// - how to construct the final Entry objects used by the rest of the application from the matched
///   items
///
/// The associated type `Data` defines the type of data stored in the matcher for each line.
pub trait EntryProcessor: Send + Sync + Clone + 'static {
    type Data: Send + Sync + Clone + 'static;

    fn push_to_injector(&self, line: String, injector: &Injector<Self::Data>);

    fn make_entry(
        &self,
        item: MatchedItem<Self::Data>,
        source_output: Option<&Arc<Template>>,
    ) -> Entry;

    fn has_ansi(&self) -> bool;

    /// Extract the frecency key from a matched item for lookup.
    ///
    /// This should return the same value that becomes `Entry.raw` so that
    /// frecency lookups match correctly.
    ///
    /// Returns `Cow<str>` to avoid allocations when possible (e.g., for ASCII text
    /// or when the data is already a String).
    fn frecency_key<'a>(item: &nucleo::Item<'a, Self::Data>) -> Cow<'a, str>;
}

/// A processor that does no special processing: matches the raw lines as-is and stores
/// them directly into the first matcher column.
///
/// Uses `Matcher<()>` since no extra data is needed which reduces memory usage.
#[derive(Clone, Debug)]
pub struct PlainProcessor;

impl EntryProcessor for PlainProcessor {
    type Data = ();

    fn push_to_injector(&self, line: String, injector: &Injector<()>) {
        injector.push((), |(), cols| {
            cols[0] = line.into();
        });
    }

    fn make_entry(
        &self,
        item: MatchedItem<()>,
        source_output: Option<&Arc<Template>>,
    ) -> Entry {
        let mut entry = Entry::new(item.matched_string)
            .with_match_indices(&item.match_indices);
        if let Some(output) = source_output {
            entry = entry.with_output(Arc::clone(output));
        }
        entry
    }

    fn has_ansi(&self) -> bool {
        false
    }

    fn frecency_key<'a>(item: &nucleo::Item<'a, Self::Data>) -> Cow<'a, str> {
        // Use slice(..) to get Utf32Str from Utf32String, then match on it
        match item.matcher_columns[0].slice(..) {
            // For ASCII, we can borrow directly without allocation.
            // Safety: Utf32Str::Ascii only contains valid ASCII bytes which are valid UTF-8.
            Utf32Str::Ascii(bytes) => {
                Cow::Borrowed(unsafe { std::str::from_utf8_unchecked(bytes) })
            }
            // For Unicode, we must allocate to convert char slice to String.
            Utf32Str::Unicode(_) => {
                Cow::Owned(item.matcher_columns[0].to_string())
            }
        }
    }
}

/// A processor that preserves ANSI codes in the matched lines by storing two versions of each
/// line in the matcher:
///
/// - the original line with ANSI codes
/// - a stripped version without ANSI codes for matching (matcher column 0)
///
/// Uses `Matcher<String>` to store original with ANSI codes.
#[derive(Clone, Debug)]
pub struct AnsiProcessor;

impl EntryProcessor for AnsiProcessor {
    type Data = String;

    fn push_to_injector(&self, line: String, injector: &Injector<String>) {
        injector.push(line, |original, cols| {
            cols[0] = strip_ansi_string(original).into();
        });
    }

    fn make_entry(
        &self,
        item: MatchedItem<String>,
        source_output: Option<&Arc<Template>>,
    ) -> Entry {
        let mut entry = Entry::new(item.inner)
            .with_display(item.matched_string)
            .with_match_indices(&item.match_indices)
            .ansi(true);
        if let Some(output) = source_output {
            entry = entry.with_output(Arc::clone(output));
        }
        entry
    }

    fn has_ansi(&self) -> bool {
        true
    }

    fn frecency_key<'a>(item: &nucleo::Item<'a, Self::Data>) -> Cow<'a, str> {
        // item.data is &String, borrow it directly without allocation
        Cow::Borrowed(item.data.as_str())
    }
}

/// A processor that applies a display template to each line before matching, also storing the
/// original line into the matcher for further uses (e.g. output templates).
///
/// Uses `Matcher<String>` to store original lines.
#[derive(Clone, Debug)]
pub struct DisplayProcessor {
    pub template: Arc<Template>,
}

impl EntryProcessor for DisplayProcessor {
    type Data = String;

    fn push_to_injector(&self, line: String, injector: &Injector<String>) {
        let template = Arc::clone(&self.template);
        injector.push(line, move |original, cols| {
            cols[0] = template.format(original)
                .unwrap_or_else(|_| {
                    panic!(
                        "Failed to format display expression '{}' with entry '{}'",
                        template.raw(),
                        original
                    )
                })
                .into();
        });
    }

    fn make_entry(
        &self,
        item: MatchedItem<String>,
        source_output: Option<&Arc<Template>>,
    ) -> Entry {
        let mut entry = Entry::new(item.inner)
            .with_display(item.matched_string)
            .with_match_indices(&item.match_indices)
            .ansi(false);
        if let Some(output) = source_output {
            entry = entry.with_output(Arc::clone(output));
        }
        entry
    }

    fn has_ansi(&self) -> bool {
        false
    }

    fn frecency_key<'a>(item: &nucleo::Item<'a, Self::Data>) -> Cow<'a, str> {
        // item.data is &String, borrow it directly without allocation
        Cow::Borrowed(item.data.as_str())
    }
}
