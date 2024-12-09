use std::collections::HashMap;

use config::ValueKind;
use ratatui::style::Color as RatatuiColor;
use serde::Deserialize;
use television_screen::colors::{
    Colorscheme, GeneralColorscheme, HelpColorscheme, InputColorscheme,
    PreviewColorscheme, ResultsColorscheme,
};

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
    pub background: Color,
    pub foreground: Color,
    pub black: Color,
    pub red: Color,
    pub green: Color,
    pub yellow: Color,
    pub blue: Color,
    pub magenta: Color,
    pub cyan: Color,
    pub white: Color,
    pub bright_black: Color,
    pub bright_red: Color,
    pub bright_green: Color,
    pub bright_yellow: Color,
    pub bright_blue: Color,
    pub bright_magenta: Color,
    pub bright_cyan: Color,
    pub bright_white: Color,
}

const DEFAULT_THEME: &str = r##"
background = "#282c34"
foreground = "#abb2bf"
black = "#5c6370"
red = "#e06c75"
green = "#98c379"
yellow = "#e5c07b"
blue = "#61afef"
magenta = "#c678dd"
cyan = "#56b6c2"
white = "#abb2bf"
bright_black = "#5c6370"
bright_red = "#e06c75"
bright_green = "#98c379"
bright_yellow = "#e5c07b"
bright_blue = "#61afef"
bright_magenta = "#c678dd"
bright_cyan = "#56b6c2"
bright_white = "#abb2bf"
"##;

impl Default for Theme {
    fn default() -> Self {
        toml::from_str(DEFAULT_THEME).unwrap()
    }
}

impl<'de> Deserialize<'de> for Theme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename = "theme")]
        struct Inner {
            background: String,
            foreground: String,
            black: String,
            red: String,
            green: String,
            yellow: String,
            blue: String,
            magenta: String,
            cyan: String,
            white: String,
            bright_black: String,
            bright_red: String,
            bright_green: String,
            bright_yellow: String,
            bright_blue: String,
            bright_magenta: String,
            bright_cyan: String,
            bright_white: String,
        }

        let inner = Inner::deserialize(deserializer)?;
        Ok(Self {
            background: Color::from_str(&inner.background)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            foreground: Color::from_str(&inner.foreground)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            black: Color::from_str(&inner.black)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            red: Color::from_str(&inner.red)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            green: Color::from_str(&inner.green)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            yellow: Color::from_str(&inner.yellow)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            blue: Color::from_str(&inner.blue)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            magenta: Color::from_str(&inner.magenta)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            cyan: Color::from_str(&inner.cyan)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            white: Color::from_str(&inner.white)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            bright_black: Color::from_str(&inner.bright_black)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            bright_red: Color::from_str(&inner.bright_red)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            bright_green: Color::from_str(&inner.bright_green)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            bright_yellow: Color::from_str(&inner.bright_yellow)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            bright_blue: Color::from_str(&inner.bright_blue)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            bright_magenta: Color::from_str(&inner.bright_magenta)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            bright_cyan: Color::from_str(&inner.bright_cyan)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
            bright_white: Color::from_str(&inner.bright_white)
                .ok_or_else(|| serde::de::Error::custom("invalid color"))?,
        })
    }
}

