use ratatui::style::{Color, Style, Stylize};

pub mod input;
pub mod results;
pub mod preview;
pub mod layout;

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
    if focused {
        Style::default().fg(Color::Green)
    } else {
        // TODO: make this depend on self.config
        Style::default().fg(Color::Rgb(90, 90, 110)).dim()
    }
}

