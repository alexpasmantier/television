use crate::{screen::colors::ModeColorscheme, television::Mode};
use ratatui::style::Color;

pub fn mode_color(mode: Mode, colorscheme: &ModeColorscheme) -> Color {
    match mode {
        Mode::Channel => colorscheme.channel,
        Mode::RemoteControl => colorscheme.remote_control,
    }
}