impl From<Theme> for ValueKind {
    fn from(val: Theme) -> Self {
        let mut m = HashMap::new();
        m.insert(
            String::from("background"),
            ValueKind::String(format!(
                "#{:02x}{:02x}{:02x}",
                val.background.r, val.background.g, val.background.b
            ))
            .into(),
        );
        m.insert(
            String::from("foreground"),
            ValueKind::String(format!(
                "#{:02x}{:02x}{:02x}",
                val.foreground.r, val.foreground.g, val.foreground.b
            ))
            .into(),
        );
        m.insert(
            String::from("black"),
            ValueKind::String(format!(
                "#{:02x}{:02x}{:02x}",
                val.black.r, val.black.g, val.black.b
            ))
            .into(),
        );
        m.insert(
            String::from("red"),
            ValueKind::String(format!(
                "#{:02x}{:02x}{:02x}",
                val.red.r, val.red.g, val.red.b
            ))
            .into(),
        );
        m.insert(
            String::from("green"),
            ValueKind::String(format!(
                "#{:02x}{:02x}{:02x}",
                val.green.r, val.green.g, val.green.b
            ))
            .into(),
        );
        m.insert(
            String::from("yellow"),
            ValueKind::String(format!(
                "#{:02x}{:02x}{:02x}",
                val.yellow.r, val.yellow.g, val.yellow.b
            ))
            .into(),
        );
        m.insert(
            String::from("blue"),
            ValueKind::String(format!(
                "#{:02x}{:02x}{:02x}",
                val.blue.r, val.blue.g, val.blue.b
            ))
            .into(),
        );
        m.insert(
            String::from("magenta"),
            ValueKind::String(format!(
                "#{:02x}{:02x}{:02x}",
                val.magenta.r, val.magenta.g, val.magenta.b
            ))
            .into(),
        );
        m.insert(
            String::from("cyan"),
            ValueKind::String(format!(
                "#{:02x}{:02x}{:02x}",
                val.cyan.r, val.cyan.g, val.cyan.b
            ))
            .into(),
        );
        m.insert(
            String::from("white"),
            ValueKind::String(format!(
                "#{:02x}{:02x}{:02x}",
                val.white.r, val.white.g, val.white.b
            ))
            .into(),
        );
        m.insert(
            String::from("bright_black"),
            ValueKind::String(format!(
                "#{:02x}{:02x}{:02x}",
                val.bright_black.r, val.bright_black.g, val.bright_black.b
            ))
            .into(),
        );
        m.insert(
            String::from("bright_red"),
            ValueKind::String(format!(
                "#{:02x}{:02x}{:02x}",
                val.bright_red.r, val.bright_red.g, val.bright_red.b
            ))
            .into(),
        );
        m.insert(
            String::from("bright_green"),
            ValueKind::String(format!(
                "#{:02x}{:02x}{:02x}",
                val.bright_green.r, val.bright_green.g, val.bright_green.b
            ))
            .into(),
        );
        m.insert(
            String::from("bright_yellow"),
            ValueKind::String(format!(
                "#{:02x}{:02x}{:02x}",
                val.bright_yellow.r, val.bright_yellow.g, val.bright_yellow.b
            ))
            .into(),
        );
        m.insert(
            String::from("bright_blue"),
            ValueKind::String(format!(
                "#{:02x}{:02x}{:02x}",
                val.bright_blue.r, val.bright_blue.g, val.bright_blue.b
            ))
            .into(),
        );
        m.insert(
            String::from("bright_magenta"),
            ValueKind::String(format!(
                "#{:02x}{:02x}{:02x}",
                val.bright_magenta.r,
                val.bright_magenta.g,
                val.bright_magenta.b
            ))
            .into(),
        );
        m.insert(
            String::from("bright_cyan"),
            ValueKind::String(format!(
                "#{:02x}{:02x}{:02x}",
                val.bright_cyan.r, val.bright_cyan.g, val.bright_cyan.b
            ))
            .into(),
        );
        m.insert(
            String::from("bright_white"),
            ValueKind::String(format!(
                "#{:02x}{:02x}{:02x}",
                val.bright_white.r, val.bright_white.g, val.bright_white.b
            ))
            .into(),
        );
        ValueKind::Table(m)
    }
}

impl Into<RatatuiColor> for &Color {
    fn into(self) -> RatatuiColor {
        RatatuiColor::Rgb(self.r, self.g, self.b)
    }
}

impl Into<Colorscheme> for &Theme {
    fn into(self) -> Colorscheme {
        Colorscheme {
            general: self.into(),
            help: self.into(),
            results: self.into(),
            preview: self.into(),
            input: self.into(),
        }
    }
}

impl Into<GeneralColorscheme> for &Theme {
    fn into(self) -> GeneralColorscheme {
        GeneralColorscheme {
            background: (&self.background).into(),
            border_fg: (&self.bright_black).into(),
        }
    }
}

impl Into<HelpColorscheme> for &Theme {
    fn into(self) -> HelpColorscheme {
        HelpColorscheme {
            action_fg: (&self.bright_black).into(),
            metadata_field_name_fg: (&self.bright_black).into(),
            metadata_field_value_fg: (&self.bright_white).into(),
        }
    }
}

impl Into<ResultsColorscheme> for &Theme {
    fn into(self) -> ResultsColorscheme {
        ResultsColorscheme {
            result_name_fg: (&self.blue).into(),
            result_preview_fg: (&self.bright_white).into(),
            result_line_number_fg: (&self.yellow).into(),
            result_selected_bg: (&self.bright_black).into(),
            match_foreground_color: (&self.red).into(),
        }
    }
}

impl Into<PreviewColorscheme> for &Theme {
    fn into(self) -> PreviewColorscheme {
        PreviewColorscheme {
            title_fg: (&self.bright_blue).into(),
            highlight_bg: (&self.bright_black).into(),
            content_fg: (&self.bright_white).into(),
            gutter_fg: (&self.bright_black).into(),
            gutter_selected_fg: (&self.red).into(),
        }
    }
}

impl Into<InputColorscheme> for &Theme {
    fn into(self) -> InputColorscheme {
        InputColorscheme {
            input_fg: (&self.bright_red).into(),
            results_count_fg: (&self.red).into(),
        }
    }
}
