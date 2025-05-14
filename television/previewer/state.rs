use crate::previewer::Preview;

#[derive(Debug, Clone, Default)]
pub struct PreviewState {
    pub enabled: bool,
    pub preview: Preview,
    pub scroll: u16,
    pub target_line: Option<u16>,
}

const PREVIEW_MIN_SCROLL_LINES: u16 = 3;

impl PreviewState {
    pub fn new(
        enabled: bool,
        preview: Preview,
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
        self.preview = Preview::default();
        self.scroll = 0;
        self.target_line = None;
    }

    pub fn update(
        &mut self,
        preview: Preview,
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
