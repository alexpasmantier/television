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
        if self.preview.entry_raw != preview.entry_raw
            || self.preview.content != preview.content
            || self.preview.target_line != preview.target_line
        {
            self.preview = preview;
            self.scroll = scroll;
        }
    }

    // FIXME: does this really need to happen for every render?
    // What if we did it only when the preview content or scroll changes?
    pub fn for_render_context(&self, height: usize) -> Self {
        // PERF: this allocates every time
        let scroll = self.scroll as usize;
        let num_lines = self.preview.total_lines.saturating_sub(
            u16::try_from(scroll).expect("scroll should fit in a u16"),
        ) as usize;
        let cropped_content: Text<'_> = self.preview.content.lines
            [scroll..scroll + num_lines.min(height)]
            .to_vec()
            .into();

        let adjusted_line_number = self
            .preview
            .target_line
            .map(|line| line.saturating_sub(self.scroll));

        PreviewState::new(
            self.enabled,
            // PERF: this allocates every time
            Preview::new(
                self.preview.entry_raw.clone(),
                self.preview.formatted_command.clone(),
                &self.preview.title,
                cropped_content,
                adjusted_line_number,
                self.preview.total_lines,
                self.preview.footer.clone(),
                self.preview.preview_index,
                self.preview.preview_count,
            ),
            self.scroll,
        )
    }
}
