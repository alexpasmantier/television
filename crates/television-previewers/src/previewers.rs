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
use television_utils::cache::RingSet;
use television_utils::syntax::HighlightedLines;

#[derive(Clone, Debug)]
pub enum PreviewContent {
    Empty,
    FileTooLarge,
    SyntectHighlightedText(HighlightedLines),
    Loading,
    Timeout,
    NotSupported,
    PlainText(Vec<String>),
    PlainTextWrapped(String),
    AnsiText(String),
}

impl PreviewContent {
    pub fn total_lines(&self) -> u16 {
        match self {
            PreviewContent::SyntectHighlightedText(hl_lines) => {
                hl_lines.lines.len().try_into().unwrap_or(u16::MAX)
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

pub const PREVIEW_NOT_SUPPORTED_MSG: &str =
    "Preview for this file type is not supported";
pub const FILE_TOO_LARGE_MSG: &str = "File too large";
pub const LOADING_MSG: &str = "Loading...";
pub const TIMEOUT_MSG: &str = "Preview timed out";

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
    /// If the preview is partial, this field contains the byte offset
    /// up to which the preview holds.
    pub partial_offset: Option<usize>,
    pub total_lines: u16,
}

impl Default for Preview {
    fn default() -> Self {
        Preview {
            title: String::new(),
            content: PreviewContent::Empty,
            icon: None,
            partial_offset: None,
            total_lines: 0,
        }
    }
}

impl Preview {
    pub fn new(
        title: String,
        content: PreviewContent,
        icon: Option<FileIcon>,
        partial_offset: Option<usize>,
        total_lines: u16,
    ) -> Self {
        Preview {
            title,
            content,
            icon,
            partial_offset,
            total_lines,
        }
    }

    pub fn total_lines(&self) -> u16 {
        match &self.content {
            PreviewContent::SyntectHighlightedText(hl_lines) => {
                hl_lines.lines.len().try_into().unwrap_or(u16::MAX)
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
    requests: RingSet<Entry>,
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

const REQUEST_STACK_SIZE: usize = 20;

impl Previewer {
    pub fn new(config: Option<PreviewerConfig>) -> Self {
        let config = config.unwrap_or_default();
        Previewer {
            basic: BasicPreviewer::new(Some(config.basic)),
            file: FilePreviewer::new(Some(config.file)),
            env_var: EnvVarPreviewer::new(Some(config.env_var)),
            command: CommandPreviewer::new(Some(config.command)),
            requests: RingSet::with_capacity(REQUEST_STACK_SIZE),
        }
    }

    fn dispatch_request(&mut self, entry: &Entry) -> Option<Arc<Preview>> {
        match &entry.preview_type {
            PreviewType::Basic => Some(self.basic.preview(entry)),
            PreviewType::EnvVar => Some(self.env_var.preview(entry)),
            PreviewType::Files => self.file.preview(entry),
            PreviewType::Command(cmd) => self.command.preview(entry, cmd),
            PreviewType::None => Some(Arc::new(Preview::default())),
        }
    }

    fn cached(&self, entry: &Entry) -> Option<Arc<Preview>> {
        match &entry.preview_type {
            PreviewType::Files => self.file.cached(entry),
            PreviewType::Command(_) => self.command.cached(entry),
            PreviewType::Basic | PreviewType::EnvVar => None,
            PreviewType::None => Some(Arc::new(Preview::default())),
        }
    }

    pub fn preview(&mut self, entry: &Entry) -> Option<Arc<Preview>> {
        // if we haven't acknowledged the request yet, acknowledge it
        self.requests.push(entry.clone());

        if let Some(preview) = self.dispatch_request(entry) {
            return Some(preview);
        }
        // lookup request stack and return the most recent preview available
        for request in self.requests.back_to_front() {
            if let Some(preview) = self.cached(&request) {
                return Some(preview);
            }
        }
        None
    }

    pub fn set_config(&mut self, config: PreviewerConfig) {
        self.basic = BasicPreviewer::new(Some(config.basic));
        self.file = FilePreviewer::new(Some(config.file));
        self.env_var = EnvVarPreviewer::new(Some(config.env_var));
    }
}
