use std::sync::Arc;

use crate::channels::entry::{Entry, PreviewType};
use devicons::FileIcon;
use ratatui::layout::Rect;

pub mod ansi;
pub mod cache;
pub mod previewers;

// previewer types
use crate::utils::cache::RingSet;
use crate::utils::image::ImagePreviewWidget;
use crate::utils::syntax::HighlightedLines;
pub use previewers::basic::BasicPreviewer;
pub use previewers::basic::BasicPreviewerConfig;
pub use previewers::command::CommandPreviewer;
pub use previewers::command::CommandPreviewerConfig;
pub use previewers::env::EnvVarPreviewer;
pub use previewers::env::EnvVarPreviewerConfig;
pub use previewers::files::FilePreviewer;
pub use previewers::files::FilePreviewerConfig;

#[derive(Clone, Debug, PartialEq, Hash)]
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
    Image(ImagePreviewWidget),
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
            PreviewContent::Image(image) => {
                image.height().try_into().unwrap_or(u16::MAX)
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
#[derive(Clone, Debug, PartialEq, Hash)]
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
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct PreviewState {
    pub enabled: bool,
    pub preview: Arc<Preview>,
    pub scroll: u16,
    pub target_line: Option<u16>,
}

impl Default for PreviewState {
    fn default() -> Self {
        PreviewState {
            enabled: false,
            preview: Arc::new(Preview::default()),
            scroll: 0,
            target_line: None,
        }
    }
}

const PREVIEW_MIN_SCROLL_LINES: u16 = 3;

impl PreviewState {
    pub fn new(
        enabled: bool,
        preview: Arc<Preview>,
        scroll: u16,
        target_line: Option<u16>,
    ) -> Self {
        PreviewState {
            enabled,
            preview,
            scroll,
            target_line,
        }
    }

    pub fn scroll_down(&mut self, offset: u16) {
        self.scroll = self.scroll.saturating_add(offset).min(
            self.preview
                .total_lines
                .saturating_sub(PREVIEW_MIN_SCROLL_LINES),
        );
    }

    pub fn scroll_up(&mut self, offset: u16) {
        self.scroll = self.scroll.saturating_sub(offset);
    }

    pub fn reset(&mut self) {
        self.preview = Arc::new(Preview::default());
        self.scroll = 0;
        self.target_line = None;
    }

    pub fn update(
        &mut self,
        preview: Arc<Preview>,
        scroll: u16,
        target_line: Option<u16>,
    ) {
        if self.preview.title != preview.title {
            self.preview = preview;
            self.scroll = scroll;
            self.target_line = target_line;
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

const REQUEST_STACK_SIZE: usize = 10;

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

    fn dispatch_request(
        &mut self,
        entry: &Entry,
        preview_window: Option<Rect>,
    ) -> Option<Arc<Preview>> {
        match &entry.preview_type {
            PreviewType::Basic => Some(self.basic.preview(entry)),
            PreviewType::EnvVar => Some(self.env_var.preview(entry)),
            PreviewType::Files => self.file.preview(entry, preview_window),
            PreviewType::Command(cmd) => self.command.preview(entry, cmd),
            PreviewType::None => Some(Arc::new(Preview::default())),
        }
    }

    fn cached(&self, entry: &Entry) -> Option<Arc<Preview>> {
        match &entry.preview_type {
            PreviewType::Basic => Some(self.basic.preview(entry)),
            PreviewType::EnvVar => Some(self.env_var.preview(entry)),
            PreviewType::Files => self.file.cached(entry),
            PreviewType::Command(_) => self.command.cached(entry),
            PreviewType::None => None,
        }
    }

    // we could use a target scroll here to make the previewer
    // faster, but since it's already running in the background and quite
    // fast for most standard file sizes, plus we're caching the previews,
    // I'm not sure the extra complexity is worth it.
    pub fn preview(
        &mut self,
        entry: &Entry,
        preview_window: Option<Rect>,
    ) -> Option<Arc<Preview>> {
        // check if we have a preview for the current request
        if let Some(preview) = self.cached(entry) {
            return Some(preview);
        }

        // otherwise, if we haven't acknowledged the request yet, acknowledge it
        self.requests.push(entry.clone());

        // lookup request stack and return the most recent preview available
        for request in self.requests.back_to_front() {
            if let Some(preview) =
                self.dispatch_request(&request, preview_window)
            {
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
