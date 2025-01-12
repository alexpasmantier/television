use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use cache::PreviewCache;
use devicons::FileIcon;
use parking_lot::Mutex;
use rustc_hash::FxHashMap;
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
use tracing::debug;

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
        stale: bool,
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
    requests: FxHashMap<Entry, Instant>,
    cache: Arc<Mutex<PreviewCache>>,
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

const DEBOUNCE_DURATION: Duration = Duration::from_millis(20);
const REQUEST_TIMEOUT: Duration = Duration::from_millis(200);

impl Previewer {
    pub fn new(config: Option<PreviewerConfig>) -> Self {
        let config = config.unwrap_or_default();
        Previewer {
            basic: BasicPreviewer::new(Some(config.basic)),
            file: FilePreviewer::new(Some(config.file)),
            env_var: EnvVarPreviewer::new(Some(config.env_var)),
            command: CommandPreviewer::new(Some(config.command)),
            requests: FxHashMap::default(),
            cache: Arc::new(Mutex::new(PreviewCache::default())),
        }
    }

    pub fn preview(&mut self, entry: &Entry) -> Option<Arc<Preview>> {
        // remove any requests that have timed out
        self.requests
            .retain(|e, v| v.elapsed() < REQUEST_TIMEOUT || e == entry);

        // if we have a preview in cache, return it
        if let Some(preview) = self.cache.lock().get(&entry.name) {
            debug!("Preview already in cache");
            return Some(preview);
        }
        // if we haven't acknowledged the request yet, acknowledge it
        if !self.requests.contains_key(entry) {
            self.requests.insert(entry.clone(), Instant::now());
        }

        let initial_request = self.requests.get(entry).unwrap();
        // if we're past the debounce duration
        if initial_request.elapsed() > DEBOUNCE_DURATION {
            debug!("Past debounce duration");
            // forward the request to the appropriate previewer
            let preview = match &entry.preview_type {
                PreviewType::Basic => Some(self.basic.preview(entry)),
                PreviewType::EnvVar => Some(self.env_var.preview(entry)),
                PreviewType::Files => self.file.preview(entry),
                PreviewType::Command(cmd) => self.command.preview(entry, cmd),
                PreviewType::None => Some(Arc::new(Preview::default())),
            };
            // if we got a preview, cache it
            if let Some(preview) = preview {
                self.cache.lock().insert(entry.name.clone(), &preview);
                Some(preview)
            } else {
                None
            }
        } else {
            debug!("Not past debounce duration");
            // partial preview
            let preview = match &entry.preview_type {
                PreviewType::Basic => Some(self.basic.preview(entry)),
                PreviewType::EnvVar => Some(self.env_var.preview(entry)),
                PreviewType::Files => self.file.preview(entry),
                PreviewType::Command(cmd) => {
                    self.command.partial_preview(entry, cmd)
                }
                PreviewType::None => Some(Arc::new(Preview::default())),
            };
            Some(preview)
        }
    }

    pub fn set_config(&mut self, config: PreviewerConfig) {
        self.basic = BasicPreviewer::new(Some(config.basic));
        self.file = FilePreviewer::new(Some(config.file));
        self.env_var = EnvVarPreviewer::new(Some(config.env_var));
    }
}
