use ratatui::layout;
use ratatui::layout::{Constraint, Direction, Rect};

pub(crate) struct Dimensions {
    pub x: u16,
    pub y: u16,
}

impl Dimensions {
    pub(crate) fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

impl Default for Dimensions {
    fn default() -> Self {
        Self::new(UI_WIDTH_PERCENT, UI_HEIGHT_PERCENT)
    }
}

pub(crate) struct Layout {
    pub results: Rect,
    pub input: Rect,
    pub preview_title: Option<Rect>,
    pub preview_window: Option<Rect>,
}

impl Layout {
    pub(crate) fn new(
        results: Rect,
        input: Rect,
        preview_title: Option<Rect>,
        preview_window: Option<Rect>,
    ) -> Self {
        Self {
            results,
            input,
            preview_title,
            preview_window,
        }
    }

    /// TODO: add diagram
    #[allow(dead_code)]
    pub(crate) fn all_panes_centered(
        dimensions: Dimensions,
        area: Rect,
    ) -> Self {
        let main_block = centered_rect(dimensions.x, dimensions.y, area);
        // split the main block into two vertical chunks
        let chunks = layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(main_block);

        // left block: results + input field
        let left_chunks = layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(3)])
            .split(chunks[0]);

        // right block: preview title + preview
        let right_chunks = layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(10)])
            .split(chunks[1]);

        Self::new(
            left_chunks[0],
            left_chunks[1],
            Some(right_chunks[0]),
            Some(right_chunks[1]),
        )
    }

    /// TODO: add diagram
    #[allow(dead_code)]
    pub(crate) fn results_only_centered(
        dimensions: Dimensions,
        area: Rect,
    ) -> Self {
        let main_block = centered_rect(dimensions.x, dimensions.y, area);
        // split the main block into two vertical chunks
        let chunks = layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(3)])
            .split(main_block);

        Self::new(chunks[0], chunks[1], None, None)
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
const UI_WIDTH_PERCENT: u16 = 90;
const UI_HEIGHT_PERCENT: u16 = 90;
