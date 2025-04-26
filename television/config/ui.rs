use serde::{Deserialize, Serialize};

use crate::screen::layout::{InputPosition, PreviewTitlePosition};

use super::themes::DEFAULT_THEME;

const DEFAULT_UI_SCALE: u16 = 100;

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Hash)]
#[serde(default)]
pub struct UiConfig {
    pub use_nerd_font_icons: bool,
    pub ui_scale: u16,
    pub no_help: bool,
    pub show_help_bar: bool,
    pub show_preview_panel: bool,
    #[serde(default)]
    pub input_bar_position: InputPosition,
    pub preview_title_position: Option<PreviewTitlePosition>,
    pub theme: String,
    pub custom_header: Option<String>,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            use_nerd_font_icons: false,
            ui_scale: DEFAULT_UI_SCALE,
            no_help: false,
            show_help_bar: false,
            show_preview_panel: true,
            input_bar_position: InputPosition::Top,
            preview_title_position: None,
            theme: String::from(DEFAULT_THEME),
            custom_header: None,
        }
    }
}
