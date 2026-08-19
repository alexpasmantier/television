use crate::screen::{
    constants::HAIRLINE_BORDER_SET, layout::pane_separator_side,
};
use crate::utils::strings::SPACE;
use crate::{
    action::{Action, CUSTOM_ACTION_PREFIX},
    config::layers::MergedConfig,
    screen::colors::Colorscheme,
    television::Mode,
};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
};
use std::collections::BTreeMap;

/// Keep a few lines visible when scrolled all the way down, mirroring the
/// preview's scrolling behavior
const MIN_VISIBLE_LINES: u16 = 3;

/// Furthest the help panel can scroll given the current mode's content
pub fn max_help_scroll(config: &MergedConfig, mode: Mode) -> u16 {
    u16::try_from(help_line_count(config, mode))
        .unwrap_or(u16::MAX)
        .saturating_sub(MIN_VISIBLE_LINES)
}

/// Draws the help panel inside the preview pane (behind the hairline
/// separator), like the actions picker.
pub fn draw_help_pane(
    f: &mut Frame<'_>,
    rect: Rect,
    config: &MergedConfig,
    tv_mode: Mode,
    scroll: u16,
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

    let mut block = Block::default()
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

    // the percent title below shares the top row with the panel title, so
    // it doesn't affect the inner rect: safe to check for room (and bail)
    // before generating any content
    let inner = block.inner(rect);
    if inner.area() == 0 {
        f.render_widget(block, rect);
        return;
    }

    let content = generate_help_content(config, tv_mode, colorscheme);
    let total = u16::try_from(content.len()).unwrap_or(u16::MAX);
    // the scroll state can go stale when the content changes (e.g. on mode
    // switch), so clamp it defensively
    let scroll = scroll.min(total.saturating_sub(MIN_VISIBLE_LINES));

    // dimmed scroll percentage on the right of the title row, standing in
    // for a scrollbar (same treatment as the preview)
    if scroll > 0 {
        let percent = (u32::from(scroll) * 100 / u32::from(total)).min(100);
        let mut percent_spans = vec![Span::styled(
            format!(" {}% ", percent),
            Style::default()
                .fg(colorscheme.general.dimmed_text_fg)
                .italic(),
        )];
        if separator.intersects(Borders::TOP) {
            percent_spans.push(Span::styled(
                "─",
                Style::default().fg(colorscheme.general.border_fg),
            ));
        }
        block = block
            .title_top(Line::from(percent_spans).alignment(Alignment::Right));
    }

    f.render_widget(block, rect);
    f.render_widget(Paragraph::new(content).scroll((scroll, 0)), inner);
}

/// Help panel section titles, in display order. Editing goes last: it's
/// standard readline fare, and the least painful to lose when the panel is
/// too short to show everything.
const SECTION_TITLES: [&str; 7] = [
    "navigation",
    "selection",
    "preview",
    "source",
    "external actions",
    "general",
    "editing",
];

/// Maps an action to its help panel section and its rank within that
/// section, so related bindings appear together in a stable, logical order.
/// Actions that shouldn't appear in the help panel map to `None`.
fn help_section(action: &Action) -> Option<(usize, u8)> {
    Some(match action {
        // navigation
        Action::SelectPrevEntry => (0, 0),
        Action::SelectNextEntry => (0, 1),
        Action::SelectPrevPage => (0, 2),
        Action::SelectNextPage => (0, 3),
        Action::SelectPrevHistory => (0, 4),
        Action::SelectNextHistory => (0, 5),
        // selection
        Action::ConfirmSelection => (1, 0),
        Action::ToggleSelectionDown => (1, 1),
        Action::ToggleSelectionUp => (1, 2),
        Action::CopyEntryToClipboard => (1, 3),
        // preview
        Action::ScrollPreviewUp => (2, 0),
        Action::ScrollPreviewDown => (2, 1),
        Action::ScrollPreviewHalfPageUp => (2, 2),
        Action::ScrollPreviewHalfPageDown => (2, 3),
        Action::TogglePreview => (2, 4),
        Action::CyclePreviews => (2, 5),
        // source
        Action::ReloadSource => (3, 0),
        Action::CycleSources => (3, 1),
        // external actions
        Action::ExternalAction(_) => (4, 0),
        // general
        Action::ToggleRemoteControl => (5, 0),
        Action::ToggleActionPicker => (5, 1),
        Action::ToggleHelp => (5, 2),
        Action::ToggleStatusBar => (5, 3),
        Action::ToggleOrientation => (5, 4),
        Action::Quit => (5, 5),
        // editing
        Action::GoToPrevChar => (6, 0),
        Action::GoToNextChar => (6, 1),
        Action::GoToInputStart => (6, 2),
        Action::GoToInputEnd => (6, 3),
        Action::DeletePrevChar => (6, 4),
        Action::DeleteNextChar => (6, 5),
        Action::DeletePrevWord => (6, 6),
        Action::DeleteLine => (6, 7),
        _ => return None,
    })
}

/// Formats an external action name (`actions:goto_parent_dir`) like the
/// built-in sentence-case descriptions ("Goto parent dir")
fn external_action_description(name: &str) -> String {
    let name = name
        .trim_start_matches(CUSTOM_ACTION_PREFIX)
        .replace('_', SPACE);
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => {
            first.to_uppercase().collect::<String>() + chars.as_str()
        }
        None => name,
    }
}

