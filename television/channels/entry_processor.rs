use crate::{
    channels::{entry::Entry, prototypes::Template},
    matcher::{injector::Injector, matched_item::MatchedItem},
};
use fast_strip_ansi::strip_ansi_string;

pub trait EntryProcessor: Send + Sync + Clone + 'static {
    type Data: Send + Sync + Clone + 'static;

    fn push_to_injector(&self, line: String, injector: &Injector<Self::Data>);

    fn make_entry(
        &self,
        item: MatchedItem<Self::Data>,
        source_output: Option<&Template>,
    ) -> Entry;

    fn has_ansi(&self) -> bool;
}

/// no transformation: matches the raw line as-is
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
        source_output: Option<&Template>,
    ) -> Entry {
        let mut entry = Entry::new(item.matched_string.clone())
            .with_match_indices(&item.match_indices);
        if let Some(output) = source_output {
            entry = entry.with_output(output.clone());
        }
        entry
    }

    fn has_ansi(&self) -> bool {
        false
    }
}

/// ANSI mode: strips ANSI codes for matching
/// Uses Matcher<String> to store original with ANSI codes
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
}

/// Display mode: applies custom template for matching
/// Uses Matcher<String> to store original
#[derive(Clone, Debug)]
pub struct DisplayProcessor {
    pub template: Template,
}

impl EntryProcessor for DisplayProcessor {
    type Data = String;

    fn push_to_injector(&self, line: String, injector: &Injector<String>) {
        let template = self.template.clone();
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
}
