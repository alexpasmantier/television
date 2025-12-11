use ratatui::text::Text;

use crate::previewer::Preview;

#[derive(Debug, Clone, Default)]
pub struct PreviewState {
    pub enabled: bool,
    // FIXME: this should probably be an Arc<Preview>
    pub preview: Preview,
    pub scroll: u16,
}

const PREVIEW_MIN_SCROLL_LINES: u16 = 3;
pub const ANSI_BEFORE_CONTEXT_SIZE: u16 = 3;
const ANSI_CONTEXT_SIZE: u16 = 150;

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
            || self.preview.target_line != preview.target_line
            || self.scroll != scroll
        {
            self.preview = preview;
            self.scroll = scroll;
        }
    }

    // FIXME: does this really need to happen for every render?
    // What if we did it only when the preview content or scroll changes?
    pub fn for_render_context(&self) -> Self {
        let num_skipped_lines =
            self.scroll.saturating_sub(ANSI_BEFORE_CONTEXT_SIZE);

        // PERF: this allocates every time
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
            .target_line
            .map(|line| line.saturating_sub(num_skipped_lines));

        PreviewState::new(
            self.enabled,
            // PERF: this allocates every time
            Preview::new(
                self.preview.entry_raw.clone(),
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
