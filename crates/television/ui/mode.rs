use crate::television::Mode;
use ratatui::style::Color;

const CHANNEL_COLOR: Color = Color::LightYellow;
const REMOTE_CONTROL_COLOR: Color = Color::LightMagenta;
const SEND_TO_CHANNEL_COLOR: Color = Color::LightCyan;


pub fn mode_color(mode: Mode) -> Color {
    match mode {
        Mode::Channel => CHANNEL_COLOR,
        Mode::RemoteControl => REMOTE_CONTROL_COLOR,
        Mode::SendToChannel => SEND_TO_CHANNEL_COLOR,
    }
}