use config::ValueKind;
use serde::Deserialize;
use std::collections::HashMap;

use television_screen::layout::{InputPosition, PreviewTitlePosition};

use super::themes::DEFAULT_THEME;

const DEFAULT_UI_SCALE: u16 = 90;

#[derive(Clone, Debug, Deserialize)]
pub struct UiConfig {
    pub use_nerd_font_icons: bool,
    pub ui_scale: u16,
    pub show_help_bar: bool,
    #[serde(default)]
    pub input_bar_position: InputPosition,
    pub preview_title_position: PreviewTitlePosition,
    pub theme: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            use_nerd_font_icons: false,
            ui_scale: DEFAULT_UI_SCALE,
            show_help_bar: true,
            input_bar_position: InputPosition::Bottom,
            preview_title_position: PreviewTitlePosition::Top,
            theme: String::from(DEFAULT_THEME),
        }
    }
}

impl From<UiConfig> for ValueKind {
    fn from(val: UiConfig) -> Self {
        let mut m = HashMap::new();
        m.insert(
            String::from("use_nerd_font_icons"),
            ValueKind::Boolean(val.use_nerd_font_icons).into(),
        );
        m.insert(
            String::from("ui_scale"),
            ValueKind::U64(val.ui_scale.into()).into(),
        );
        m.insert(
            String::from("show_help_bar"),
            ValueKind::Boolean(val.show_help_bar).into(),
        );
        m.insert(
            String::from("input_position"),
            ValueKind::String(val.input_bar_position.to_string()).into(),
        );
        m.insert(
            String::from("preview_title_position"),
            ValueKind::String(val.preview_title_position.to_string()).into(),
        );
        m.insert(String::from("theme"), ValueKind::String(val.theme).into());
        ValueKind::Table(m)
    }
}
