use ratatui::style::Color;

pub const BORDER_COLOR: Color = Color::Blue;
pub const ACTION_COLOR: Color = Color::DarkGray;
// Styles
//  input
pub const DEFAULT_INPUT_FG: Color = Color::LightRed;
pub const DEFAULT_RESULTS_COUNT_FG: Color = Color::LightRed;
//  preview
pub const DEFAULT_PREVIEW_TITLE_FG: Color = Color::Blue;
pub const DEFAULT_SELECTED_PREVIEW_BG: Color = Color::Rgb(50, 50, 50);
pub const DEFAULT_PREVIEW_CONTENT_FG: Color = Color::Rgb(150, 150, 180);
pub const DEFAULT_PREVIEW_GUTTER_FG: Color = Color::Rgb(70, 70, 70);
pub const DEFAULT_PREVIEW_GUTTER_SELECTED_FG: Color =
    Color::Rgb(255, 150, 150);
// Styles
pub const DEFAULT_RESULT_NAME_FG: Color = Color::Blue;
pub const DEFAULT_RESULT_PREVIEW_FG: Color = Color::Rgb(150, 150, 150);
pub const DEFAULT_RESULT_LINE_NUMBER_FG: Color = Color::Yellow;
pub const DEFAULT_RESULT_SELECTED_BG: Color = Color::Rgb(50, 50, 50);

pub const DEFAULT_RESULTS_LIST_MATCH_FOREGROUND_COLOR: Color = Color::Red;

pub struct ResultsListColors {
    pub result_name_fg: Color,
    pub result_preview_fg: Color,
    pub result_line_number_fg: Color,
    pub result_selected_bg: Color,
}

impl Default for ResultsListColors {
    fn default() -> Self {
        Self {
            result_name_fg: DEFAULT_RESULT_NAME_FG,
            result_preview_fg: DEFAULT_RESULT_PREVIEW_FG,
            result_line_number_fg: DEFAULT_RESULT_LINE_NUMBER_FG,
            result_selected_bg: DEFAULT_RESULT_SELECTED_BG,
        }
    }
}

#[allow(dead_code)]
impl ResultsListColors {
    pub fn result_name_fg(mut self, color: Color) -> Self {
        self.result_name_fg = color;
        self
    }

    pub fn result_preview_fg(mut self, color: Color) -> Self {
        self.result_preview_fg = color;
        self
    }

    pub fn result_line_number_fg(mut self, color: Color) -> Self {
        self.result_line_number_fg = color;
        self
    }

    pub fn result_selected_bg(mut self, color: Color) -> Self {
        self.result_selected_bg = color;
        self
    }
}
