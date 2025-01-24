use ratatui::style::Color;
use serde::{Deserialize, Serialize};

use crate::screen::colors::ModeColorscheme;

pub fn mode_color(mode: Mode, colorscheme: &ModeColorscheme) -> Color {
    match mode {
        Mode::Channel => colorscheme.channel,
        Mode::RemoteControl => colorscheme.remote_control,
        Mode::SendToChannel => colorscheme.send_to_channel,
    }
}

// FIXME: Mode shouldn't be in the screen crate
#[derive(PartialEq, Copy, Clone, Hash, Eq, Debug, Serialize, Deserialize)]
pub enum Mode {
    Channel,
    RemoteControl,
    SendToChannel,
}
