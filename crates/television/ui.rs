use ratatui::style::{Color, Style};

pub(crate) mod help;
pub mod input;
pub mod keymap;
pub mod layout;
pub mod logo;
pub mod metadata;
pub mod preview;
mod remote_control;
pub mod results;
pub mod spinner;
//  input
//const DEFAULT_INPUT_FG: Color = Color::Rgb(200, 200, 200);
//const DEFAULT_RESULTS_COUNT_FG: Color = Color::Rgb(150, 150, 150);
//  preview
//const DEFAULT_PREVIEW_TITLE_FG: Color = Color::Blue;
//const DEFAULT_SELECTED_PREVIEW_BG: Color = Color::Rgb(50, 50, 50);
//const DEFAULT_PREVIEW_CONTENT_FG: Color = Color::Rgb(150, 150, 180);
//const DEFAULT_PREVIEW_GUTTER_FG: Color = Color::Rgb(70, 70, 70);
//const DEFAULT_PREVIEW_GUTTER_SELECTED_FG: Color = Color::Rgb(255, 150, 150);

pub fn get_border_style(focused: bool) -> Style {
    Style::default().fg(Color::Blue)

    // NOTE: do we want to change the border color based on focus? Are we
    // keeping the focus feature at all?
    // if focused {
    //    Style::default().fg(Color::Green)
    // } else {
    //    Style::default().fg(Color::Blue)
    // }
}
