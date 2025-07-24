use crate::{
    action::Action,
    config::{Config, KeyBindings},
    event::Key,
    screen::colors::Colorscheme,
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
    config: &Config,
    tv_mode: Mode,
    colorscheme: &Colorscheme,
) {
    if area.width < MIN_PANEL_WIDTH || area.height < MIN_PANEL_HEIGHT {
        return; // Too small to display anything meaningful
    }

    // Generate content
    let content = generate_help_content(config, tv_mode, colorscheme);

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

/// Adds keybinding lines for specific keys to the given lines vector
fn add_keybinding_lines_for_keys(
    lines: &mut Vec<Line<'static>>,
    keybindings: &KeyBindings,
    keys: impl Iterator<Item = Key>,
    mode: Mode,
    colorscheme: &Colorscheme,
    category_name: &str,
) {
    use tracing::trace;

    // Collect all valid keybinding entries
    let mut entries: Vec<(String, String)> = Vec::new();
    let mut filtered_count = 0;

    for key in keys {
        if let Some(actions) = keybindings.get(&key) {
            for action in actions.as_slice() {
                // Filter out NoOp actions (unbound keys)
                if matches!(action, Action::NoOp) {
                    trace!(
                        "Filtering out NoOp (unboud keys) action for key '{}' in {} category",
                        key, category_name
                    );
                    filtered_count += 1;
                    continue;
                }

                let description = action.description();
                let key_string = key.to_string();
                entries.push((description.to_string(), key_string.clone()));
                trace!(
                    "Added keybinding: {} -> {} ({})",
                    key_string, description, category_name
                );
            }
        }
    }

    // Sort entries alphabetically by description
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    trace!(
        "Sorted {} keybindings for {} category (filtered {} NoOp entries)",
        entries.len(),
        category_name,
        filtered_count
    );

    // Create lines from sorted entries
    for (description, key_string) in entries {
        lines.push(create_compact_keybinding_line(
            &key_string,
            &description,
            mode,
            colorscheme,
        ));
    }
}

/// Generates the help content organized into global and channel-specific groups
fn generate_help_content(
    config: &Config,
    mode: Mode,
    colorscheme: &Colorscheme,
) -> Vec<Line<'static>> {
    use tracing::debug;

    let mut lines = Vec::new();

    debug!("Generating help content for mode: {:?}", mode);
    debug!(
        "Keybinding source tracking - Global keys: {}, Channel keys: {}",
        config.keybinding_source.global_keys.len(),
        config.keybinding_source.channel_keys.len()
    );

    // Global keybindings section header
    lines.push(Line::from(vec![Span::styled(
        "Global",
        Style::default()
            .fg(colorscheme.help.metadata_field_name_fg)
            .bold()
            .underlined(),
    )]));

    // Global keybindings - all keys that are NOT from channel configs
    let global_keys = config.keybinding_source.global_keys.iter().copied();
    add_keybinding_lines_for_keys(
        &mut lines,
        &config.keybindings,
        global_keys,
        mode,
        colorscheme,
        "Global",
    );

    // Add spacing between Global and channel-specific sections only if we have channel keys
    if !config.keybinding_source.channel_keys.is_empty() {
        lines.push(Line::from(""));
    }

    // Channel-specific keybindings section header
    let mode_name = match mode {
        Mode::Channel => "Channel",
        Mode::RemoteControl => "Remote",
    };

    // Only show channel section if there are channel-specific keybindings
    if config.keybinding_source.has_channel_keys() {
        lines.push(Line::from(vec![Span::styled(
            mode_name,
            Style::default()
                .fg(colorscheme.help.metadata_field_name_fg)
                .bold()
                .underlined(),
        )]));

        // Channel-specific keybindings - only keys from channel configs
        let channel_keys =
            config.keybinding_source.channel_keys.iter().copied();
        add_keybinding_lines_for_keys(
            &mut lines,
            &config.keybindings,
            channel_keys,
            mode,
            colorscheme,
            mode_name,
        );
    } else {
        debug!(
            "No channel-specific keybindings found, skipping channel section for mode: {:?}",
            mode
        );
    }

    // Handle edge case where no keybindings are found at all
    if !config.keybinding_source.has_any_keys() {
        debug!("Warning: No keybindings found in source tracking!");
        lines.push(Line::from(vec![Span::styled(
            "No keybindings configured",
            Style::default()
                .fg(colorscheme.help.metadata_field_name_fg)
                .italic(),
        )]));
    }

    debug!("Generated help content with {} total lines", lines.len());
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
    config: &Config,
    mode: Mode,
    colorscheme: &Colorscheme,
) -> (u16, u16) {
    use tracing::trace;

    // Generate content to count items and calculate width
    let content = generate_help_content(config, mode, colorscheme);

    // Calculate required width based on actual content
    let max_content_width = content
        .iter()
        .map(Line::width) // Use Line's width method for sizing calculation
        .max()
        .unwrap_or(25);

    // Calculate dimensions with proper padding:
    // - Width: content + 4 (2 borders + 2 padding)
    // - Height: content lines + 2 (2 borders, no title or padding)
    let required_width = (max_content_width + 4).max(25) as u16;
    let required_height = (content.len() + 2).max(8) as u16;

    trace!(
        "Help panel size calculation: {} lines, max width {}, final dimensions {}x{}",
        content.len(),
        max_content_width,
        required_width,
        required_height
    );

    (required_width, required_height)
}
