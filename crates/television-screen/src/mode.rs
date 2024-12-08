use ratatui::style::Color;
use serde::{Deserialize, Serialize};

pub const CHANNEL_COLOR: Color = Color::Indexed(222);
pub const REMOTE_CONTROL_COLOR: Color = Color::Indexed(1);
pub const SEND_TO_CHANNEL_COLOR: Color = Color::Indexed(105);

pub fn mode_color(mode: Mode) -> Color {
    match mode {
        Mode::Channel => CHANNEL_COLOR,
        Mode::RemoteControl => REMOTE_CONTROL_COLOR,
        Mode::SendToChannel => SEND_TO_CHANNEL_COLOR,
    }
}

// FIXME: Mode shouldn't be in the screen crate
#[derive(PartialEq, Copy, Clone, Hash, Eq, Debug, Serialize, Deserialize)]
pub enum Mode {
    Channel,
    RemoteControl,
    SendToChannel,
}
