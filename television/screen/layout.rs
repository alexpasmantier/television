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

        // Define the constraints for the results area (results list + input bar).
        // We keep this near the top so we can derive the input-bar height before
        // calculating the preview/results split.
        let results_constraints =
            vec![Constraint::Min(3), Constraint::Length(3)];

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
            let raw_preview_percentage = ui_config
                .preview_size
                .get(ui_config.current_preview_size_idx)
                .copied()
                .unwrap_or(50)
                .clamp(1, 99); // ensure sane value

            // In portrait orientation, reserve the input bar height from the total
            // vertical space before applying the percentage split so the preview
            // takes the intended share of *usable* height.
            let mut preview_percentage = raw_preview_percentage;
            if ui_config.orientation == Orientation::Portrait
                && input_bar_height > 0
            {
                let total_height = remote_chunks[0].height;
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

            match (ui_config.orientation, ui_config.input_bar_position) {
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

        let main_chunks = layout::Layout::default()
            .direction(match ui_config.orientation {
                Orientation::Portrait => Direction::Vertical,
                Orientation::Landscape => Direction::Horizontal,
            })
            .constraints(constraints)
            .split(remote_chunks[0]);

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
                    .constraints(match ui_config.input_bar_position {
                        InputPosition::Top => results_constraints
                            .clone()
                            .into_iter()
                            .rev()
                            .collect(),
                        InputPosition::Bottom => results_constraints.clone(),
                    })
                    .split(result_window);

                let (input_rect, results_rect) = match ui_config
                    .input_bar_position
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

                match ui_config.input_bar_position {
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
                    let preview_pct = ui_config
                        .preview_size
                        .get(ui_config.current_preview_size_idx)
                        .copied()
                        .unwrap_or(50)
                        .clamp(1, 99);

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
                let port_chunks = layout::Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(portrait_constraints)
                    .split(remote_chunks[0]);

                let input_rect = port_chunks[input_idx];
                let results_rect = port_chunks[results_idx];
                let preview_rect = preview_idx.map(|idx| port_chunks[idx]);

                (input_rect, results_rect, preview_rect)
            }
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
