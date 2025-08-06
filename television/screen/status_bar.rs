use crate::{action::Action, draw::Ctx, television::Mode};
use ratatui::{
    Frame,
    layout::{
        Alignment, Constraint, Direction, Layout as RatatuiLayout, Rect,
    },
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

/// Draw the status bar at the bottom of the screen
pub fn draw_status_bar(f: &mut Frame<'_>, area: Rect, ctx: &Ctx) {
    // Split status bar into three sections
    let chunks = RatatuiLayout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1), // Left: mode + channel info
            Constraint::Fill(3), // Middle: hints
            Constraint::Fill(1), // Right: version
        ])
        .split(area);

    // === LEFT SECTION: Mode bubble and channel info ===
    let mut left_spans = vec![Span::raw(" ")]; // Initial spacing

    // Get mode-specific styling
    let (mode_text, mode_fg, mode_bg) = match ctx.tv_state.mode {
        Mode::Channel => (
            "CHANNEL",
            ctx.colorscheme.mode.channel_fg,
            ctx.colorscheme.mode.channel,
        ),
        Mode::RemoteControl => (
            "REMOTE",
            ctx.colorscheme.mode.remote_control_fg,
            ctx.colorscheme.mode.remote_control,
        ),
    };

    // Create mode bubble with separators
    let separator_style = Style::default().fg(mode_bg).bg(Color::Reset);
    let mode_style = Style::default()
        .fg(mode_fg)
        .bg(mode_bg)
        .add_modifier(Modifier::BOLD);

    // Add opening separator
    if !ctx.config.status_bar_separator_open.is_empty() {
        left_spans.push(Span::styled(
            ctx.config.status_bar_separator_open.clone(),
            separator_style,
        ));
    }

    // Add mode text
    left_spans.push(Span::styled(format!(" {} ", mode_text), mode_style));

    // Add closing separator
    if !ctx.config.status_bar_separator_close.is_empty() {
        left_spans.push(Span::styled(
            ctx.config.status_bar_separator_close.clone(),
            separator_style,
        ));
    }

    // Add channel-specific info in Channel mode
    if ctx.tv_state.mode == Mode::Channel {
        let name_style = Style::default()
            .fg(ctx.colorscheme.results.result_fg)
            .add_modifier(Modifier::BOLD);

        // Channel name
        left_spans.push(Span::styled(
            format!(" {}", ctx.tv_state.channel_state.current_channel_name),
            name_style,
        ));

        // Selected count indicator
        let selected_count = ctx.tv_state.channel_state.selected_entries.len();
        if selected_count > 0 {
            left_spans.extend([
                Span::styled(
                    " • ",
                    Style::default().fg(ctx.colorscheme.general.border_fg),
                ),
                Span::styled(
                    format!("{} selected", selected_count),
                    Style::default()
                        .fg(ctx.colorscheme.results.result_fg)
                        .add_modifier(Modifier::ITALIC),
                ),
            ]);
        }
    }

    // === MIDDLE SECTION: Hints ===
    let mut middle_spans = Vec::new();
    let mut hint_spans = Vec::new();

    // Use mode color for keybinding hints
    let key_color = match ctx.tv_state.mode {
        Mode::Channel => ctx.colorscheme.mode.channel,
        Mode::RemoteControl => ctx.colorscheme.mode.remote_control,
    };

    // Helper to add a hint with consistent styling
    let mut add_hint = |description: &str, keybinding: &str| {
        if !hint_spans.is_empty() {
            hint_spans.push(Span::raw(" • "));
        }
        hint_spans.extend([
            Span::styled(
                format!("{}:", description),
                Style::default()
                    .fg(ctx.colorscheme.help.metadata_field_name_fg),
            ),
            Span::raw(" "),
            Span::styled(
                keybinding.to_string(),
                Style::default().fg(key_color).add_modifier(Modifier::BOLD),
            ),
        ]);
    };

    // Add remote control hint (available in both modes, but only if remote control is enabled)
    if !ctx.config.remote_disabled {
        let key = &ctx
            .config
            .input_map
            .get_key_for_action(&Action::ToggleRemoteControl);
        if let Some(k) = key {
            let hint_text = match ctx.tv_state.mode {
                Mode::Channel => "Remote Control",
                Mode::RemoteControl => "Back to Channel",
            };
            add_hint(hint_text, &k.to_string());
        }
    }

    // Add preview hint (Channel mode only, and only if preview feature is enabled)
    if ctx.tv_state.mode == Mode::Channel && !ctx.config.preview_panel_disabled
    {
        let key = &ctx
            .config
            .input_map
            .get_key_for_action(&Action::TogglePreview);
        if let Some(k) = key {
            let hint_text = if ctx.config.preview_panel_hidden {
                "Show Preview"
            } else {
                "Hide Preview"
            };
            add_hint(hint_text, &k.to_string());
        }
    }

    // Add keybinding help hint (available in both modes)
    let key = &ctx.config.input_map.get_key_for_action(&Action::ToggleHelp);
    if let Some(k) = key {
        add_hint("Help", &k.to_string());
    }

    // Build middle section if we have hints
    if !hint_spans.is_empty() {
        middle_spans.extend([
            Span::styled(
                "[Hint]",
                Style::default()
                    .fg(ctx.colorscheme.general.border_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
        ]);
        middle_spans.extend(hint_spans);
    }

    // === RIGHT SECTION: Version ===
    let right_spans = vec![Span::styled(
        format!("v{} ", ctx.app_metadata.version),
        Style::default()
            .fg(ctx.colorscheme.results.result_fg)
            .add_modifier(Modifier::ITALIC),
    )];

    // Render all sections
    f.render_widget(
        Paragraph::new(Line::from(left_spans)).alignment(Alignment::Left),
        chunks[0],
    );
    f.render_widget(
        Paragraph::new(Line::from(middle_spans)).alignment(Alignment::Center),
        chunks[1],
    );
    f.render_widget(
        Paragraph::new(Line::from(right_spans)).alignment(Alignment::Right),
        chunks[2],
    );
}
