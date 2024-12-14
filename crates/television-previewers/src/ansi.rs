#![allow(unused_imports)]
//! This module provides a way to parse ansi escape codes and convert them to ratatui objects.
//!
//! This code is a modified version of [ansi_to_tui](https://github.com/ratatui/ansi-to-tui).

// mod ansi;
pub mod code;
pub mod error;
pub mod parser;
pub use error::Error;
use tui::text::Text;

/// IntoText will convert any type that has a AsRef<[u8]> to a Text.
pub trait IntoText {
    /// Convert the type to a Text.
    #[allow(clippy::wrong_self_convention)]
    fn into_text(&self) -> Result<Text<'static>, Error>;
    /// Convert the type to a Text while trying to copy as less as possible
    #[cfg(feature = "zero-copy")]
    fn to_text(&self) -> Result<Text<'_>, Error>;
}
impl<T> IntoText for T
where
    T: AsRef<[u8]>,
{
    fn into_text(&self) -> Result<Text<'static>, Error> {
        Ok(crate::ansi::parser::text(self.as_ref())?.1)
    }

    #[cfg(feature = "zero-copy")]
    fn to_text(&self) -> Result<Text<'_>, Error> {
        Ok(crate::ansi::parser::text_fast(self.as_ref())?.1)
    }
}
