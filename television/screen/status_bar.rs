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
use unicode_width::UnicodeWidthStr;

const HINT_SEP: &str = " · ";
const HINT_SEP_WIDTH: usize = 3;

///  |● files            source ctrl-s · remote ctrl-t · actions ctrl-x · help ctrl-h|
pub fn draw_status_bar(f: &mut Frame<'_>, area: Rect, ctx: &Ctx) {
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
    let selected_count = ctx.tv_state.channel_state.selected.len();
    let mut left_chunk_width = 3 + mode_label.width();
    if selected_count > 0 {
        left_chunk_width += 3 + selected_count.to_string().len();
    }

    let chunks = RatatuiLayout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(
                u16::try_from(left_chunk_width).unwrap_or(u16::MAX),
            ),
            Constraint::Fill(1),
        ])
        .split(area);

    let dimmed = Style::default().fg(ctx.colorscheme.general.dimmed_text_fg);
    let faint = Style::default().fg(ctx.colorscheme.general.border_fg);

    let mut left_spans = vec![
        Span::raw(SPACE),
        Span::styled("●", Style::default().fg(mode_color)),
        Span::raw(SPACE),
        Span::styled(
            mode_label,
            Style::default().fg(ctx.colorscheme.results.result_fg),
        ),
    ];

    if selected_count > 0 {
        left_spans
            .push(Span::styled(format!(" · {}", selected_count), dimmed));
    }

    // keybinding hints
    let mut hints: Vec<(String, String)> = Vec::new();
    if ctx.tv_state.mode == Mode::Channel
        && ctx.tv_state.channel_state.source_count > 1
    {
        let key = &ctx
            .config
            .input_map
            .get_key_for_action(&Action::CycleSources);
        if let Some(k) = key {
            hints.push(("source".to_string(), k.to_string()));
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
            hints.push((hint_text.to_string(), k.to_string()));
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
            hints.push(("actions".to_string(), k.to_string()));
        }
    }
    let help_hint = ctx
        .config
        .input_map
        .get_key_for_action(&Action::ToggleHelp)
        .map(|k| ("help".to_string(), k.to_string()));
    hints.extend(help_hint.clone());

    // if we lack space, fall back to just showing the help hint or nothing at all
    let available_width = chunks[1].width as usize;
    let hint_width =
        |hint: &(String, String)| hint.0.width() + 1 + hint.1.width();
    if hints.iter().map(hint_width).sum::<usize>()
        + HINT_SEP_WIDTH * hints.len().saturating_sub(1)
        + 2 // LR padding
        > available_width
    {
        hints = help_hint.into_iter().collect();
        if hints.iter().map(hint_width).sum::<usize>() + 2 > available_width {
            hints.clear();
        }
    }

    let mut hint_spans = Vec::new();
    for hint in &hints {
        if !hint_spans.is_empty() {
            hint_spans.push(Span::styled(HINT_SEP, faint));
        }
        hint_spans.extend([
            Span::styled(hint.0.clone(), dimmed),
            Span::raw(SPACE),
            Span::styled(hint.1.clone(), dimmed.add_modifier(Modifier::BOLD)),
        ]);
    }
    hint_spans.push(Span::raw(SPACE));

    f.render_widget(
        Paragraph::new(Line::from(left_spans)).alignment(Alignment::Left),
        chunks[0],
    );
    f.render_widget(
        Paragraph::new(Line::from(hint_spans)).alignment(Alignment::Right),
        chunks[1],
    );
}
