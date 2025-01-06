use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct Colorscheme {
    pub general: GeneralColorscheme,
    pub help: HelpColorscheme,
    pub results: ResultsColorscheme,
    pub preview: PreviewColorscheme,
    pub input: InputColorscheme,
    pub mode: ModeColorscheme,
}

#[derive(Debug, Clone)]
pub struct GeneralColorscheme {
    pub border_fg: Color,
    pub background: Option<Color>,
}

#[derive(Debug, Clone)]
pub struct HelpColorscheme {
    pub metadata_field_name_fg: Color,
    pub metadata_field_value_fg: Color,
}

#[derive(Debug, Clone)]
pub struct ResultsColorscheme {
    pub result_name_fg: Color,
    pub result_preview_fg: Color,
    pub result_line_number_fg: Color,
    pub result_selected_bg: Color,
    pub result_selected_fg: Color,
    pub match_foreground_color: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct PreviewColorscheme {
    pub title_fg: Color,
    pub highlight_bg: Color,
    pub content_fg: Color,
    pub gutter_fg: Color,
    pub gutter_selected_fg: Color,
}

#[derive(Debug, Clone)]
pub struct InputColorscheme {
    pub input_fg: Color,
    pub results_count_fg: Color,
}

#[derive(Debug, Clone)]
pub struct ModeColorscheme {
    pub channel: Color,
    pub remote_control: Color,
    pub send_to_channel: Color,
}
