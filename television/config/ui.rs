use crate::channels::prototypes::Template;
use serde::{Deserialize, Serialize};

use crate::screen::layout::{
    InputPosition, Orientation, PreviewTitlePosition,
};

use super::themes::DEFAULT_THEME;

const DEFAULT_UI_SCALE: u16 = 100;

fn serialize_maybe_template<S>(
    template: &Option<Template>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match template {
        Some(tpl) => serializer.serialize_str(tpl.raw()),
        None => serializer.serialize_none(),
    }
}

fn deserialize_maybe_template<'de, D>(
    deserializer: D,
) -> Result<Option<Template>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw: Option<String> = Option::<String>::deserialize(deserializer)?;
    match raw {
        Some(s) => Template::parse(&s)
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

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
    pub orientation: Orientation,
    pub preview_title_position: Option<PreviewTitlePosition>,
    pub theme: String,
    #[serde(
        default,
        deserialize_with = "deserialize_maybe_template",
        serialize_with = "serialize_maybe_template"
    )]
    pub input_header: Option<Template>,
    #[serde(
        default,
        deserialize_with = "deserialize_maybe_template",
        serialize_with = "serialize_maybe_template"
    )]
    pub preview_header: Option<Template>,
    #[serde(
        default,
        deserialize_with = "deserialize_maybe_template",
        serialize_with = "serialize_maybe_template"
    )]
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
            input_bar_position: InputPosition::Top,
            orientation: Orientation::Landscape,
            preview_title_position: None,
            theme: String::from(DEFAULT_THEME),
            input_header: None,
            preview_header: None,
            preview_footer: None,
        }
    }
}
