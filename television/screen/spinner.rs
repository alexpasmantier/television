use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// A spinner widget.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct Spinner {
    frames: &'static [&'static str],
    state: SpinnerState,
}

impl Spinner {
    pub fn new(frames: &'static [&str]) -> Spinner {
        Spinner {
            frames,
            state: SpinnerState::new(frames.len()),
        }
    }

    pub fn frame(&self, index: usize) -> &str {
        self.frames[index]
    }

    pub fn tick(&mut self) {
        self.state.tick();
    }
}

impl Default for Spinner {
    fn default() -> Spinner {
        Spinner::new(FRAMES)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SpinnerState {
    pub current_frame: usize,
    total_frames: usize,
}

impl SpinnerState {
    pub fn new(total_frames: usize) -> SpinnerState {
        SpinnerState {
            current_frame: 0,
            total_frames,
        }
    }

    fn tick(&mut self) {
        self.current_frame = (self.current_frame + 1) % self.total_frames;
    }
}

impl From<&Spinner> for SpinnerState {
    fn from(spinner: &Spinner) -> SpinnerState {
        SpinnerState::new(spinner.frames.len())
    }
}

impl Widget for Spinner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_string(
            area.left(),
            area.top(),
            self.frame(self.state.current_frame),
            Style::default(),
        );
    }
}

impl Widget for &Spinner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_string(
            area.left(),
            area.top(),
            self.frame(self.state.current_frame),
            Style::default(),
        );
    }
}