/// Checks if an action is relevant for the given mode
fn is_action_relevant_for_mode(action: &Action, mode: Mode) -> bool {
    match mode {
        Mode::Channel | Mode::ActionPicker => true,
        // remote control only supports input editing, navigation and a few
        // global toggles
        Mode::RemoteControl => matches!(
            action,
            Action::SelectNextEntry
                | Action::SelectPrevEntry
                | Action::SelectNextPage
                | Action::SelectPrevPage
                | Action::ConfirmSelection
                | Action::GoToPrevChar
                | Action::GoToNextChar
                | Action::GoToInputStart
                | Action::GoToInputEnd
                | Action::DeletePrevChar
                | Action::DeleteNextChar
                | Action::DeletePrevWord
                | Action::DeleteLine
                | Action::ToggleRemoteControl
                | Action::ToggleHelp
                | Action::ToggleStatusBar
                | Action::Quit
        ),
    }
}

/// The effective keybinding rows for the given mode: `(section, keys,
/// description)`, ordered by section then rank
fn help_rows(
    config: &MergedConfig,
    mode: Mode,
) -> Vec<(usize, String, String)> {
    // what the user can actually press right now: channel bindings override
    // globals in channel mode, other modes only use globals (iterate by
    // reference, this runs on every render frame)
    let global = &config.input_map.global_keybindings;
    let channel = match mode {
        Mode::Channel => Some(&config.input_map.channel_keybindings),
        Mode::RemoteControl | Mode::ActionPicker => None,
    };
    let bindings = global
        .iter()
        .filter(|(key, _)| !channel.is_some_and(|c| c.contains_key(key)))
        .chain(channel.into_iter().flat_map(|c| c.iter()));

    // group keys sharing the same actions onto one line, ordered by
    // (section, rank); the description disambiguates equal ranks
    let mut entries: BTreeMap<(usize, u8, String), Vec<String>> =
        BTreeMap::new();
    for (key, actions) in bindings {
        let relevant: Vec<&Action> = actions
            .as_slice()
            .iter()
            .filter(|a| {
                help_section(a).is_some()
                    && is_action_relevant_for_mode(a, mode)
            })
            .collect();
        let Some(first) = relevant.first() else {
            continue;
        };
        let (section, rank) = help_section(first).unwrap();
        let description = relevant
            .iter()
            .map(|a| match a {
                Action::ExternalAction(name) => {
                    external_action_description(name)
                }
                _ => a.description().to_string(),
            })
            .collect::<Vec<_>>()
            .join(" + ");
        entries
            .entry((section, rank, description))
            .or_default()
            .push(key.to_string());
    }

    entries
        .into_iter()
        .map(|((section, _, description), mut keys)| {
            // keybindings iterate in hash order: sort for a stable display
            keys.sort();
            keys.dedup();
            (section, keys.join(" / "), description)
        })
        .collect()
}

/// Number of lines the help content occupies, including section headers and
/// the blank lines between sections
fn help_line_count(config: &MergedConfig, mode: Mode) -> usize {
    let rows = help_rows(config, mode);
    if !config.help_panel_show_categories {
        return rows.len();
    }
    let mut sections = 0;
    let mut current = None;
    for (section, _, _) in &rows {
        if current != Some(section) {
            sections += 1;
            current = Some(section);
        }
    }
    if sections == 0 {
        0
    } else {
        // a header per section, plus a blank line between sections
        rows.len() + 2 * sections - 1
    }
}

/// Generates the help content: the effective keybindings for the current
/// mode, grouped into sections
fn generate_help_content(
    config: &MergedConfig,
    mode: Mode,
    colorscheme: &Colorscheme,
) -> Vec<Line<'static>> {
    let mode_color = match mode {
        Mode::Channel => colorscheme.mode.channel,
        Mode::RemoteControl => colorscheme.mode.remote_control,
        Mode::ActionPicker => colorscheme.mode.action_picker,
    };

    let rows = help_rows(config, mode);

    // align descriptions across the whole panel, not per section
    let key_col_width = rows
        .iter()
        .map(|(_, keys, _)| keys.chars().count())
        .max()
        .unwrap_or(0);

    let mut lines = Vec::new();
    let mut current_section = None;
    for (section, keys, description) in rows {
        if config.help_panel_show_categories
            && current_section != Some(section)
        {
            if current_section.is_some() {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(Span::styled(
                SECTION_TITLES[section],
                Style::default().fg(mode_color),
            )));
            current_section = Some(section);
        }
        lines.push(create_compact_keybinding_line(
            &keys,
            &description,
            key_col_width,
            colorscheme,
        ));
    }
    lines
}

/// Minimum width of the key column, so short key lists don't cram the
/// descriptions against them
const KEY_COLUMN_MIN_WIDTH: usize = 12;

/// Creates a compact keybinding line: the key(s) in the foreground color,
/// followed by a dimmed description
fn create_compact_keybinding_line(
    key: &str,
    action: &str,
    key_col_width: usize,
    colorscheme: &Colorscheme,
) -> Line<'static> {
    let width = key_col_width.max(KEY_COLUMN_MIN_WIDTH);
    Line::from(vec![
        Span::styled(
            format!("  {:<width$}", key),
            Style::default().fg(colorscheme.results.result_fg),
        ),
        Span::raw(SPACE),
        Span::styled(
            action.to_string(),
            Style::default().fg(colorscheme.general.dimmed_text_fg),
        ),
    ])
}
