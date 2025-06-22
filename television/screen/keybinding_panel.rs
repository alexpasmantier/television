use crate::{
    action::Action, config::KeyBindings, screen::colors::Colorscheme,
    television::Mode,
};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

/// Draws a Helix-style floating keybinding panel in the bottom-right corner
pub fn draw_keybinding_panel(
    f: &mut Frame<'_>,
    area: Rect,
    keybindings: &KeyBindings,
    mode: Mode,
    colorscheme: &Colorscheme,
) {
    if area.width < 15 || area.height < 5 {
        return; // Too small to display anything meaningful
    }

    // Generate content
    let content = generate_keybinding_content(keybindings, mode, colorscheme);

    // Clear the area first to create the floating effect
    f.render_widget(Clear, area);

    // Create the main block with consistent styling
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colorscheme.general.border_fg))
        .title_top(Line::from(" Keybindings ").alignment(Alignment::Center))
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        );

    let paragraph = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

/// Helper function to extract keys from a binding and convert to strings
fn extract_keys_from_binding(
    binding: &crate::config::keybindings::Binding,
) -> Vec<String> {
    match binding {
        crate::config::keybindings::Binding::SingleKey(key) => {
            vec![key.to_string()]
        }
        crate::config::keybindings::Binding::MultipleKeys(keys) => {
            keys.iter().map(ToString::to_string).collect()
        }
    }
}

/// Adds keybinding lines for a list of actions to the given lines vector
fn add_keybinding_lines_for_actions(
    lines: &mut Vec<Line<'static>>,
    keybindings: &KeyBindings,
    actions: &[(Action, &str)],
    colorscheme: &Colorscheme,
) {
    for (action, description) in actions {
        if let Some(binding) = keybindings.get(action) {
            let keys = extract_keys_from_binding(binding);
            for key in keys {
                lines.push(create_compact_keybinding_line(
                    &key,
                    description,
                    colorscheme,
                ));
            }
        }
    }
}

/// Generates the keybinding content organized into global and mode-specific groups
fn generate_keybinding_content(
    keybindings: &KeyBindings,
    mode: Mode,
    colorscheme: &Colorscheme,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    // Global keybindings section header
    lines.push(Line::from(vec![
        Span::raw(" "), // Left padding
        Span::styled(
            "Global",
            Style::default()
                .fg(colorscheme.help.metadata_field_name_fg)
                .bold()
                .underlined(),
        ),
    ]));

    // Global actions that work in all modes
    let global_actions = [
        (Action::Quit, "Quit"),
        (Action::ToggleHelp, "Toggle help"),
        (Action::TogglePreview, "Toggle preview"),
        (Action::ToggleKeybindingPanel, "Toggle keys"),
    ];

    add_keybinding_lines_for_actions(
        &mut lines,
        keybindings,
        &global_actions,
        colorscheme,
    );

    // Add spacing between Global and mode-specific sections
    lines.push(Line::from(""));

    // Mode-specific keybindings section header
    let mode_name = match mode {
        Mode::Channel => "Channel",
        Mode::RemoteControl => "Remote",
    };

    lines.push(Line::from(vec![
        Span::raw(" "), // Left padding
        Span::styled(
            mode_name,
            Style::default()
                .fg(colorscheme.help.metadata_field_name_fg)
                .bold()
                .underlined(),
        ),
    ]));

    // Navigation actions (common to both modes)
    let nav_actions = [
        (Action::SelectPrevEntry, "Navigate up"),
        (Action::SelectNextEntry, "Navigate down"),
        (Action::SelectPrevPage, "Page up"),
        (Action::SelectNextPage, "Page down"),
    ];

    add_keybinding_lines_for_actions(
        &mut lines,
        keybindings,
        &nav_actions,
        colorscheme,
    );

    // Mode-specific actions
    match mode {
        Mode::Channel => {
            let channel_actions = [
                (Action::ConfirmSelection, "Select entry"),
                (Action::ToggleSelectionDown, "Toggle selection down"),
                (Action::ToggleSelectionUp, "Toggle selection up"),
                (Action::CopyEntryToClipboard, "Copy to clipboard"),
                (Action::ScrollPreviewHalfPageUp, "Preview scroll up"),
                (Action::ScrollPreviewHalfPageDown, "Preview scroll down"),
                (Action::ToggleRemoteControl, "Toggle remote"),
                (Action::CycleSources, "Cycle sources"),
                (Action::ReloadSource, "Reload source"),
            ];
            add_keybinding_lines_for_actions(
                &mut lines,
                keybindings,
                &channel_actions,
                colorscheme,
            );
        }
        Mode::RemoteControl => {
            let remote_actions = [
                (Action::ConfirmSelection, "Select entry"),
                (Action::ToggleRemoteControl, "Back to channel"),
            ];
            add_keybinding_lines_for_actions(
                &mut lines,
                keybindings,
                &remote_actions,
                colorscheme,
            );
        }
    }

    lines
}

/// Creates a compact keybinding line with one space of left padding
fn create_compact_keybinding_line(
    key: &str,
    action: &str,
    colorscheme: &Colorscheme,
) -> Line<'static> {
    Line::from(vec![
        Span::raw(" "), // Left padding
        Span::styled(
            format!("{}:", action),
            Style::default().fg(colorscheme.help.metadata_field_name_fg),
        ),
        Span::raw(" "), // Space between action and key
        Span::styled(
            key.to_string(),
            Style::default().fg(colorscheme.mode.channel).bold(),
        ),
    ])
}

/// Calculates the required dimensions for the keybinding panel based on content
#[allow(clippy::cast_possible_truncation)]
pub fn calculate_keybinding_panel_size(
    keybindings: &KeyBindings,
    mode: Mode,
    colorscheme: &Colorscheme,
) -> (u16, u16) {
    // Generate content to count items and calculate width
    let content = generate_keybinding_content(keybindings, mode, colorscheme);

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
