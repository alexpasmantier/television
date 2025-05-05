use std::sync::Arc;

use devicons::FileIcon;

pub mod cache;
pub mod meta;
pub mod previewer;

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum PreviewContent {
    Empty,
    Loading,
    Timeout,
    AnsiText(String),
}

impl PreviewContent {
    pub fn total_lines(&self) -> u16 {
        match self {
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
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Preview {
    pub title: String,
    pub content: PreviewContent,
    pub icon: Option<FileIcon>,
    pub total_lines: u16,
}

impl Default for Preview {
    fn default() -> Self {
        Preview {
            title: String::new(),
            content: PreviewContent::Empty,
            icon: None,
            total_lines: 0,
        }
    }
}

impl Preview {
    pub fn new(
        title: String,
        content: PreviewContent,
        icon: Option<FileIcon>,
        total_lines: u16,
    ) -> Self {
        Preview {
            title,
            content,
            icon,
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
