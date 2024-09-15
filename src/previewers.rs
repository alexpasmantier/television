use super::display::StyledString;

mod env;

// previewer types
pub use env::EnvVarPreviewer;

/// A preview of an entry.
///
/// # Fields
/// - `title`: The title of the preview.
/// - `content`: The content of the preview.
pub struct Preview {
    pub title: String,
    pub content: Vec<StyledString>,
}

/// A trait for a previewer that can preview entries.
///
/// # Type Parameters
/// - `E`: The type of the entry to preview.
/// - `P`: The type of the preview to produce.
///
/// # Methods
/// - `preview`: Preview an entry and produce a preview.
pub trait Previewer<E>
where
    E: super::finders::Entry,
{
    fn preview(&self, e: E) -> Preview;
}
