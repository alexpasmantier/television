use anyhow::Result;
use std::path::PathBuf;

use crate::screen::colors::{
    Colorscheme, GeneralColorscheme, HelpColorscheme, InputColorscheme,
    ModeColorscheme, PreviewColorscheme, ResultsColorscheme,
};
use ratatui::style::Color as RatatuiColor;
use serde::Deserialize;

use super::get_config_dir;

pub mod builtin;

#[derive(Clone, Debug, PartialEq)]
pub enum Color {
    Ansi(ANSIColor),
    Rgb(RGBColor),
}

impl Color {
    pub fn from_str(s: &str) -> Option<Self> {
        if s.starts_with('#') {
            RGBColor::from_str(s).map(Self::Rgb)
        } else {
            match s.to_lowercase().as_str() {
                "black" => Some(Self::Ansi(ANSIColor::Black)),
                "red" => Some(Self::Ansi(ANSIColor::Red)),
                "green" => Some(Self::Ansi(ANSIColor::Green)),
                "yellow" => Some(Self::Ansi(ANSIColor::Yellow)),
                "blue" => Some(Self::Ansi(ANSIColor::Blue)),
                "magenta" => Some(Self::Ansi(ANSIColor::Magenta)),
                "cyan" => Some(Self::Ansi(ANSIColor::Cyan)),
                "white" => Some(Self::Ansi(ANSIColor::White)),
                "bright-black" => Some(Self::Ansi(ANSIColor::BrightBlack)),
                "bright-red" => Some(Self::Ansi(ANSIColor::BrightRed)),
                "bright-green" => Some(Self::Ansi(ANSIColor::BrightGreen)),
                "bright-yellow" => Some(Self::Ansi(ANSIColor::BrightYellow)),
                "bright-blue" => Some(Self::Ansi(ANSIColor::BrightBlue)),
                "bright-magenta" => Some(Self::Ansi(ANSIColor::BrightMagenta)),
                "bright-cyan" => Some(Self::Ansi(ANSIColor::BrightCyan)),
                "bright-white" => Some(Self::Ansi(ANSIColor::BrightWhite)),
                _ => None,
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ANSIColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RGBColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGBColor {
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

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    // general
    pub background: Option<Color>,
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
    pub selection_fg: Color,
    pub match_fg: Color,
    // preview
    pub preview_title_fg: Color,
    // modes
    pub channel_mode_fg: Color,
    pub channel_mode_bg: Color,
    pub remote_control_mode_fg: Color,
    pub remote_control_mode_bg: Color,
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
        let builtin_themes = builtin::builtin_themes();
        let theme_content: &str = builtin_themes
            .get(name)
            .map_or(builtin_themes.get(DEFAULT_THEME).unwrap(), |t| *t);
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

    /// Merge this theme with color overrides from the configuration
    pub fn merge_with_overrides(
        &self,
        overrides: &crate::config::ui::ThemeOverrides,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut merged_theme = self.clone();

        macro_rules! apply_override {
            ($field:ident, $override_field:expr) => {
                if let Some(ref color_str) = $override_field {
                    merged_theme.$field = Color::from_str(color_str)
                        .ok_or_else(|| {
                            format!(
                                "invalid {} color: {}",
                                stringify!($field),
                                color_str
                            )
                        })?;
                }
            };
            (opt $field:ident, $override_field:expr) => {
                if let Some(ref color_str) = $override_field {
                    merged_theme.$field =
                        Some(Color::from_str(color_str).ok_or_else(|| {
                            format!(
                                "invalid {} color: {}",
                                stringify!($field),
                                color_str
                            )
                        })?);
                }
            };
        }

        // Apply overrides using the macro
        apply_override!(opt background, overrides.background);
        apply_override!(border_fg, overrides.border_fg);
        apply_override!(text_fg, overrides.text_fg);
        apply_override!(dimmed_text_fg, overrides.dimmed_text_fg);
        apply_override!(input_text_fg, overrides.input_text_fg);
        apply_override!(result_count_fg, overrides.result_count_fg);
        apply_override!(result_name_fg, overrides.result_name_fg);
        apply_override!(
            result_line_number_fg,
            overrides.result_line_number_fg
        );
        apply_override!(result_value_fg, overrides.result_value_fg);
        apply_override!(selection_bg, overrides.selection_bg);
        apply_override!(selection_fg, overrides.selection_fg);
        apply_override!(match_fg, overrides.match_fg);
        apply_override!(preview_title_fg, overrides.preview_title_fg);
        apply_override!(channel_mode_fg, overrides.channel_mode_fg);
        apply_override!(channel_mode_bg, overrides.channel_mode_bg);
        apply_override!(
            remote_control_mode_fg,
            overrides.remote_control_mode_fg
        );
        apply_override!(
            remote_control_mode_bg,
            overrides.remote_control_mode_bg
        );

        Ok(merged_theme)
    }
}

pub const DEFAULT_THEME: &str = "default";

impl Default for Theme {
    fn default() -> Self {
        let theme_content = include_str!("../../themes/default.toml");
        toml::from_str(theme_content).unwrap()
    }
}

#[derive(Deserialize)]
#[serde(rename = "theme")]
struct Inner {
    // general
    background: Option<String>,
    border_fg: String,
    // info
    text_fg: String,
    dimmed_text_fg: String,
    // input
    input_text_fg: String,
    result_count_fg: String,
    // results
    result_name_fg: String,
    result_line_number_fg: String,
    result_value_fg: String,
    selection_bg: String,
    // this is made optional for theme backwards compatibility
    // and falls back to match_fg
    selection_fg: Option<String>,
    match_fg: String,
    // preview
    preview_title_fg: String,
    // modes
    channel_mode_fg: String,
    channel_mode_bg: Option<String>,
    remote_control_mode_fg: String,
    remote_control_mode_bg: String,
}

impl<'de> Deserialize<'de> for Theme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = Inner::deserialize(deserializer).unwrap_or_else(|err| {
            eprintln!("Failed to deserialize theme: {}", err);
            std::process::exit(1);
        });
        Ok(Self {
            background: inner
                .background
                .map(|s| {
                    Color::from_str(&s).ok_or_else(|| {
                        serde::de::Error::custom(format!(
                            "invalid color {}",
                            s
                        ))
                    })
                })
                .transpose()?,
            border_fg: Color::from_str(&inner.border_fg).ok_or_else(|| {
                serde::de::Error::custom(format!(
                    "invalid color {}",
                    &inner.border_fg
                ))
            })?,
            text_fg: Color::from_str(&inner.text_fg).ok_or_else(|| {
                serde::de::Error::custom(format!(
                    "invalid color {}",
                    &inner.text_fg
                ))
            })?,
            dimmed_text_fg: Color::from_str(&inner.dimmed_text_fg)
                .ok_or_else(|| {
                    serde::de::Error::custom(format!(
                        "invalid color {}",
                        &inner.dimmed_text_fg
                    ))
                })?,
            input_text_fg: Color::from_str(&inner.input_text_fg).ok_or_else(
                || {
                    serde::de::Error::custom(format!(
                        "invalid color {}",
                        &inner.input_text_fg
                    ))
                },
            )?,
            result_count_fg: Color::from_str(&inner.result_count_fg)
                .ok_or_else(|| {
                    serde::de::Error::custom(format!(
                        "invalid color {}",
                        &inner.result_count_fg
                    ))
                })?,
            result_name_fg: Color::from_str(&inner.result_name_fg)
                .ok_or_else(|| {
                    serde::de::Error::custom(format!(
                        "invalid color {}",
                        &inner.result_name_fg
                    ))
                })?,
            result_line_number_fg: Color::from_str(
                &inner.result_line_number_fg,
            )
            .ok_or_else(|| {
                serde::de::Error::custom(format!(
                    "invalid color {}",
                    &inner.result_line_number_fg
                ))
            })?,
            result_value_fg: Color::from_str(&inner.result_value_fg)
                .ok_or_else(|| {
                    serde::de::Error::custom(format!(
                        "invalid color {}",
                        &inner.result_value_fg
                    ))
                })?,
            selection_bg: Color::from_str(&inner.selection_bg).ok_or_else(
                || {
                    serde::de::Error::custom(format!(
                        "invalid color {}",
                        &inner.selection_bg
                    ))
                },
            )?,
            // this is optional for theme backwards compatibility and falls back to match_fg
            selection_fg: match inner.selection_fg {
                Some(s) => Color::from_str(&s).ok_or_else(|| {
                    serde::de::Error::custom(format!("invalid color {}", &s))
                })?,
                None => Color::from_str(&inner.match_fg).ok_or_else(|| {
                    serde::de::Error::custom(format!(
                        "invalid color {}",
                        &inner.match_fg
                    ))
                })?,
            },

            match_fg: Color::from_str(&inner.match_fg).ok_or_else(|| {
                serde::de::Error::custom(format!(
                    "invalid color {}",
                    &inner.match_fg
                ))
            })?,
            preview_title_fg: Color::from_str(&inner.preview_title_fg)
                .ok_or_else(|| {
                    serde::de::Error::custom(format!(
                        "invalid color {}",
                        &inner.preview_title_fg
                    ))
                })?,
            channel_mode_fg: Color::from_str(&inner.channel_mode_fg)
                .ok_or_else(|| {
                    serde::de::Error::custom(format!(
                        "invalid color {}",
                        &inner.channel_mode_fg
                    ))
                })?,
            channel_mode_bg: match inner.channel_mode_bg {
                Some(s) => Color::from_str(&s).ok_or_else(|| {
                    serde::de::Error::custom(format!("invalid color {}", &s))
                })?,
                // Default to black. Not sure if black is the best choice
                None => Color::Ansi(ANSIColor::Black),
            },
            remote_control_mode_fg: Color::from_str(
                &inner.remote_control_mode_fg,
            )
            .ok_or_else(|| {
                serde::de::Error::custom(format!(
                    "invalid color {}",
                    &inner.remote_control_mode_fg
                ))
            })?,
            remote_control_mode_bg: Color::from_str(
                &inner.remote_control_mode_bg,
            )
            .ok_or_else(|| {
                serde::de::Error::custom(format!(
                    "invalid color {}",
                    &inner.remote_control_mode_bg
                ))
            })?,
        })
    }
}

#[allow(clippy::from_over_into)]
impl Into<RatatuiColor> for &RGBColor {
    fn into(self) -> RatatuiColor {
        RatatuiColor::Rgb(self.r, self.g, self.b)
    }
}

#[allow(clippy::from_over_into)]
impl Into<RatatuiColor> for &ANSIColor {
    fn into(self) -> RatatuiColor {
        match self {
            ANSIColor::Black => RatatuiColor::Black,
            ANSIColor::Red => RatatuiColor::Red,
            ANSIColor::Green => RatatuiColor::Green,
            ANSIColor::Yellow => RatatuiColor::Yellow,
            ANSIColor::Blue => RatatuiColor::Blue,
            ANSIColor::Magenta => RatatuiColor::Magenta,
            ANSIColor::Cyan => RatatuiColor::Cyan,
            ANSIColor::White => RatatuiColor::Gray,
            ANSIColor::BrightBlack => RatatuiColor::DarkGray,
            ANSIColor::BrightRed => RatatuiColor::LightRed,
            ANSIColor::BrightGreen => RatatuiColor::LightGreen,
            ANSIColor::BrightYellow => RatatuiColor::LightYellow,
            ANSIColor::BrightBlue => RatatuiColor::LightBlue,
            ANSIColor::BrightMagenta => RatatuiColor::LightMagenta,
            ANSIColor::BrightCyan => RatatuiColor::LightCyan,
            ANSIColor::BrightWhite => RatatuiColor::White,
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<RatatuiColor> for &Color {
    fn into(self) -> RatatuiColor {
        match self {
            Color::Ansi(ansi) => ansi.into(),
            Color::Rgb(rgb) => rgb.into(),
        }
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
            background: self.background.as_ref().map(Into::into),
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
            result_fg: (&self.result_name_fg).into(),
            result_selected_bg: (&self.selection_bg).into(),
            result_selected_fg: (&self.selection_fg).into(),
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
            channel: (&self.channel_mode_bg).into(),
            channel_fg: (&self.channel_mode_fg).into(),
            remote_control: (&self.remote_control_mode_bg).into(),
            remote_control_fg: (&self.remote_control_mode_fg).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_theme() -> Theme {
        Theme {
            background: Some(Color::Ansi(ANSIColor::Black)),
            border_fg: Color::Ansi(ANSIColor::White),
            text_fg: Color::Ansi(ANSIColor::BrightWhite),
            dimmed_text_fg: Color::Ansi(ANSIColor::BrightBlack),
            input_text_fg: Color::Ansi(ANSIColor::BrightWhite),
            result_count_fg: Color::Ansi(ANSIColor::BrightWhite),
            result_name_fg: Color::Ansi(ANSIColor::BrightWhite),
            result_line_number_fg: Color::Ansi(ANSIColor::BrightWhite),
            result_value_fg: Color::Ansi(ANSIColor::BrightWhite),
            selection_bg: Color::Ansi(ANSIColor::BrightWhite),
            selection_fg: Color::Ansi(ANSIColor::BrightWhite),
            match_fg: Color::Ansi(ANSIColor::BrightWhite),
            preview_title_fg: Color::Ansi(ANSIColor::BrightWhite),
            channel_mode_fg: Color::Ansi(ANSIColor::BrightWhite),
            channel_mode_bg: Color::Ansi(ANSIColor::BrightBlack),
            remote_control_mode_fg: Color::Ansi(ANSIColor::BrightWhite),
            remote_control_mode_bg: Color::Ansi(ANSIColor::BrightBlack),
        }
    }

    #[test]
    fn test_theme_deserialization() {
        let theme_content = r##"
            background = "#000000"
            border_fg = "black"
            text_fg = "white"
            dimmed_text_fg = "bright-black"
            input_text_fg = "bright-white"
            result_count_fg = "bright-white"
            result_name_fg = "bright-white"
            result_line_number_fg = "bright-white"
            result_value_fg = "bright-white"
            selection_bg = "bright-white"
            selection_fg = "bright-white"
            match_fg = "bright-white"
            preview_title_fg = "bright-white"
            channel_mode_fg = "bright-white"
            channel_mode_bg = "bright-black"
            remote_control_mode_fg = "bright-white"
            remote_control_mode_bg = "bright-black"
        "##;
        let theme: Theme = toml::from_str(theme_content).unwrap();
        assert_eq!(
            theme.background,
            Some(Color::Rgb(RGBColor::from_str("000000").unwrap()))
        );
        assert_eq!(theme.border_fg, Color::Ansi(ANSIColor::Black));
        assert_eq!(theme.text_fg, Color::Ansi(ANSIColor::White));
        assert_eq!(theme.dimmed_text_fg, Color::Ansi(ANSIColor::BrightBlack));
        assert_eq!(theme.input_text_fg, Color::Ansi(ANSIColor::BrightWhite));
        assert_eq!(theme.result_count_fg, Color::Ansi(ANSIColor::BrightWhite));
        assert_eq!(theme.result_name_fg, Color::Ansi(ANSIColor::BrightWhite));
        assert_eq!(
            theme.result_line_number_fg,
            Color::Ansi(ANSIColor::BrightWhite)
        );
        assert_eq!(theme.result_value_fg, Color::Ansi(ANSIColor::BrightWhite));
        assert_eq!(theme.selection_bg, Color::Ansi(ANSIColor::BrightWhite));
        assert_eq!(theme.selection_fg, Color::Ansi(ANSIColor::BrightWhite));
        assert_eq!(theme.match_fg, Color::Ansi(ANSIColor::BrightWhite));
        assert_eq!(
            theme.preview_title_fg,
            Color::Ansi(ANSIColor::BrightWhite)
        );
        assert_eq!(theme.channel_mode_fg, Color::Ansi(ANSIColor::BrightWhite));
        assert_eq!(
            theme.remote_control_mode_fg,
            Color::Ansi(ANSIColor::BrightWhite)
        );
    }

    #[test]
    fn test_theme_deserialization_no_background() {
        let theme_content = r##"
            border_fg = "black"
            text_fg = "white"
            dimmed_text_fg = "bright-black"
            input_text_fg = "bright-white"
            result_count_fg = "#ffffff"
            result_name_fg = "bright-white"
            result_line_number_fg = "#ffffff"
            result_value_fg = "bright-white"
            selection_bg = "bright-white"
            selection_fg = "bright-white"
            match_fg = "bright-white"
            preview_title_fg = "bright-white"
            channel_mode_fg = "bright-white"
            channel_mode_bg = "bright-black"
            remote_control_mode_fg = "bright-white"
            remote_control_mode_bg = "bright-black"
        "##;
        let theme: Theme = toml::from_str(theme_content).unwrap();
        assert_eq!(theme.background, None);
        assert_eq!(theme.border_fg, Color::Ansi(ANSIColor::Black));
        assert_eq!(theme.text_fg, Color::Ansi(ANSIColor::White));
        assert_eq!(theme.dimmed_text_fg, Color::Ansi(ANSIColor::BrightBlack));
        assert_eq!(theme.input_text_fg, Color::Ansi(ANSIColor::BrightWhite));
        assert_eq!(
            theme.result_count_fg,
            Color::Rgb(RGBColor::from_str("ffffff").unwrap())
        );
        assert_eq!(theme.result_name_fg, Color::Ansi(ANSIColor::BrightWhite));
        assert_eq!(
            theme.result_line_number_fg,
            Color::Rgb(RGBColor::from_str("ffffff").unwrap())
        );
        assert_eq!(theme.result_value_fg, Color::Ansi(ANSIColor::BrightWhite));
        assert_eq!(theme.selection_bg, Color::Ansi(ANSIColor::BrightWhite));
        assert_eq!(theme.selection_fg, Color::Ansi(ANSIColor::BrightWhite));
        assert_eq!(theme.match_fg, Color::Ansi(ANSIColor::BrightWhite));
        assert_eq!(
            theme.preview_title_fg,
            Color::Ansi(ANSIColor::BrightWhite)
        );
        assert_eq!(theme.channel_mode_fg, Color::Ansi(ANSIColor::BrightWhite));
        assert_eq!(
            theme.remote_control_mode_fg,
            Color::Ansi(ANSIColor::BrightWhite)
        );
    }

    #[test]
    fn test_theme_merge_with_overrides() {
        let base_theme = create_test_theme();
        let overrides = crate::config::ui::ThemeOverrides {
            background: Some("#ff0000".to_string()),
            text_fg: Some("red".to_string()),
            selection_bg: Some("#00ff00".to_string()),
            ..Default::default()
        };

        let merged_theme =
            base_theme.merge_with_overrides(&overrides).unwrap();

        // Check that overridden colors are changed
        assert_eq!(
            merged_theme.background,
            Some(Color::Rgb(RGBColor::from_str("ff0000").unwrap()))
        );
        assert_eq!(merged_theme.text_fg, Color::Ansi(ANSIColor::Red));
        assert_eq!(
            merged_theme.selection_bg,
            Color::Rgb(RGBColor::from_str("00ff00").unwrap())
        );

        // Check that non-overridden colors remain the same
        assert_eq!(merged_theme.border_fg, Color::Ansi(ANSIColor::White));
        assert_eq!(
            merged_theme.input_text_fg,
            Color::Ansi(ANSIColor::BrightWhite)
        );
        assert_eq!(merged_theme.match_fg, Color::Ansi(ANSIColor::BrightWhite));
    }

    #[test]
    fn test_theme_merge_with_invalid_color() {
        let base_theme = create_test_theme();
        let overrides = crate::config::ui::ThemeOverrides {
            text_fg: Some("invalid-color".to_string()),
            ..Default::default()
        };

        let result = base_theme.merge_with_overrides(&overrides);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("invalid text_fg color")
        );
    }

    #[test]
    fn test_theme_merge_with_empty_overrides() {
        let base_theme = create_test_theme();
        let overrides = crate::config::ui::ThemeOverrides::default();

        let merged_theme =
            base_theme.merge_with_overrides(&overrides).unwrap();

        // Check that all colors remain the same when no overrides are provided
        assert_eq!(merged_theme.background, base_theme.background);
        assert_eq!(merged_theme.border_fg, base_theme.border_fg);
        assert_eq!(merged_theme.text_fg, base_theme.text_fg);
        assert_eq!(merged_theme.dimmed_text_fg, base_theme.dimmed_text_fg);
        assert_eq!(merged_theme.input_text_fg, base_theme.input_text_fg);
        assert_eq!(merged_theme.result_count_fg, base_theme.result_count_fg);
        assert_eq!(merged_theme.result_name_fg, base_theme.result_name_fg);
        assert_eq!(
            merged_theme.result_line_number_fg,
            base_theme.result_line_number_fg
        );
        assert_eq!(merged_theme.result_value_fg, base_theme.result_value_fg);
        assert_eq!(merged_theme.selection_bg, base_theme.selection_bg);
        assert_eq!(merged_theme.selection_fg, base_theme.selection_fg);
        assert_eq!(merged_theme.match_fg, base_theme.match_fg);
        assert_eq!(merged_theme.preview_title_fg, base_theme.preview_title_fg);
        assert_eq!(merged_theme.channel_mode_fg, base_theme.channel_mode_fg);
        assert_eq!(merged_theme.channel_mode_bg, base_theme.channel_mode_bg);
        assert_eq!(
            merged_theme.remote_control_mode_fg,
            base_theme.remote_control_mode_fg
        );
        assert_eq!(
            merged_theme.remote_control_mode_bg,
            base_theme.remote_control_mode_bg
        );
    }

    #[test]
    fn test_theme_deserialization_invalid_color() {
        let theme_content = r##"
            background = "#000000"
            border_fg = "invalid-color"
            text_fg = "white"
            dimmed_text_fg = "bright-black"
            input_text_fg = "bright-white"
            result_count_fg = "bright-white"
            result_name_fg = "bright-white"
            result_line_number_fg = "bright-white"
            result_value_fg = "bright-white"
            selection_bg = "bright-white"
            selection_fg = "bright-white"
            match_fg = "bright-white"
            preview_title_fg = "bright-white"
            channel_mode_fg = "bright-white"
            channel_mode_bg = "bright-black"
            remote_control_mode_fg = "bright-white"
            remote_control_mode_bg = "bright-black"
        "##;
        let result: Result<Theme, _> = toml::from_str(theme_content);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("invalid color invalid-color"));
        }
    }

    #[test]
    fn test_theme_deserialization_fallback_channel_mode_bg() {
        let theme_content = r##"
            background = "#000000"
            border_fg = "black"
            text_fg = "white"
            dimmed_text_fg = "bright-black"
            input_text_fg = "bright-white"
            result_count_fg = "bright-white"
            result_name_fg = "bright-white"
            result_line_number_fg = "bright-white"
            result_value_fg = "bright-white"
            selection_bg = "bright-white"
            selection_fg = "bright-white"
            match_fg = "bright-white"
            preview_title_fg = "bright-white"
            channel_mode_fg = "bright-white"
            remote_control_mode_fg = "bright-white"
            remote_control_mode_bg = "bright-black"
        "##;
        let theme: Theme = toml::from_str(theme_content).unwrap();
        assert_eq!(theme.channel_mode_bg, Color::Ansi(ANSIColor::Black));
    }
}
