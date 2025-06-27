use crate::{
    channels::prototypes::Template,
    config::themes::DEFAULT_THEME,
    features::Features,
    screen::layout::{InputPosition, Orientation},
};
use serde::{Deserialize, Serialize};

pub const DEFAULT_UI_SCALE: u16 = 100;
pub const DEFAULT_PREVIEW_SIZE: u16 = 50;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Hash, Default)]
#[serde(default)]
pub struct StatusBarConfig {
    pub separator_open: String,
    pub separator_close: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Hash)]
#[serde(default)]
pub struct PreviewPanelConfig {
    pub size: u16,
    pub header: Option<Template>,
    pub footer: Option<Template>,
    pub scrollbar: bool,
}

impl Default for PreviewPanelConfig {
    fn default() -> Self {
        Self {
            size: DEFAULT_PREVIEW_SIZE,
            header: None,
            footer: None,
            scrollbar: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Hash)]
#[serde(default)]
pub struct HelpPanelConfig {
    pub show_categories: bool,
}

impl Default for HelpPanelConfig {
    fn default() -> Self {
        Self {
            show_categories: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Hash)]
#[serde(default)]
pub struct RemoteControlConfig {
    pub show_channel_descriptions: bool,
    pub sort_alphabetically: bool,
}

impl Default for RemoteControlConfig {
    fn default() -> Self {
        Self {
            show_channel_descriptions: true,
            sort_alphabetically: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Hash)]
#[serde(default)]
pub struct UiConfig {
    pub use_nerd_font_icons: bool,
    pub ui_scale: u16,
    pub input_bar_position: InputPosition,
    pub orientation: Orientation,
    pub theme: String,
    pub input_header: Option<Template>,
    pub features: Features,

    // Feature-specific configurations
    pub status_bar: StatusBarConfig,
    pub preview_panel: PreviewPanelConfig,
    pub help_panel: HelpPanelConfig,
    pub remote_control: RemoteControlConfig,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            use_nerd_font_icons: false,
            ui_scale: DEFAULT_UI_SCALE,
            input_bar_position: InputPosition::Top,
            orientation: Orientation::Landscape,
            theme: String::from(DEFAULT_THEME),
            input_header: None,
            features: Features::default(),
            status_bar: StatusBarConfig::default(),
            preview_panel: PreviewPanelConfig::default(),
            help_panel: HelpPanelConfig::default(),
            remote_control: RemoteControlConfig::default(),
        }
    }
}
