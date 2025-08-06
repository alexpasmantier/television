use ratatui::text::Text;

use crate::previewer::Preview;

#[derive(Debug, Clone, Default)]
pub struct PreviewState {
    pub enabled: bool,
    pub preview: Preview,
    pub scroll: u16,
}

const PREVIEW_MIN_SCROLL_LINES: u16 = 3;
pub const ANSI_BEFORE_CONTEXT_SIZE: u16 = 3;
const ANSI_CONTEXT_SIZE: u16 = 500;

impl PreviewState {
    pub fn new(enabled: bool, preview: Preview, scroll: u16) -> Self {
        PreviewState {
            enabled,
            preview,
            scroll,
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
    }

    pub fn update(&mut self, preview: Preview, scroll: u16) {
        if self.preview.title != preview.title
            || self.preview.content != preview.content
            || self.preview.footer != preview.footer
            || self.preview.line_number != preview.line_number
            || self.scroll != scroll
        {
            self.preview = preview;
            self.scroll = scroll;
        }
    }

    pub fn for_render_context(&self) -> Self {
        let num_skipped_lines =
            self.scroll.saturating_sub(ANSI_BEFORE_CONTEXT_SIZE);

        let cropped_content = self
            .preview
            .content
            .lines
            .iter()
            .skip(num_skipped_lines as usize)
            .take(ANSI_CONTEXT_SIZE as usize)
            .cloned()
            .collect::<Vec<_>>();

        let adjusted_line_number = self
            .preview
            .line_number
            .map(|line| line.saturating_sub(num_skipped_lines));

        PreviewState::new(
            self.enabled,
            Preview::new(
                &self.preview.title,
                Text::from(cropped_content),
                adjusted_line_number,
                self.preview.total_lines,
                self.preview.footer.clone(),
            ),
            self.scroll,
        )
    }
}
