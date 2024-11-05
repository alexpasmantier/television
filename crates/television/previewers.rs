use std::sync::Arc;

use crate::entry::Entry;

mod basic;
mod cache;
mod directory;
mod env;
mod files;
mod meta;

// previewer types
pub use basic::BasicPreviewer;
pub use directory::DirectoryPreviewer;
pub use env::EnvVarPreviewer;
pub use files::FilePreviewer;
//use ratatui_image::protocol::StatefulProtocol;
use syntect::highlighting::Style;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum PreviewType {
    #[default]
    Basic,
    Directory,
    EnvVar,
    Files,
}

#[derive(Clone)]
pub enum PreviewContent {
    Empty,
    FileTooLarge,
    SyntectHighlightedText(Vec<Vec<(Style, String)>>),
    //Image(Box<dyn StatefulProtocol>),
    Loading,
    NotSupported,
    PlainText(Vec<String>),
    PlainTextWrapped(String),
}

pub const PREVIEW_NOT_SUPPORTED_MSG: &str =
    "Preview for this file type is not yet supported";
pub const FILE_TOO_LARGE_MSG: &str = "File too large";

/// A preview of an entry.
///
/// # Fields
/// - `title`: The title of the preview.
/// - `content`: The content of the preview.
#[derive(Clone)]
pub struct Preview {
    pub title: String,
    pub content: PreviewContent,
}

impl Default for Preview {
    fn default() -> Self {
        Preview {
            title: String::new(),
            content: PreviewContent::Empty,
        }
    }
}

impl Preview {
    pub fn new(title: String, content: PreviewContent) -> Self {
        Preview { title, content }
    }

    pub fn total_lines(&self) -> u16 {
        match &self.content {
            PreviewContent::SyntectHighlightedText(lines) => {
                lines.len().try_into().unwrap_or(u16::MAX)
            }
            PreviewContent::PlainText(lines) => {
                lines.len().try_into().unwrap_or(u16::MAX)
            }
            _ => 0,
        }
    }
}

pub struct Previewer {
    basic: BasicPreviewer,
    directory: DirectoryPreviewer,
    file: FilePreviewer,
    env_var: EnvVarPreviewer,
}

impl Default for Previewer {
    fn default() -> Self {
        Previewer::new()
    }
}

impl Previewer {
    pub fn new() -> Self {
        Previewer {
            basic: BasicPreviewer::new(),
            directory: DirectoryPreviewer::new(),
            file: FilePreviewer::new(),
            env_var: EnvVarPreviewer::new(),
        }
    }

    pub async fn preview(&mut self, entry: &Entry) -> Arc<Preview> {
        match entry.preview_type {
            PreviewType::Basic => self.basic.preview(entry),
            PreviewType::Directory => self.directory.preview(entry).await,
            PreviewType::EnvVar => self.env_var.preview(entry),
            PreviewType::Files => self.file.preview(entry).await,
        }
    }
}
