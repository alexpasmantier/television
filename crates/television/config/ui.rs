use config::ValueKind;
use serde::Deserialize;
use std::collections::HashMap;

use television_screen::layout::{InputPosition, PreviewTitlePosition};

use super::themes::DEFAULT_THEME;

const DEFAULT_UI_SCALE: u16 = 100;

#[derive(Clone, Debug, Deserialize)]
pub struct UiConfig {
    pub use_nerd_font_icons: bool,
    pub ui_scale: u16,
    pub show_help_bar: bool,
    pub show_preview_panel: bool,
    #[serde(default)]
    pub input_bar_position: InputPosition,
    pub preview_title_position: Option<PreviewTitlePosition>,
    pub theme: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            use_nerd_font_icons: false,
            ui_scale: DEFAULT_UI_SCALE,
            show_help_bar: false,
            show_preview_panel: true,
            input_bar_position: InputPosition::Top,
            preview_title_position: None,
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
            String::from("show_preview_panel"),
            ValueKind::Boolean(val.show_preview_panel).into(),
        );
        m.insert(
            String::from("input_position"),
            ValueKind::String(val.input_bar_position.to_string()).into(),
        );
        m.insert(
            String::from("preview_title_position"),
            match val.preview_title_position {
                Some(pos) => ValueKind::String(pos.to_string()),
                None => ValueKind::Nil,
            }
            .into(),
        );
        m.insert(String::from("theme"), ValueKind::String(val.theme).into());
        ValueKind::Table(m)
    }
}
