use ratatui::style::Color;

pub(crate) mod help;
pub mod input;
pub mod keymap;
pub mod layout;
pub mod logo;
pub mod metadata;
mod mode;
pub mod preview;
mod remote_control;
pub mod results;
pub mod spinner;

pub const BORDER_COLOR: Color = Color::Blue;
