/// This module contains abstract display logic for the application.

/// A group of styles that can be applied to a string.
pub enum StyleGroup {
    Default,
    Entry,
    EntryLineNumber,
    EntryContent,
}

/// A styled string with a text and a style group for that string.
/// This is used to produce styled output in a way that can then be translated to the actual
/// frontend implementation.
pub struct StyledString {
    pub text: String,
    pub style: StyleGroup,
}
