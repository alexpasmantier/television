use crate::{
    channels::prototypes::Template,
    config::themes::DEFAULT_THEME,
    screen::layout::{InputPosition, Orientation},
};
use serde::{Deserialize, Serialize};

pub const DEFAULT_UI_SCALE: u16 = 100;
pub const DEFAULT_PREVIEW_SIZE: u16 = 50;
pub const DEFAULT_PROMPT: &str = ">";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Hash)]
#[serde(default)]
pub struct InputBarConfig {
    pub position: InputPosition,
    pub header: Option<String>,
    #[serde(default = "default_prompt")]
    pub prompt: Option<String>,
    pub border_type: BorderType,
    pub padding: Padding,
}

impl Default for InputBarConfig {
    fn default() -> Self {
        Self {
            position: InputPosition::default(),
            header: None,
            prompt: Some(String::from(DEFAULT_PROMPT)),
            border_type: BorderType::default(),
            padding: Padding::uniform(0),
        }
    }
}

#[allow(clippy::unnecessary_wraps)]
fn default_prompt() -> Option<String> {
    Some(String::from(DEFAULT_PROMPT))
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Hash, Default)]
#[serde(default)]
pub struct StatusBarConfig {
    pub separator_open: String,
    pub separator_close: String,
    pub hidden: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Hash, Default)]
#[serde(default)]
pub struct ResultsPanelConfig {
    pub border_type: BorderType,
    pub padding: Padding,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Hash)]
#[serde(default)]
pub struct PreviewPanelConfig {
    pub size: u16,
    pub header: Option<Template>,
    pub footer: Option<Template>,
    pub scrollbar: bool,
    pub border_type: BorderType,
    pub padding: Padding,
    pub hidden: bool,
}

impl Default for PreviewPanelConfig {
    fn default() -> Self {
        Self {
            size: DEFAULT_PREVIEW_SIZE,
            header: None,
            footer: None,
            scrollbar: true,
            border_type: BorderType::default(),
            padding: Padding::uniform(0),
            hidden: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Hash)]
#[serde(default)]
pub struct HelpPanelConfig {
    pub show_categories: bool,
    #[serde(default = "default_help_hidden")]
    pub hidden: bool,
    pub disabled: bool,
}

fn default_help_hidden() -> bool {
    true
}

impl Default for HelpPanelConfig {
    fn default() -> Self {
        Self {
            show_categories: true,
            hidden: true,
            disabled: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Hash)]
#[serde(default)]
pub struct RemoteControlConfig {
    pub show_channel_descriptions: bool,
    pub sort_alphabetically: bool,
    pub disabled: bool,
}

impl Default for RemoteControlConfig {
    fn default() -> Self {
        Self {
            show_channel_descriptions: true,
            sort_alphabetically: true,
            disabled: false,
        }
    }
}

/// Theme color overrides that can be specified in the configuration file
/// to customize the appearance of the selected theme
#[derive(
    Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Default,
)]
#[serde(default)]
pub struct ThemeOverrides {
    // General colors
    pub background: Option<String>,
    pub border_fg: Option<String>,
    pub text_fg: Option<String>,
    pub dimmed_text_fg: Option<String>,

    // Input colors
    pub input_text_fg: Option<String>,
    pub result_count_fg: Option<String>,

    // Result colors
    pub result_name_fg: Option<String>,
    pub result_line_number_fg: Option<String>,
    pub result_value_fg: Option<String>,
    pub selection_bg: Option<String>,
    pub selection_fg: Option<String>,
    pub match_fg: Option<String>,

    // Preview colors
    pub preview_title_fg: Option<String>,

    // Mode colors
    pub channel_mode_fg: Option<String>,
    pub channel_mode_bg: Option<String>,
    pub remote_control_mode_fg: Option<String>,
    pub remote_control_mode_bg: Option<String>,
}

