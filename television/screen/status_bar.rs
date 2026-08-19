use crate::{
    action::Action, draw::Ctx, television::Mode, utils::strings::SPACE,
};
use ratatui::{
    Frame,
    layout::{
        Alignment, Constraint, Direction, Layout as RatatuiLayout, Rect,
    },
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

const HINT_SEP: &str = " · ";

/// Draw the status bar at the bottom of the screen: a mode-colored dot and
/// the channel name on the left, dimmed keybinding hints in the middle, and
/// the version on the right.
pub fn draw_status_bar(f: &mut Frame<'_>, area: Rect, ctx: &Ctx) {
    let chunks = RatatuiLayout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1), // Left: mode dot + channel info
            Constraint::Fill(3), // Middle: hints
            Constraint::Fill(1), // Right: version
        ])
        .split(area);

    let dimmed = Style::default().fg(ctx.colorscheme.general.dimmed_text_fg);
    let faint = Style::default().fg(ctx.colorscheme.general.border_fg);

    // === LEFT SECTION: mode dot + label ===
    let (mode_color, mode_label) = match ctx.tv_state.mode {
        Mode::Channel => (
            ctx.colorscheme.mode.channel,
            ctx.tv_state.channel_state.current_channel_name.as_str(),
        ),
        Mode::RemoteControl => {
            (ctx.colorscheme.mode.remote_control, "channels")
        }
        Mode::ActionPicker => (ctx.colorscheme.mode.action_picker, "actions"),
    };

    let mut left_spans = vec![
        Span::raw(SPACE),
        Span::styled("●", Style::default().fg(mode_color)),
        Span::raw(SPACE),
        Span::styled(
            mode_label,
            Style::default().fg(ctx.colorscheme.results.result_fg),
        ),
    ];

    // Selected count indicator
    if ctx.tv_state.mode == Mode::Channel {
        let selected_count = ctx.tv_state.channel_state.selected_entries.len();
        if selected_count > 0 {
            left_spans.extend([
                Span::styled(" · ", faint),
                Span::styled(
                    format!("{} selected", selected_count),
                    dimmed.add_modifier(Modifier::ITALIC),
                ),
            ]);
        }
    }

    // === MIDDLE SECTION: dimmed hints ===
    let mut hints: Vec<String> = Vec::new();
    let mut add_hint = |description: &str, keybinding: &str| {
        hints.push(format!("{} {}", description, keybinding));
    };

    if ctx.tv_state.mode == Mode::Channel
        && ctx.tv_state.channel_state.source_count > 1
    {
        let key = &ctx
            .config
            .input_map
            .get_key_for_action(&Action::CycleSources);
        if let Some(k) = key {
            add_hint("source", &k.to_string());
        }
    }

    if !ctx.config.remote_disabled {
        let key = &ctx
            .config
            .input_map
            .get_key_for_action(&Action::ToggleRemoteControl);
        if let Some(k) = key {
            let hint_text = match ctx.tv_state.mode {
                Mode::Channel | Mode::ActionPicker => "remote",
                Mode::RemoteControl => "back",
            };
            add_hint(hint_text, &k.to_string());
        }
    }

    if ctx.tv_state.mode == Mode::Channel
        && !ctx.config.channel_actions.is_empty()
    {
        let key = &ctx
            .config
            .input_map
            .get_key_for_action(&Action::ToggleActionPicker);
        if let Some(k) = key {
            add_hint("actions", &k.to_string());
        }
    }

    let help_hint = ctx
        .config
        .input_map
        .get_key_for_action(&Action::ToggleHelp)
        .map(|k| format!("help {}", k));
    hints.extend(help_hint.clone());

    // on narrow terminals, rather than let the centered paragraph clip the
    // line mid-word, fall back to the help hint alone: it leads to the full
    // keybinding list anyway
    let line_width = |hints: &[String]| {
        hints.iter().map(|h| h.chars().count()).sum::<usize>()
            + HINT_SEP.chars().count() * hints.len().saturating_sub(1)
    };
    if line_width(&hints) > chunks[1].width as usize {
        hints = help_hint.into_iter().collect();
        if line_width(&hints) > chunks[1].width as usize {
            hints.clear();
        }
    }

    let mut hint_spans = Vec::new();
    for hint in &hints {
        if !hint_spans.is_empty() {
            hint_spans.push(Span::styled(HINT_SEP, faint));
        }
        hint_spans.push(Span::styled(hint.clone(), dimmed));
    }

    // === RIGHT SECTION: version ===
    let right_spans = vec![Span::styled(
        format!("v{} ", ctx.app_metadata.version),
        faint.add_modifier(Modifier::ITALIC),
    )];

    f.render_widget(
        Paragraph::new(Line::from(left_spans)).alignment(Alignment::Left),
        chunks[0],
    );
    f.render_widget(
        Paragraph::new(Line::from(hint_spans)).alignment(Alignment::Center),
        chunks[1],
    );
    f.render_widget(
        Paragraph::new(Line::from(right_spans)).alignment(Alignment::Right),
        chunks[2],
    );
}
