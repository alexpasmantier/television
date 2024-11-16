use ratatui::layout;
use ratatui::layout::{Constraint, Direction, Rect};

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

#[derive(Debug, Clone, Copy)]
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

pub struct Layout {
    pub help_bar: Option<HelpBarLayout>,
    pub results: Rect,
    pub input: Rect,
    pub preview_title: Rect,
    pub preview_window: Rect,
    pub remote_control: Option<Rect>,
}

impl Layout {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        help_bar: Option<HelpBarLayout>,
        results: Rect,
        input: Rect,
        preview_title: Rect,
        preview_window: Rect,
        remote_control: Option<Rect>,
    ) -> Self {
        Self {
            help_bar,
            results,
            input,
            preview_title,
            preview_window,
            remote_control,
        }
    }

    pub fn build(
        dimensions: &Dimensions,
        area: Rect,
        with_remote: bool,
        with_help_bar: bool,
    ) -> Self {
        let main_block = centered_rect(dimensions.x, dimensions.y, area);
        // split the main block into two vertical chunks (help bar + rest)
        let main_rect: Rect;
        let help_bar_layout: Option<HelpBarLayout>;

        if with_help_bar {
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

        // split the main block into two vertical chunks
        let constraints = if with_remote {
            vec![
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Length(24),
            ]
        } else {
            vec![Constraint::Percentage(50), Constraint::Percentage(50)]
        };
        let vt_chunks = layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(main_rect);

        // left block: results + input field
        let left_chunks = layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(vt_chunks[0]);

        // right block: preview title + preview
        let right_chunks = layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(3)])
            .split(vt_chunks[1]);

        Self::new(
            help_bar_layout,
            left_chunks[0],
            left_chunks[1],
            right_chunks[0],
            right_chunks[1],
            if with_remote {
                Some(vt_chunks[2])
            } else {
                None
            },
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
