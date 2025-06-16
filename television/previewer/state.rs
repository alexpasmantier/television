use crate::previewer::Preview;

#[derive(Debug, Clone, Default)]
pub struct PreviewState {
    pub enabled: bool,
    pub preview: Preview,
    pub scroll: u16,
    pub target_line: Option<u16>,
}

const PREVIEW_MIN_SCROLL_LINES: u16 = 3;
pub const ANSI_BEFORE_CONTEXT_SIZE: u16 = 3;
const ANSI_CONTEXT_SIZE: usize = 500;

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
        if self.preview.title != preview.title
            || self.preview.content != preview.content
            || self.preview.footer != preview.footer
            || self.scroll != scroll
        {
            self.preview = preview;
            self.scroll = scroll;
            self.target_line = target_line;
        }
    }

    pub fn for_render_context(&self) -> Self {
        let num_skipped_lines =
            self.scroll.saturating_sub(ANSI_BEFORE_CONTEXT_SIZE);
        let cropped_content = self
            .preview
            .content
            .lines()
            .skip(num_skipped_lines as usize)
            .take(ANSI_CONTEXT_SIZE)
            .collect::<Vec<_>>()
            .join("\n");

        let target_line: Option<u16> =
            if let Some(target_line) = self.target_line {
                if num_skipped_lines < target_line
                    && (target_line - num_skipped_lines)
                        <= u16::try_from(ANSI_CONTEXT_SIZE).unwrap()
                {
                    Some(target_line.saturating_sub(num_skipped_lines))
                } else {
                    None
                }
            } else {
                None
            };

        PreviewState::new(
            self.enabled,
            Preview::new(
                &self.preview.title,
                cropped_content,
                self.preview.icon,
                self.preview.total_lines,
                self.preview.footer.clone(),
            ),
            num_skipped_lines,
            target_line,
        )
    }
}
