use crate::event::Key;
use crate::screen::{
    constants::HAIRLINE_BORDER_SET, layout::pane_separator_side,
};
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
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
};
use rustc_hash::FxHashMap;
use tracing::{debug, trace};

/// Draws the help panel inside the preview pane (behind the hairline
/// separator), like the actions picker.
pub fn draw_help_pane(
    f: &mut Frame<'_>,
    rect: Rect,
    config: &MergedConfig,
    tv_mode: Mode,
    colorscheme: &Colorscheme,
) {
    // hairline on the side facing the results, mirroring the preview
    let separator =
        pane_separator_side(config.layout, config.input_bar_position);
    let mode_color = match tv_mode {
        Mode::Channel => colorscheme.mode.channel,
        Mode::RemoteControl => colorscheme.mode.remote_control,
        Mode::ActionPicker => colorscheme.mode.action_picker,
    };
    let mut title_spans = vec![Span::from(" ")];
    // the title embeds into a horizontal hairline, so lead with a line
    // segment (same treatment as the preview title)
    if separator.intersects(Borders::TOP) {
        title_spans.insert(
            0,
            Span::styled(
                "─",
                Style::default().fg(colorscheme.general.border_fg),
            ),
        );
    }
    title_spans
        .push(Span::styled("help", Style::default().fg(mode_color).bold()));
    title_spans.push(Span::from(" "));

    let block = Block::default()
        .title_top(Line::from(title_spans))
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        )
        .borders(separator)
        .border_set(HAIRLINE_BORDER_SET)
        .border_style(Style::default().fg(colorscheme.general.border_fg))
        .padding(Padding {
            top: 1,
            right: 1,
            bottom: 0,
            left: 2,
        });
    let inner = block.inner(rect);
    f.render_widget(block, rect);
    if inner.area() == 0 {
        return;
    }

    let content = generate_help_content(config, tv_mode, colorscheme);
    f.render_widget(Paragraph::new(content), inner);
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
        lines.push(create_compact_keybinding_line(
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
    let (mode_name, mode_color) = match mode {
        Mode::Channel => ("channel", colorscheme.mode.channel),
        Mode::RemoteControl => {
            ("remote control", colorscheme.mode.remote_control)
        }
        Mode::ActionPicker => ("actions", colorscheme.mode.action_picker),
    };

    lines.push(Line::from(vec![Span::styled(
        mode_name,
        Style::default().fg(mode_color),
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
            "external actions",
            Style::default().fg(mode_color),
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

/// Creates a compact keybinding line: the key in the foreground color,
/// followed by a dimmed description
fn create_compact_keybinding_line(
    key: &str,
    action: &str,
    colorscheme: &Colorscheme,
) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {:<12}", key),
            Style::default().fg(colorscheme.results.result_fg),
        ),
        Span::raw(SPACE),
        Span::styled(
            action.to_string(),
            Style::default().fg(colorscheme.general.dimmed_text_fg),
        ),
    ])
}
