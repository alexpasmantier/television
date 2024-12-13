use color_eyre::Result;
use std::path::PathBuf;

use ratatui::style::Color as RatatuiColor;
use serde::Deserialize;
use television_screen::colors::{
    Colorscheme, GeneralColorscheme, HelpColorscheme, InputColorscheme,
    ModeColorscheme, PreviewColorscheme, ResultsColorscheme,
};

use super::get_config_dir;

pub mod builtin;

#[derive(Clone, Debug, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.trim_start_matches('#');
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(Self { r, g, b })
    }
}

#[derive(Clone, Debug)]
pub struct Theme {
    // general
    pub border_fg: Color,
    pub text_fg: Color,
    pub dimmed_text_fg: Color,
    // input
    pub input_text_fg: Color,
    pub result_count_fg: Color,
    // results
    pub result_name_fg: Color,
    pub result_line_number_fg: Color,
    pub result_value_fg: Color,
    pub selection_bg: Color,
    pub match_fg: Color,
    // preview
    pub preview_title_fg: Color,
    // modes
    pub channel_mode_fg: Color,
    pub remote_control_mode_fg: Color,
    pub send_to_channel_mode_fg: Color,
}

impl Theme {
    pub fn from_name(name: &str) -> Self {
        Self::from_path(
            &get_config_dir()
                .join("themes")
                .join(name)
                .with_extension("toml"),
        )
        .unwrap_or_else(|_| {
            Self::from_builtin(name).unwrap_or_else(|_| Self::default())
        })
    }

    pub fn from_builtin(
        name: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let theme_content: &str = builtin::BUILTIN_THEMES.get(name).map_or(
            builtin::BUILTIN_THEMES.get(DEFAULT_THEME).unwrap(),
            |t| *t,
        );
        let theme = toml::from_str(theme_content)?;
        Ok(theme)
    }

    pub fn from_path(
        path: &PathBuf,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let theme = std::fs::read_to_string(path)?;
        let theme: Theme = toml::from_str(&theme)?;
        Ok(theme)
    }
}

pub const DEFAULT_THEME: &str = "gruvbox-dark";

impl Default for Theme {
    fn default() -> Self {
        let theme_content = include_str!("../../../themes/gruvbox-dark.toml");
        toml::from_str(theme_content).unwrap()
    }
}

#[derive(Deserialize)]
#[serde(rename = "theme")]
struct Inner {
    // general
    border_fg: String,
    // info
    text_fg: String,
    dimmed_text_fg: String,
    // input
    input_text_fg: String,
    result_count_fg: String,
    //results
    result_name_fg: String,
    result_line_number_fg: String,
    result_value_fg: String,
    selection_bg: String,
    match_fg: String,
    //preview
    preview_title_fg: String,
    //modes
    channel_mode_fg: String,
    remote_control_mode_fg: String,
    send_to_channel_mode_fg: String,
}

impl<'de> Deserialize<'de> for Theme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = Inner::deserialize(deserializer).unwrap();
        Ok(Self {
            border_fg: Color::from_str(&inner.border_fg)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            text_fg: Color::from_str(&inner.text_fg)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            dimmed_text_fg: Color::from_str(&inner.dimmed_text_fg)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            input_text_fg: Color::from_str(&inner.input_text_fg)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            result_count_fg: Color::from_str(&inner.result_count_fg)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            result_name_fg: Color::from_str(&inner.result_name_fg)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            result_line_number_fg: Color::from_str(
                &inner.result_line_number_fg,
            )
            .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            result_value_fg: Color::from_str(&inner.result_value_fg)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            selection_bg: Color::from_str(&inner.selection_bg)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            match_fg: Color::from_str(&inner.match_fg)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            preview_title_fg: Color::from_str(&inner.preview_title_fg)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            channel_mode_fg: Color::from_str(&inner.channel_mode_fg)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            remote_control_mode_fg: Color::from_str(
                &inner.remote_control_mode_fg,
            )
            .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            send_to_channel_mode_fg: Color::from_str(
                &inner.send_to_channel_mode_fg,
            )
            .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
        })
    }
}

#[allow(clippy::from_over_into)]
impl Into<RatatuiColor> for &Color {
    fn into(self) -> RatatuiColor {
        RatatuiColor::Rgb(self.r, self.g, self.b)
    }
}

#[allow(clippy::from_over_into)]
impl Into<Colorscheme> for &Theme {
    fn into(self) -> Colorscheme {
        Colorscheme {
            general: self.into(),
            help: self.into(),
            results: self.into(),
            preview: self.into(),
            input: self.into(),
            mode: self.into(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<GeneralColorscheme> for &Theme {
    fn into(self) -> GeneralColorscheme {
        GeneralColorscheme {
            border_fg: (&self.border_fg).into(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<HelpColorscheme> for &Theme {
    fn into(self) -> HelpColorscheme {
        HelpColorscheme {
            metadata_field_name_fg: (&self.dimmed_text_fg).into(),
            metadata_field_value_fg: (&self.text_fg).into(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<ResultsColorscheme> for &Theme {
    fn into(self) -> ResultsColorscheme {
        ResultsColorscheme {
            result_name_fg: (&self.result_name_fg).into(),
            result_preview_fg: (&self.result_value_fg).into(),
            result_line_number_fg: (&self.result_line_number_fg).into(),
            result_selected_bg: (&self.selection_bg).into(),
            match_foreground_color: (&self.match_fg).into(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<PreviewColorscheme> for &Theme {
    fn into(self) -> PreviewColorscheme {
        PreviewColorscheme {
            title_fg: (&self.preview_title_fg).into(),
            highlight_bg: (&self.selection_bg).into(),
            content_fg: (&self.text_fg).into(),
            gutter_fg: (&self.dimmed_text_fg).into(),
            gutter_selected_fg: (&self.match_fg).into(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<InputColorscheme> for &Theme {
    fn into(self) -> InputColorscheme {
        InputColorscheme {
            input_fg: (&self.input_text_fg).into(),
            results_count_fg: (&self.result_count_fg).into(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<ModeColorscheme> for &Theme {
    fn into(self) -> ModeColorscheme {
        ModeColorscheme {
            channel: (&self.channel_mode_fg).into(),
            remote_control: (&self.remote_control_mode_fg).into(),
            send_to_channel: (&self.send_to_channel_mode_fg).into(),
        }
    }
}
