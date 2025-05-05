use std::fmt::Display;

use ratatui::layout;
use ratatui::layout::{Constraint, Direction, Rect};
use serde::{Deserialize, Serialize};

use crate::config::UiConfig;

pub struct Dimensions {
    pub x: u16,
    pub y: u16,
}

impl Dimensions {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

impl From<u16> for Dimensions {
    fn from(x: u16) -> Self {
        Self::new(x, x)
    }
}

impl Default for Dimensions {
    fn default() -> Self {
        Self::new(UI_WIDTH_PERCENT, UI_HEIGHT_PERCENT)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HelpBarLayout {
    pub left: Rect,
    pub middle: Rect,
    pub right: Rect,
}

impl HelpBarLayout {
    pub fn new(left: Rect, middle: Rect, right: Rect) -> Self {
        Self {
            left,
            middle,
            right,
        }
    }
}

#[derive(
    Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Hash,
)]
pub enum InputPosition {
    #[serde(rename = "top")]
    #[default]
    Top,
    #[serde(rename = "bottom")]
    Bottom,
}

impl Display for InputPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputPosition::Top => write!(f, "top"),
            InputPosition::Bottom => write!(f, "bottom"),
        }
    }
}

#[derive(
    Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Hash,
)]
pub enum Orientation {
    #[serde(rename = "landscape")]
    #[default]
    Landscape,
    #[serde(rename = "portrait")]
    Portrait,
}

#[derive(
    Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Hash,
)]
pub enum PreviewTitlePosition {
    #[serde(rename = "top")]
    #[default]
    Top,
    #[serde(rename = "bottom")]
    Bottom,
}

impl Display for PreviewTitlePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PreviewTitlePosition::Top => write!(f, "top"),
            PreviewTitlePosition::Bottom => write!(f, "bottom"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub help_bar: Option<HelpBarLayout>,
    pub results: Rect,
    pub input: Rect,
    pub preview_window: Option<Rect>,
    pub remote_control: Option<Rect>,
}

impl Default for Layout {
    /// Having a default layout with a non-zero height for the results area
    /// is important for the initial rendering of the application. For the first
    /// frame, this avoids not rendering any results at all since the picker's contents
    /// depend on the height of the results area which is not known until the first
    /// frame is rendered.
    fn default() -> Self {
        Self::new(None, Rect::new(0, 0, 0, 100), Rect::default(), None, None)
    }
}

impl Layout {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        help_bar: Option<HelpBarLayout>,
        results: Rect,
        input: Rect,
        preview_window: Option<Rect>,
        remote_control: Option<Rect>,
    ) -> Self {
        Self {
            help_bar,
            results,
            input,
            preview_window,
            remote_control,
        }
    }

    pub fn build(
        area: Rect,
        ui_config: &UiConfig,
        show_remote: bool,
        show_preview: bool,
        //
    ) -> Self {
        let show_preview = show_preview && ui_config.show_preview_panel;
        let dimensions = Dimensions::from(ui_config.ui_scale);
        let main_block = centered_rect(dimensions.x, dimensions.y, area);
        // split the main block into two vertical chunks (help bar + rest)
        let main_rect: Rect;
        let help_bar_layout: Option<HelpBarLayout>;

        if ui_config.show_help_bar {
            let hz_chunks = layout::Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Max(9), Constraint::Fill(1)])
                .split(main_block);
            main_rect = hz_chunks[1];

            // split the help bar into three horizontal chunks (left + center + right)
            let help_bar_chunks = layout::Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    // metadata
                    Constraint::Fill(1),
                    // keymaps
                    Constraint::Fill(1),
                    // logo
                    Constraint::Length(24),
                ])
                .split(hz_chunks[0]);

            help_bar_layout = Some(HelpBarLayout {
                left: help_bar_chunks[0],
                middle: help_bar_chunks[1],
                right: help_bar_chunks[2],
            });
        } else {
            main_rect = main_block;
            help_bar_layout = None;
        }

        let remote_constraints = if show_remote {
            vec![Constraint::Fill(1), Constraint::Length(24)]
        } else {
            vec![Constraint::Fill(1)]
        };
        let remote_chunks = layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints(remote_constraints)
            .split(main_rect);

        let remote_control = if show_remote {
            Some(remote_chunks[1])
        } else {
            None
        };

        // split the main block into 1 or 2 chunks
        // (results + preview)
        let constraints = if show_preview {
            vec![Constraint::Fill(1), Constraint::Fill(1)]
        } else {
            vec![Constraint::Fill(1)]
        };

        let main_chunks = layout::Layout::default()
            .direction(match ui_config.orientation {
                Orientation::Portrait => Direction::Vertical,
                Orientation::Landscape => Direction::Horizontal,
            })
            .constraints(constraints)
            .split(remote_chunks[0]);

        // result block: results + input field
        let results_constraints =
            vec![Constraint::Min(3), Constraint::Length(3)];

        let (result_window, preview_window) = if show_preview {
            match (ui_config.orientation, ui_config.input_bar_position) {
                (Orientation::Landscape, _)
                | (Orientation::Portrait, InputPosition::Top) => {
                    (main_chunks[0], Some(main_chunks[1]))
                }
                (Orientation::Portrait, InputPosition::Bottom) => {
                    (main_chunks[1], Some(main_chunks[0]))
                }
            }
        } else {
            (main_chunks[0], None)
        };

        let result_chunks = layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints(match ui_config.input_bar_position {
                InputPosition::Top => {
                    results_constraints.into_iter().rev().collect()
                }
                InputPosition::Bottom => results_constraints,
            })
            .split(result_window);
        let (input, results) = match ui_config.input_bar_position {
            InputPosition::Bottom => (result_chunks[1], result_chunks[0]),
            InputPosition::Top => (result_chunks[0], result_chunks[1]),
        };

        Self::new(
            help_bar_layout,
            results,
            input,
            preview_window,
            remote_control,
        )
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = layout::Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    layout::Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

// UI size
const UI_WIDTH_PERCENT: u16 = 95;
const UI_HEIGHT_PERCENT: u16 = 95;
