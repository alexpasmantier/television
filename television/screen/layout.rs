use crate::{
    config::{
        Config, UiConfig,
        ui::{BorderType, InputBarConfig},
    },
    features::FeatureFlags,
    screen::{
        colors::Colorscheme, help_panel::calculate_help_panel_size,
        logo::REMOTE_LOGO_HEIGHT_U16,
    },
    television::Mode,
};
use clap::ValueEnum;
use ratatui::layout::{
    self, Constraint, Direction, Layout as RatatuiLayout, Rect,
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

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
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    Default,
    PartialEq,
    Hash,
    ValueEnum,
)]
pub enum Orientation {
    #[serde(rename = "landscape")]
    #[default]
    Landscape,
    #[serde(rename = "portrait")]
    Portrait,
}

impl From<crate::cli::args::LayoutOrientation> for Orientation {
    fn from(value: crate::cli::args::LayoutOrientation) -> Self {
        match value {
            crate::cli::args::LayoutOrientation::Landscape => {
                Orientation::Landscape
            }
            crate::cli::args::LayoutOrientation::Portrait => {
                Orientation::Portrait
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub results: Rect,
    pub input: Rect,
    pub preview_window: Option<Rect>,
    pub remote_control: Option<Rect>,
    pub help_panel: Option<Rect>,
    pub status_bar: Option<Rect>,
}

const REMOTE_PANEL_WIDTH_PERCENTAGE: u16 = 62;

impl Default for Layout {
    /// Having a default layout with a non-zero height for the results area
    /// is important for the initial rendering of the application. For the first
    /// frame, this avoids not rendering any results at all since the picker's contents
    /// depend on the height of the results area which is not known until the first
    /// frame is rendered.
    fn default() -> Self {
        Self::new(
            Rect::new(0, 0, 0, 100),
            Rect::default(),
            None,
            None,
            None,
            None,
        )
    }
}

impl Layout {
    pub fn new(
        results: Rect,
        input: Rect,
        preview_window: Option<Rect>,
        remote_control: Option<Rect>,
        help_panel: Option<Rect>,
        status_bar: Option<Rect>,
    ) -> Self {
        Self {
            results,
            input,
            preview_window,
            remote_control,
            help_panel,
            status_bar,
        }
    }

    pub fn build(
        area: Rect,
        ui_config: &UiConfig,
        show_remote: bool,
        show_preview: bool,
        config: Option<&Config>,
        mode: Mode,
        colorscheme: &Colorscheme,
    ) -> Self {
        let show_preview = show_preview
            && ui_config.features.is_visible(FeatureFlags::PreviewPanel);
        let dimensions = Dimensions::from(ui_config.ui_scale);

        // Reserve space for status bar if enabled
        let working_area =
            if ui_config.features.is_visible(FeatureFlags::StatusBar) {
                Rect {
                    x: area.x,
                    y: area.y,
                    width: area.width,
                    height: area.height.saturating_sub(1), // Reserve exactly 1 line for status bar
                }
            } else {
                area
            };

        let main_block =
            centered_rect(dimensions.x, dimensions.y, working_area);

        // Use the entire main block since help bar is removed
        let main_rect = main_block;

        // Define the constraints for the results area (results list + input bar).
        // We keep this near the top so we can derive the input-bar height before
        // calculating the preview/results split.
        let results_constraints = vec![
            Constraint::Min(3),
            Constraint::Length(input_bar_height(&ui_config.input_bar)),
        ];

        // Extract the explicit height of the input bar from the vector above so
        // the value stays in sync if the constraint is ever changed elsewhere.
        let input_bar_height: u16 = results_constraints
            .iter()
            .find_map(|c| match c {
                Constraint::Length(h) => Some(*h),
                _ => None,
            })
            .unwrap_or(0);

        // split the main block into 1 or 2 chunks (results + preview)
        let constraints = if show_preview {
            // Determine the desired preview percentage (as configured by the user)
            let raw_preview_percentage =
                ui_config.preview_panel.size.clamp(1, 99); // ensure sane value

            // In portrait orientation, reserve the input bar height from the total
            // vertical space before applying the percentage split so the preview
            // takes the intended share of *usable* height.
            let mut preview_percentage = raw_preview_percentage;
            if ui_config.orientation == Orientation::Portrait
                && input_bar_height > 0
            {
                let total_height = main_rect.height;
                if total_height > input_bar_height {
                    let available_height = total_height - input_bar_height;
                    preview_percentage = raw_preview_percentage
                        * available_height
                        / total_height;

                    preview_percentage = preview_percentage.clamp(1, 99);
                }
            }

            // results percentage is whatever remains
            let results_percentage = 100u16.saturating_sub(preview_percentage);

            match (ui_config.orientation, ui_config.input_bar.position) {
                // Preview is rendered on the right or bottom depending on orientation
                (Orientation::Landscape, _)
                | (Orientation::Portrait, InputPosition::Top) => {
                    vec![
                        Constraint::Percentage(results_percentage),
                        Constraint::Percentage(preview_percentage),
                    ]
                }
                // In portrait orientation with the input bar at the bottom, the preview
                // is rendered at the top. Swap the constraints so that the preview comes first.
                (Orientation::Portrait, InputPosition::Bottom) => vec![
                    Constraint::Percentage(preview_percentage),
                    Constraint::Percentage(results_percentage),
                ],
            }
        } else {
            vec![Constraint::Fill(1)]
        };

        let main_chunks = RatatuiLayout::default()
            .direction(match ui_config.orientation {
                Orientation::Portrait => Direction::Vertical,
                Orientation::Landscape => Direction::Horizontal,
            })
            .constraints(constraints)
            .split(main_rect);

        // ------------------------------------------------------------------
        // Determine the rectangles for input, results list and optional preview
        // ------------------------------------------------------------------

        let (input, results, preview_window) = match ui_config.orientation {
            Orientation::Landscape => {
                // Landscape keeps the old behaviour: horizontally split results+input
                // on the left and preview (if any) on the right. We still need to
                // carve out the input bar inside the results area.

                // First, decide which chunk is results vs preview based on the
                // earlier `main_chunks` computation.
                let (result_window, preview_window) = if show_preview {
                    (main_chunks[0], Some(main_chunks[1]))
                } else {
                    (main_chunks[0], None)
                };

                // Now split the results window vertically into results list + input
                let result_chunks = layout::Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(match ui_config.input_bar.position {
                        InputPosition::Top => results_constraints
                            .clone()
                            .into_iter()
                            .rev()
                            .collect(),
                        InputPosition::Bottom => results_constraints.clone(),
                    })
                    .split(result_window);

                let (input_rect, results_rect) = match ui_config
                    .input_bar
                    .position
                {
                    InputPosition::Bottom => {
                        (result_chunks[1], result_chunks[0])
                    }
                    InputPosition::Top => (result_chunks[0], result_chunks[1]),
                };

                (input_rect, results_rect, preview_window)
            }
            Orientation::Portrait => {
                // Portrait: build everything (preview, results, input) in a single
                // vertical split so percentages are applied correctly without the
                // need for post-hoc adjustments.

                // Index helpers
                let (input_idx, results_idx, preview_idx): (
                    usize,
                    usize,
                    Option<usize>,
                );

                let mut portrait_constraints: Vec<Constraint> = Vec::new();

                match ui_config.input_bar.position {
                    InputPosition::Top => {
                        // Input bar is always the first chunk
                        portrait_constraints
                            .push(Constraint::Length(input_bar_height));
                        input_idx = 0;

                        if show_preview {
                            // results then preview
                            portrait_constraints
                                .push(Constraint::Percentage(100));
                            portrait_constraints
                                .push(Constraint::Percentage(0));
                            results_idx = 1;
                            preview_idx = Some(2);
                        } else {
                            // only results
                            portrait_constraints.push(Constraint::Fill(1));
                            results_idx = 1;
                            preview_idx = None;
                        }
                    }
                    InputPosition::Bottom => {
                        // For bottom input bar we might put preview at the top if
                        // present, then results, then input.
                        if show_preview {
                            portrait_constraints
                                .push(Constraint::Percentage(0));
                            preview_idx = Some(0);
                        } else {
                            preview_idx = None;
                        }

                        // results (placeholder percentage)
                        portrait_constraints.push(Constraint::Percentage(100));
                        results_idx = usize::from(show_preview);

                        // finally the input bar
                        portrait_constraints
                            .push(Constraint::Length(input_bar_height));
                        input_idx = portrait_constraints.len() - 1;
                    }
                }

                // If preview is enabled, calculate the concrete percentages now
                if let Some(p_idx) = preview_idx {
                    // Determine preview percentage from config
                    let preview_pct =
                        ui_config.preview_panel.size.clamp(1, 99);

                    // Remaining for results
                    let results_pct = 100u16.saturating_sub(preview_pct);

                    // Assign
                    portrait_constraints[results_idx] =
                        Constraint::Percentage(results_pct);
                    portrait_constraints[p_idx] =
                        Constraint::Percentage(preview_pct);
                } else {
                    // preview disabled: results takes the remaining space
                    portrait_constraints[results_idx] = Constraint::Fill(1);
                }

                // Perform the split now that we have the final constraints vector
                let portrait_chunks = layout::Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(portrait_constraints)
                    .split(main_rect);

                let input_rect = portrait_chunks[input_idx];
                let results_rect = portrait_chunks[results_idx];
                let preview_rect = preview_idx.map(|idx| portrait_chunks[idx]);

                (input_rect, results_rect, preview_rect)
            }
        };

        // the remote control is a centered popup
        let remote_control = if show_remote {
            let remote_control_rect = centered_rect_with_dimensions(
                &Dimensions::new(
                    area.width * REMOTE_PANEL_WIDTH_PERCENTAGE / 100,
                    REMOTE_LOGO_HEIGHT_U16,
                ),
                area,
            );
            Some(remote_control_rect)
        } else {
            None
        };

        // the help panel is positioned at bottom-right, accounting for status bar
        let help_panel = if ui_config
            .features
            .is_visible(FeatureFlags::HelpPanel)
        {
            // Calculate available area for help panel (excluding status bar if enabled)
            let hp_area =
                if ui_config.features.is_visible(FeatureFlags::StatusBar) {
                    Rect {
                        x: area.x,
                        y: area.y,
                        width: area.width,
                        height: area.height.saturating_sub(1), // Account for single line status bar
                    }
                } else {
                    area
                };

            if let Some(cfg) = config {
                let (width, height) =
                    calculate_help_panel_size(cfg, mode, colorscheme);
                Some(bottom_right_rect(width, height, hp_area))
            } else {
                // Fallback to reasonable default if config not available
                Some(bottom_right_rect(45, 25, hp_area))
            }
        } else {
            None
        };

        // Create status bar at the bottom if enabled
        let status_bar =
            if ui_config.features.is_visible(FeatureFlags::StatusBar) {
                Some(Rect {
                    x: area.x,
                    y: area.y + area.height - 1, // Position at the very last line
                    width: area.width,
                    height: 1, // Single line status bar
                })
            } else {
                None
            };

        Self::new(
            results,
            input,
            preview_window,
            remote_control,
            help_panel,
            status_bar,
        )
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let height = r.height.saturating_mul(percent_y) / 100;
    let width = r.width.saturating_mul(percent_x) / 100;

    centered_rect_with_dimensions(&Dimensions::new(width, height), r)
}

fn centered_rect_with_dimensions(dimensions: &Dimensions, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = layout::Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(dimensions.y),
            Constraint::Fill(1),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    layout::Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(dimensions.x),
            Constraint::Fill(1),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

/// helper function to create a floating rect positioned at the bottom-right corner
fn bottom_right_rect(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.width.saturating_sub(width + 2); // 2 for padding from edge
    let y = r.height.saturating_sub(height + 1); // 1 for padding from edge

    Rect {
        x: r.x + x,
        y: r.y + y,
        width: width.min(r.width.saturating_sub(2)),
        height: height.min(r.height.saturating_sub(2)),
    }
}

fn input_bar_height(input_bar_config: &InputBarConfig) -> u16 {
    // input line + header + vertical padding
    let mut h =
        1 + 1 + input_bar_config.padding.top + input_bar_config.padding.bottom;

    // add the bottom border if applicable (top is already included with the header)
    if input_bar_config.border_type != BorderType::None {
        h += 1;
    }
    h
}

#[cfg(test)]
mod tests {
    use crate::config::ui::Padding;

    use super::*;

    #[test]
    /// ---h----
    ///  input
    /// --------
    fn test_input_bar_height_top_with_borders() {
        let config = InputBarConfig {
            position: InputPosition::Top,
            padding: Padding::default(),
            border_type: BorderType::Rounded,
            header: None,
            prompt: "> ".to_string(),
        };
        assert_eq!(input_bar_height(&config), 3);
    }

    #[test]
    fn test_input_bar_height_bottom_with_borders() {
        let config = InputBarConfig {
            position: InputPosition::Bottom,
            padding: Padding::default(),
            border_type: BorderType::Rounded,
            header: None,
            prompt: "> ".to_string(),
        };
        assert_eq!(input_bar_height(&config), 3);
    }

    #[test]
    ///      h
    ///    input
    fn test_input_bar_height_top_without_borders() {
        let config = InputBarConfig {
            position: InputPosition::Top,
            padding: Padding::default(),
            border_type: BorderType::None,
            header: None,
            prompt: "> ".to_string(),
        };
        assert_eq!(input_bar_height(&config), 2);
    }

    #[test]
    ///  input
    ///    h
    fn test_input_bar_height_bottom_without_borders() {
        let config = InputBarConfig {
            position: InputPosition::Bottom,
            padding: Padding::default(),
            border_type: BorderType::None,
            header: None,
            prompt: "> ".to_string(),
        };
        assert_eq!(input_bar_height(&config), 2);
    }

    #[test]
    fn test_input_bar_height_with_padding() {
        let config = InputBarConfig {
            position: InputPosition::Top,
            padding: Padding {
                top: 1,
                bottom: 2,
                left: 0,
                right: 0,
            },
            border_type: BorderType::None,
            header: None,
            prompt: "> ".to_string(),
        };
        assert_eq!(input_bar_height(&config), 5);
    }
}
