use ratatui::{
    buffer::Buffer, layout::Rect, style::Style, widgets::StatefulWidget,
};

const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// A spinner widget.
#[derive(Debug, Clone, Copy)]
pub struct Spinner {
    frames: &'static [&'static str],
}

impl Spinner {
    pub fn new(frames: &'static [&str]) -> Spinner {
        Spinner { frames }
    }

    pub fn frame(&self, index: usize) -> &str {
        self.frames[index]
    }
}

impl Default for Spinner {
    fn default() -> Spinner {
        Spinner::new(FRAMES)
    }
}

#[derive(Debug)]
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

impl StatefulWidget for Spinner {
    type State = SpinnerState;

    /// Renders the spinner in the given area.
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_string(
            area.left(),
            area.top(),
            self.frame(state.current_frame),
            Style::default(),
        );
        state.tick();
    }
}
impl StatefulWidget for &Spinner {
    type State = SpinnerState;

    /// Renders the spinner in the given area.
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_string(
            area.left(),
            area.top(),
            self.frame(state.current_frame),
            Style::default(),
        );
        state.tick();
    }
}
