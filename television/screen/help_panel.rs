use crate::{
    config::KeyBindings,
    screen::colors::Colorscheme,
    screen::keybindings::{ActionMapping, find_keys_for_single_action},
    television::Mode,
};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
};

const MIN_PANEL_WIDTH: u16 = 25;
const MIN_PANEL_HEIGHT: u16 = 5;

/// Draws a Helix-style floating help panel in the bottom-right corner
pub fn draw_help_panel(
    f: &mut Frame<'_>,
    area: Rect,
    keybindings: &KeyBindings,
    tv_mode: Mode,
    colorscheme: &Colorscheme,
) {
    if area.width < MIN_PANEL_WIDTH || area.height < MIN_PANEL_HEIGHT {
        return; // Too small to display anything meaningful
    }

    // Generate content
    let content = generate_help_content(keybindings, tv_mode, colorscheme);

    // Clear the area first to create the floating effect
    f.render_widget(Clear, area);

    // Create the main block with consistent styling
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colorscheme.general.border_fg))
        .title_top(Line::from(" Help ").alignment(Alignment::Center))
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        )
        .padding(Padding::horizontal(1));

    let paragraph = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

/// Adds keybinding lines for action mappings to the given lines vector
fn add_keybinding_lines_for_mappings(
    lines: &mut Vec<Line<'static>>,
    keybindings: &KeyBindings,
    mappings: &[ActionMapping],
    mode: Mode,
    colorscheme: &Colorscheme,
) {
    for mapping in mappings {
        for (action, description) in &mapping.actions {
            let keys = find_keys_for_single_action(keybindings, action);
            for key in keys {
                lines.push(create_compact_keybinding_line(
                    &key,
                    description,
                    mode,
                    colorscheme,
                ));
            }
        }
    }
}

/// Generates the help content organized into global and mode-specific groups
fn generate_help_content(
    keybindings: &KeyBindings,
    mode: Mode,
    colorscheme: &Colorscheme,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    // Global keybindings section header
    lines.push(Line::from(vec![Span::styled(
        "Global",
        Style::default()
            .fg(colorscheme.help.metadata_field_name_fg)
            .bold()
            .underlined(),
    )]));

    // Global actions using centralized system
    let global_mappings = ActionMapping::global_actions();
    add_keybinding_lines_for_mappings(
        &mut lines,
        keybindings,
        &global_mappings,
        mode,
        colorscheme,
    );

    // Add spacing between Global and mode-specific sections
    lines.push(Line::from(""));

    // Mode-specific keybindings section header
    let mode_name = match mode {
        Mode::Channel => "Channel",
        Mode::RemoteControl => "Remote",
    };

    lines.push(Line::from(vec![Span::styled(
        mode_name,
        Style::default()
            .fg(colorscheme.help.metadata_field_name_fg)
            .bold()
            .underlined(),
    )]));

    // Navigation actions (common to both modes) using centralized system
    let nav_mappings = ActionMapping::navigation_actions();
    add_keybinding_lines_for_mappings(
        &mut lines,
        keybindings,
        &nav_mappings,
        mode,
        colorscheme,
    );

    // Mode-specific actions using centralized system
    let mode_mappings = ActionMapping::mode_specific_actions(mode);
    add_keybinding_lines_for_mappings(
        &mut lines,
        keybindings,
        &mode_mappings,
        mode,
        colorscheme,
    );

    lines
}

/// Creates a compact keybinding line with one space of left padding
fn create_compact_keybinding_line(
    key: &str,
    action: &str,
    mode: Mode,
    colorscheme: &Colorscheme,
) -> Line<'static> {
    // Use the appropriate mode color
    let key_color = match mode {
        Mode::Channel => colorscheme.mode.channel,
        Mode::RemoteControl => colorscheme.mode.remote_control,
    };

    Line::from(vec![
        Span::styled(
            format!("{}:", action),
            Style::default().fg(colorscheme.help.metadata_field_name_fg),
        ),
        Span::raw(" "), // Space between action and key
        Span::styled(key.to_string(), Style::default().fg(key_color).bold()),
    ])
}

/// Calculates the required dimensions for the help panel based on content
#[allow(clippy::cast_possible_truncation)]
pub fn calculate_help_panel_size(
    keybindings: &KeyBindings,
    mode: Mode,
    colorscheme: &Colorscheme,
) -> (u16, u16) {
    // Generate content to count items and calculate width
    let content = generate_help_content(keybindings, mode, colorscheme);

    // Calculate required width based on actual content
    let max_content_width = content
        .iter()
        .map(Line::width) // Use Line's width method for sizing calculation
        .max()
        .unwrap_or(25);

    // Calculate dimensions with proper padding:
    // - Width: content + 3 (2 borders + 1 padding)
    // - Height: content lines + 2 (2 borders, no title or padding)
    let required_width = (max_content_width + 3).max(25) as u16;
    let required_height = (content.len() + 2).max(8) as u16;

    (required_width, required_height)
}
