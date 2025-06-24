use super::themes::DEFAULT_THEME;
use crate::channels::prototypes::Template;
use crate::features::Features;
use crate::screen::layout::{
    InputPosition, Orientation, PreviewTitlePosition,
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
    pub position: PreviewPosition,
    pub header: Option<Template>,
    pub footer: Option<Template>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Hash)]
pub enum PreviewPosition {
    #[serde(alias = "right")]
    Right,
    #[serde(alias = "left")]
    Left,
    #[serde(alias = "top")]
    Top,
    #[serde(alias = "bottom")]
    Bottom,
}

impl Default for PreviewPosition {
    fn default() -> Self {
        Self::Right
    }
}

impl Default for PreviewPanelConfig {
    fn default() -> Self {
        Self {
            size: DEFAULT_PREVIEW_SIZE,
            position: PreviewPosition::default(),
            header: None,
            footer: None,
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
    pub preview_title_position: Option<PreviewTitlePosition>,
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
            preview_title_position: None,
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

impl UiConfig {
    pub fn preview_enabled(&self) -> bool {
        self.features.contains(Features::PREVIEW_PANEL)
    }

    pub fn help_panel_enabled(&self) -> bool {
        self.features.contains(Features::HELP_PANEL)
    }

    pub fn status_bar_enabled(&self) -> bool {
        self.features.contains(Features::STATUS_BAR)
    }

    pub fn remote_control_enabled(&self) -> bool {
        self.features.contains(Features::REMOTE_CONTROL)
    }

    pub fn toggle_feature(&mut self, feat: Features) {
        self.features.toggle(feat);
    }
}
