pub const POINTER_SYMBOL: &str = "> ";
pub const SELECTED_SYMBOL: &str = "● ";
pub const DESELECTED_SYMBOL: &str = "  ";
pub const LOGO_WIDTH: u16 = 24;

/// Thin hairline border set used by the minimal UI separators.
///
/// Eighth-block characters are an eighth of the cell *height* horizontally,
/// which renders visibly thicker than the eighth of the cell *width* used
/// vertically, so horizontal separators use the light box-drawing line
/// instead.
pub const HAIRLINE_BORDER_SET: ratatui::symbols::border::Set =
    ratatui::symbols::border::Set {
        top_left: "┌",
        top_right: "┐",
        bottom_left: "└",
        bottom_right: "┘",
        vertical_left: "▏",
        vertical_right: "▕",
        horizontal_top: "─",
        horizontal_bottom: "─",
    };
