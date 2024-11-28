use std::sync::Arc;

use devicons::FileIcon;
use television_channels::entry::{Entry, PreviewType};

pub mod basic;
pub mod cache;
pub mod command;
pub mod env;
pub mod files;
pub mod meta;

// previewer types
pub use basic::BasicPreviewer;
pub use basic::BasicPreviewerConfig;
pub use command::CommandPreviewer;
pub use command::CommandPreviewerConfig;
pub use env::EnvVarPreviewer;
pub use env::EnvVarPreviewerConfig;
pub use files::FilePreviewer;
pub use files::FilePreviewerConfig;
use syntect::highlighting::Style;

#[derive(Clone, Debug)]
pub enum PreviewContent {
    Empty,
    FileTooLarge,
    SyntectHighlightedText(Vec<Vec<(Style, String)>>),
    Loading,
    NotSupported,
    PlainText(Vec<String>),
    PlainTextWrapped(String),
    AnsiText(String),
}

pub const PREVIEW_NOT_SUPPORTED_MSG: &str =
    "Preview for this file type is not supported";
pub const FILE_TOO_LARGE_MSG: &str = "File too large";

/// A preview of an entry.
///
/// # Fields
/// - `title`: The title of the preview.
/// - `content`: The content of the preview.
#[derive(Clone, Debug)]
pub struct Preview {
    pub title: String,
    pub content: PreviewContent,
    pub icon: Option<FileIcon>,
}

impl Default for Preview {
    fn default() -> Self {
        Preview {
            title: String::new(),
            content: PreviewContent::Empty,
            icon: None,
        }
    }
}

impl Preview {
    pub fn new(
        title: String,
        content: PreviewContent,
        icon: Option<FileIcon>,
    ) -> Self {
        Preview {
            title,
            content,
            icon,
        }
    }

    pub fn total_lines(&self) -> u16 {
        match &self.content {
            PreviewContent::SyntectHighlightedText(lines) => {
                lines.len().try_into().unwrap_or(u16::MAX)
            }
            PreviewContent::PlainText(lines) => {
                lines.len().try_into().unwrap_or(u16::MAX)
            }
            PreviewContent::AnsiText(text) => {
                text.lines().count().try_into().unwrap_or(u16::MAX)
            }
            _ => 0,
        }
    }
}

#[derive(Debug, Default)]
pub struct Previewer {
    basic: BasicPreviewer,
    file: FilePreviewer,
    env_var: EnvVarPreviewer,
    command: CommandPreviewer,
}

#[derive(Debug, Default)]
pub struct PreviewerConfig {
    basic: BasicPreviewerConfig,
    file: FilePreviewerConfig,
    env_var: EnvVarPreviewerConfig,
    command: CommandPreviewerConfig,
}

impl PreviewerConfig {
    pub fn basic(mut self, config: BasicPreviewerConfig) -> Self {
        self.basic = config;
        self
    }

    pub fn file(mut self, config: FilePreviewerConfig) -> Self {
        self.file = config;
        self
    }

    pub fn env_var(mut self, config: EnvVarPreviewerConfig) -> Self {
        self.env_var = config;
        self
    }
}

impl Previewer {
    pub fn new(config: Option<PreviewerConfig>) -> Self {
        let config = config.unwrap_or_default();
        Previewer {
            basic: BasicPreviewer::new(Some(config.basic)),
            file: FilePreviewer::new(Some(config.file)),
            env_var: EnvVarPreviewer::new(Some(config.env_var)),
            command: CommandPreviewer::new(Some(config.command)),
        }
    }

    pub fn preview(&mut self, entry: &Entry) -> Arc<Preview> {
        match &entry.preview_type {
            PreviewType::Basic => self.basic.preview(entry),
            PreviewType::EnvVar => self.env_var.preview(entry),
            PreviewType::Files => self.file.preview(entry),
            PreviewType::Command(cmd) => self.command.preview(entry, cmd),
        }
    }

    pub fn set_config(&mut self, config: PreviewerConfig) {
        self.basic = BasicPreviewer::new(Some(config.basic));
        self.file = FilePreviewer::new(Some(config.file));
        self.env_var = EnvVarPreviewer::new(Some(config.env_var));
    }
}
