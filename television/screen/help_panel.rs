use crate::event::Key;
use crate::utils::strings::SPACE;
use crate::{
    action::{Action, Actions},
    config::{Keybindings, layers::MergedConfig},
    screen::colors::Colorscheme,
    television::Mode,
    utils::strings::to_title_case,
};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
};
use rustc_hash::FxHashMap;
use tracing::{debug, trace};

const MIN_PANEL_WIDTH: u16 = 25;
const MIN_PANEL_HEIGHT: u16 = 5;

/// Draws a Helix-style floating help panel in the bottom-right corner
pub fn draw_help_panel(
    f: &mut Frame<'_>,
    area: Rect,
    config: &MergedConfig,
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

/// Checks if an action is relevant for the given mode
fn is_action_relevant_for_mode(action: &Action, mode: Mode) -> bool {
    match mode {
        Mode::Channel | Mode::ActionPicker => {
            // Channel mode - all actions except those specifically for remote mode switching
            match action {
                // Input actions - available in both modes
                Action::AddInputChar(_)
                | Action::DeletePrevChar
                | Action::DeletePrevWord
                | Action::DeleteNextChar
                | Action::DeleteLine
                | Action::GoToPrevChar
                | Action::GoToNextChar
                | Action::GoToInputStart
                | Action::GoToInputEnd
                // Navigation actions - available in both modes
                | Action::SelectNextEntry
                | Action::SelectPrevEntry
                | Action::SelectNextPage
                | Action::SelectPrevPage
                // Selection actions - channel specific (multi-select)
                | Action::ToggleSelectionDown
                | Action::ToggleSelectionUp
                | Action::ConfirmSelection
                // Preview actions - channel specific
                | Action::ScrollPreviewUp
                | Action::ScrollPreviewDown
                | Action::ScrollPreviewHalfPageUp
                | Action::ScrollPreviewHalfPageDown
                | Action::TogglePreview
                // Channel-specific actions
                | Action::CopyEntryToClipboard
                | Action::ReloadSource
                | Action::CycleSources
                | Action::CyclePreviews
                | Action::SelectPrevHistory
                | Action::SelectNextHistory
                // UI toggles - global
                | Action::ToggleRemoteControl
                | Action::ToggleActionPicker
                | Action::ToggleHelp
                | Action::ToggleStatusBar
                // Channel-mode layout
                | Action::ToggleOrientation
                // Application actions - global
                | Action::Quit
                // External actions
                | Action::ExternalAction(_) => true,

                // Skip actions not relevant to help or internal actions
                Action::NoOp
                | Action::Render
                | Action::Resize(_, _)
                | Action::ClearScreen
                | Action::Tick
                | Action::Suspend
                | Action::Resume
                | Action::Error(_)
                | Action::OpenEntry
                | Action::SwitchToChannel(_)
                | Action::WatchTimer
                | Action::SelectEntryAtPosition(_, _)
                | Action::MouseClickAt(_, _)
                | Action::Expect(_)
                | Action::SelectAndExit => false,
            }
        }
        Mode::RemoteControl => {
            // Remote control mode - limited set of actions
            match action {
                // Input actions - available in both modes
                Action::AddInputChar(_)
                | Action::DeletePrevChar
                | Action::DeletePrevWord
                | Action::DeleteNextChar
                | Action::DeleteLine
                | Action::GoToPrevChar
                | Action::GoToNextChar
                | Action::GoToInputStart
                | Action::GoToInputEnd
                // Navigation actions - available in both modes
                | Action::SelectNextEntry
                | Action::SelectPrevEntry
                | Action::SelectNextPage
                | Action::SelectPrevPage
                // Selection in remote mode - just confirm (no multi-select)
                | Action::ConfirmSelection
                // UI toggles - global
                | Action::ToggleRemoteControl
                | Action::ToggleHelp
                | Action::ToggleStatusBar
                // Application actions - global
                | Action::Quit => true,

                // All other actions not relevant in remote control mode
                _ => false,
            }
        }
    }
}

/// Adds keybinding lines for specific keys to the given lines vector
fn add_keybinding_lines_for_keys(
    lines: &mut Vec<Line<'static>>,
    keybindings: &Keybindings,
    mode: Mode,
    colorscheme: &Colorscheme,
    category_name: &str,
) {
    // Collect all valid keybinding entries
    let mut entries: Vec<(String, String)> = Vec::new();

    for (key, actions) in keybindings.iter() {
        for action in actions.as_slice() {
            // Filter out NoOp actions (unbound keys)
            // Filter out actions not relevant for current mode
            if matches!(action, Action::NoOp)
                || !is_action_relevant_for_mode(action, mode)
            {
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

    // Sort entries alphabetically by description
    entries.sort_by(|a, b| a.0.cmp(&b.0));

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

/// Adds keybinding lines for external actions to the given lines vector
fn add_actions_keybindings_section(
    lines: &mut Vec<Line<'static>>,
    key_actions: &FxHashMap<Key, Actions>,
    colorscheme: &Colorscheme,
    mode: Mode,
) {
    // Collect all valid external action entries
    let mut entries = Vec::new();

    for (key, actions) in key_actions {
        // Skip keys without external actions
        if !actions
            .as_slice()
            .iter()
            .any(|a| matches!(a, Action::ExternalAction(_)))
        {
            continue;
        }

        // Build action description from all valid actions
        let action_desc = actions
            .as_slice()
            .iter()
            .filter(|a| {
                !matches!(a, Action::NoOp)
                    && is_action_relevant_for_mode(a, mode)
            })
            .map(|a| match a {
                Action::ExternalAction(name) => to_title_case(name),
                _ => to_title_case(a.description()),
            })
            .collect::<Vec<_>>()
            .join(" + ");

        if !action_desc.is_empty() {
            entries.push((key.to_string(), action_desc));
        }
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    // Create lines from sorted entries
    for (key_string, action_desc) in entries {
        lines.push(create_external_action_line(
            &key_string,
            &action_desc,
            colorscheme,
        ));
    }
}

/// Generates the help content organized into global and mode-specific groups
fn generate_help_content(
    config: &MergedConfig,
    mode: Mode,
    colorscheme: &Colorscheme,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    debug!("Generating help content for mode: {:?}", mode);

    // Mode-specific keybindings section header
    let mode_name = match mode {
        Mode::Channel => "Channel Mode",
        Mode::RemoteControl => "Remote Control Mode",
        Mode::ActionPicker => "Action Picker Mode",
    };

    lines.push(Line::from(vec![Span::styled(
        mode_name,
        Style::default()
            .fg(colorscheme.help.metadata_field_name_fg)
            .bold()
            .underlined(),
    )]));

    add_keybinding_lines_for_keys(
        &mut lines,
        &config
            .input_map
            .global_keybindings
            .iter()
            .map(|(key, actions)| (*key, actions.first().unwrap().clone()))
            .collect::<Vec<_>>()
            .into(),
        mode,
        colorscheme,
        mode_name,
    );

    // Check if we have external actions before adding the section
    let has_external_actions =
        config
            .input_map
            .global_keybindings
            .iter()
            .any(|(_, actions)| {
                actions
                    .as_slice()
                    .iter()
                    .any(|a| matches!(a, Action::ExternalAction(_)))
            });

    if has_external_actions {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "External Actions",
            Style::default()
                .fg(colorscheme.help.metadata_field_name_fg)
                .bold()
                .underlined(),
        )]));

        add_actions_keybindings_section(
            &mut lines,
            &config.input_map.global_keybindings,
            colorscheme,
            mode,
        );
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
        Mode::Channel | Mode::ActionPicker => colorscheme.mode.channel,
        Mode::RemoteControl => colorscheme.mode.remote_control,
    };

    Line::from(vec![
        Span::styled(
            format!("{}:", action),
            Style::default().fg(colorscheme.help.metadata_field_name_fg),
        ),
        Span::raw(SPACE), // Space between action and key
        Span::styled(key.to_string(), Style::default().fg(key_color).bold()),
    ])
}

fn create_external_action_line(
    key: &str,
    actions: &str,
    colorscheme: &Colorscheme,
) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{}:", key),
            Style::default().fg(colorscheme.mode.channel).bold(),
        ),
        Span::raw(SPACE), // Space between key and actions
        Span::styled(
            actions.to_string(),
            Style::default().fg(colorscheme.help.metadata_field_name_fg),
        ),
    ])
}

/// Calculates the required dimensions for the help panel based on content
#[allow(clippy::cast_possible_truncation)]
pub fn calculate_help_panel_size(
    config: &MergedConfig,
    mode: Mode,
    colorscheme: &Colorscheme,
) -> (u16, u16) {
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
