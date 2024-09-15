use super::display::StyledString;

mod env;

// entry types
pub use env::EnvVarEntry;

// finder types
pub use env::EnvVarFinder;

/// A trait for an entry that can be displayed.
///
/// # Methods
/// - `display`: Display the entry.
pub trait Entry {
    fn display(&self) -> Vec<StyledString>;
}

/// A trait for a finder that can find entries based on a pattern.
/// The associated type `I` is an iterator that iterates over the entries found.
/// The associated type `R` is the type of the entry that the finder finds.
/// The `find` method takes a pattern and returns an iterator over the entries found.
///
/// # Type Parameters
/// - `R`: The type of the entry that the finder finds.
/// - `I`: The type of the iterator that iterates over the entries found.
///
/// # Methods
/// - `find`: Find entries based on a pattern.
pub trait Finder<R>
where
    R: Entry,
{
    type I: Iterator<Item = R>; // Associated type for the iterator

    fn find(&self, pattern: &str) -> Self::I;
}
