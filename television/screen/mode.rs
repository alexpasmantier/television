use ratatui::style::Color;

use crate::{screen::colors::ModeColorscheme, television::Mode};

pub fn mode_color(mode: Mode, colorscheme: &ModeColorscheme) -> Color {
    match mode {
        Mode::Channel => colorscheme.channel,
        Mode::RemoteControl => colorscheme.remote_control,
        Mode::SendToChannel => colorscheme.send_to_channel,
        Mode::Action => colorscheme.action,
    }
}