impl ThemeOverrides {
    pub fn merge(self, other: Self) -> Self {
        Self {
            background: self.background.or(other.background),
            border_fg: self.border_fg.or(other.border_fg),
            text_fg: self.text_fg.or(other.text_fg),
            dimmed_text_fg: self.dimmed_text_fg.or(other.dimmed_text_fg),
            input_text_fg: self.input_text_fg.or(other.input_text_fg),
            result_count_fg: self.result_count_fg.or(other.result_count_fg),
            result_name_fg: self.result_name_fg.or(other.result_name_fg),
            result_line_number_fg: self
                .result_line_number_fg
                .or(other.result_line_number_fg),
            result_value_fg: self.result_value_fg.or(other.result_value_fg),
            selection_bg: self.selection_bg.or(other.selection_bg),
            selection_fg: self.selection_fg.or(other.selection_fg),
            match_fg: self.match_fg.or(other.match_fg),
            preview_title_fg: self.preview_title_fg.or(other.preview_title_fg),
            channel_mode_fg: self.channel_mode_fg.or(other.channel_mode_fg),
            channel_mode_bg: self.channel_mode_bg.or(other.channel_mode_bg),
            remote_control_mode_fg: self
                .remote_control_mode_fg
                .or(other.remote_control_mode_fg),
            remote_control_mode_bg: self
                .remote_control_mode_bg
                .or(other.remote_control_mode_bg),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Hash)]
#[serde(default)]
pub struct UiConfig {
    pub ui_scale: u16,
    pub orientation: Orientation,
    pub theme: String,

    // Feature-specific configurations
    pub input_bar: InputBarConfig,
    pub status_bar: StatusBarConfig,
    pub preview_panel: PreviewPanelConfig,
    pub results_panel: ResultsPanelConfig,
    pub help_panel: HelpPanelConfig,
    pub remote_control: RemoteControlConfig,

    // Theme color overrides
    #[serde(default)]
    pub theme_overrides: ThemeOverrides,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            ui_scale: DEFAULT_UI_SCALE,
            orientation: Orientation::Landscape,
            theme: String::from(DEFAULT_THEME),
            input_bar: InputBarConfig::default(),
            status_bar: StatusBarConfig::default(),
            preview_panel: PreviewPanelConfig::default(),
            results_panel: ResultsPanelConfig::default(),
            help_panel: HelpPanelConfig::default(),
            remote_control: RemoteControlConfig::default(),
            theme_overrides: ThemeOverrides::default(),
        }
    }
}

#[derive(
    Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Hash, Default, Eq,
)]
#[serde(rename_all = "snake_case")]
pub enum BorderType {
    None,
    Plain,
    #[default]
    Rounded,
    Thick,
}

impl BorderType {
    pub fn to_ratatui_border_type(
        &self,
    ) -> Option<ratatui::widgets::BorderType> {
        match self {
            BorderType::None => None,
            BorderType::Plain => Some(ratatui::widgets::BorderType::Plain),
            BorderType::Rounded => Some(ratatui::widgets::BorderType::Rounded),
            BorderType::Thick => Some(ratatui::widgets::BorderType::Thick),
        }
    }
}

impl From<crate::cli::args::BorderType> for BorderType {
    fn from(border_type: crate::cli::args::BorderType) -> Self {
        match border_type {
            crate::cli::args::BorderType::None => BorderType::None,
            crate::cli::args::BorderType::Plain => BorderType::Plain,
            crate::cli::args::BorderType::Rounded => BorderType::Rounded,
            crate::cli::args::BorderType::Thick => BorderType::Thick,
        }
    }
}

#[derive(
    Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Hash, Default, Eq,
)]
#[serde(default)]
pub struct Padding {
    pub top: u16,
    pub bottom: u16,
    pub left: u16,
    pub right: u16,
}

impl Padding {
    pub fn new(top: u16, bottom: u16, left: u16, right: u16) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    pub fn uniform(padding: u16) -> Self {
        Self {
            top: padding,
            bottom: padding,
            left: padding,
            right: padding,
        }
    }

    pub fn horizontal(padding: u16) -> Self {
        Self {
            top: 0,
            bottom: 0,
            left: padding,
            right: padding,
        }
    }

    pub fn vertical(padding: u16) -> Self {
        Self {
            top: padding,
            bottom: padding,
            left: 0,
            right: 0,
        }
    }
}

impl From<Padding> for ratatui::widgets::Padding {
    fn from(padding: Padding) -> Self {
        ratatui::widgets::Padding {
            top: padding.top,
            bottom: padding.bottom,
            left: padding.left,
            right: padding.right,
        }
    }
}
