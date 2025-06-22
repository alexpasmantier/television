use crate::channels::prototypes::Template;
use serde::{Deserialize, Serialize};

use crate::screen::layout::{
    InputPosition, Orientation, PreviewTitlePosition,
};

use super::themes::DEFAULT_THEME;

pub const DEFAULT_UI_SCALE: u16 = 100;
pub const DEFAULT_PREVIEW_SIZE: u16 = 50;

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Hash)]
#[serde(default)]
pub struct UiConfig {
    pub use_nerd_font_icons: bool,
    pub ui_scale: u16,
    pub no_help: bool,
    pub show_help_bar: bool,
    pub show_preview_panel: bool,
    pub show_keybinding_panel: bool,
    #[serde(default)]
    pub input_bar_position: InputPosition,
    pub orientation: Orientation,
    pub preview_title_position: Option<PreviewTitlePosition>,
    pub theme: String,
    pub preview_size: u16,
    #[serde(default)]
    pub input_header: Option<Template>,
    #[serde(default)]
    pub preview_header: Option<Template>,
    #[serde(default)]
    pub preview_footer: Option<Template>,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            use_nerd_font_icons: false,
            ui_scale: DEFAULT_UI_SCALE,
            no_help: false,
            show_help_bar: false,
            show_preview_panel: true,
            show_keybinding_panel: false,
            input_bar_position: InputPosition::Top,
            orientation: Orientation::Landscape,
            preview_title_position: None,
            theme: String::from(DEFAULT_THEME),
            preview_size: DEFAULT_PREVIEW_SIZE,
            input_header: None,
            preview_header: None,
            preview_footer: None,
        }
    }
}
