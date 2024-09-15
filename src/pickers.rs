use super::finders;
use super::previewers;
use anyhow::Result;
mod env;

/// A trait for a picker that can load entries, select an entry, get the selected entry, get all
/// entries, clear the entries, and get a preview of the selected entry.
///
/// # Type Parameters
/// - `F`: The type of the finder used to find entries.
/// - `R`: The type of the entry.
/// - `P`: The type of the previewer used to preview entries.
///
/// # Methods
/// - `load_entries`: Load entries based on a pattern.
/// - `select_entry`: Select an entry based on an index.
/// - `selected_entry`: Get the selected entry.
/// - `entries`: Get all entries.
/// - `clear`: Clear all entries.
/// - `get_preview`: Get a preview of the currently selected entry.
pub trait Picker<F, R, P>
where
    F: finders::Finder<R>,
    R: finders::Entry,
    P: previewers::Previewer<R>,
{
    /// Load entries based on a pattern.
    /// TODO: implement some caching mechanism to avoid loading entries every time.
    fn load_entries(&mut self, pattern: &str) -> Result<()>;
    /// Select an entry based on an index.
    fn select_entry(&mut self, index: usize);
    /// Get the selected entry.
    fn selected_entry(&self) -> Option<R>;
    /// Get all entries.
    fn entries(&self) -> &Vec<R>;
    /// Clear all entries.
    fn clear(&mut self);
    /// Get a preview of the currently selected entry.
    fn get_preview(&self) -> Option<previewers::Preview>;
}
