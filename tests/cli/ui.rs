//! Tests for CLI UI/layout options: --layout, --input-header, --ui-scale, --no-remote, --no-status-bar.
//!
//! These tests verify Television's user interface customization capabilities,
//! ensuring users can adapt the layout and appearance to their preferences and needs.

use television::tui::TESTING_ENV_VAR;

use super::super::common::*;

/// Tests that --layout landscape arranges panels side-by-side.
#[test]
fn test_layout_landscape() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--layout", "landscape"],
    )
    .start()
    .unwrap();

    s.wait().text("▏").text_absent("────────").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --layout portrait arranges panels vertically stacked.
#[test]
fn test_layout_portrait() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--layout", "portrait"],
    )
    .start()
    .unwrap();

    s.wait().text("────────").text_absent("▏").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests toggling layout at runtime via a custom keybinding.
// FIXME: this should be in a separate module that tests TUI interactions
#[test]
fn test_toggle_layout() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "files",
            "--layout",
            "portrait",
            "--keybindings",
            "ctrl-l='toggle_layout'",
        ],
    )
    .start()
    .unwrap();

    s.wait().text("────────").text_absent("▏").until().unwrap();

    s.send().key("ctrl-l").unwrap();

    s.wait().text("▏").text_absent("────────").until().unwrap();

    s.send().key("ctrl-l").unwrap();

    s.wait().text("────────").text_absent("▏").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --input-header customizes the text above the search input in Channel Mode.
#[test]
fn test_input_header_in_channel_mode() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--input-header", "UNIQUE16CHARID"],
    )
    .start()
    .unwrap();

    s.wait()
        .text("UNIQUE16CHARID")
        .text("● files")
        .until()
        .unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --input-header works in Ad-hoc Mode.
#[test]
fn test_input_header_in_adhoc_mode() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--source-command", "ls", "--input-header", "UNIQUE16CHARID"],
    )
    .start()
    .unwrap();

    s.wait()
        .text("UNIQUE16CHARID")
        .text("● Custom")
        .until()
        .unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --input-prompt customizes the prompt symbol in Channel Mode.
#[test]
fn test_input_prompt_in_channel_mode() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--input-prompt", "❯ "],
    )
    .start()
    .unwrap();

    s.wait().text("❯ ").text("● files").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --input-prompt works in Ad-hoc Mode.
#[test]
fn test_input_prompt_in_adhoc_mode() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--source-command", "ls", "--input-prompt", "→ "],
    )
    .start()
    .unwrap();

    s.wait().text("→ ").text("● Custom").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that the input has no prompt (nor any other glyph) by default.
#[test]
fn test_default_input_prompt() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(&pt, &["files"])
        .env(TESTING_ENV_VAR, "1")
        .start()
        .unwrap();

    s.wait().text("● files").until().unwrap();

    // the query line is the first row: no prompt, 1-column margin
    let frame = stable_frame(&s);
    let query_row = frame.lines().next().unwrap();
    assert!(
        !query_row.contains('>') && !query_row.contains('▎'),
        "Expected no prompt or marker on the query row:\n{}",
        frame
    );

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --ui-scale adjusts the overall interface size.
#[test]
fn test_ui_scale() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "files",
            "--ui-scale",
            "80",
            "--results-border",
            "rounded",
            "--preview-border",
            "rounded",
        ],
    )
    .start()
    .unwrap();

    // match the scaled results box top border (48 columns at 80% of 120)
    s.wait()
        .text("╭─────────── Default ⟨ ● ○ ⟩ ctrl-s ───────────╮")
        .until()
        .unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --no-remote hides the remote control panel.
#[test]
fn test_no_remote_hides_remote_panel() {
    let pt = phantom();

    let s =
        tv_local_config_and_cable_with_args(&pt, &["files", "--no-remote"])
            .start()
            .unwrap();

    s.wait().text("● files").until().unwrap();

    // with the remote disabled, ctrl-t is a no-op
    s.send().key("ctrl-t").unwrap();
    assert_frame_not_contains(&s, "● channels");

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --hide-status-bar starts the interface with the status bar hidden.
#[test]
fn test_hide_status_bar_flag_hides_status_bar() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--hide-status-bar"],
    )
    .start()
    .unwrap();

    // with the status bar hidden, the channel name moves next to the count
    s.wait().text("· files").until().unwrap();
    assert_frame_not_contains(&s, "● files");

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --show-remote starts the interface with the remote control panel visible.
#[test]
fn test_show_remote_flag_shows_remote_panel() {
    let pt = phantom();

    let s =
        tv_local_config_and_cable_with_args(&pt, &["files", "--show-remote"])
            .start()
            .unwrap();

    s.wait().text("● channels").until().unwrap();

    // Send Ctrl+C to exit remote control; wait for it to close before
    // sending the app-level quit to avoid races.
    s.send().key("ctrl-c").unwrap();
    s.wait().text_absent("● channels").until().unwrap();
    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --hide-remote prevents the remote control panel from showing at startup.
#[test]
fn test_hide_remote_flag_hides_remote_panel() {
    let pt = phantom();

    let s =
        tv_local_config_and_cable_with_args(&pt, &["files", "--hide-remote"])
            .start()
            .unwrap();

    s.wait().text("● files").until().unwrap();
    assert_frame_not_contains(&s, "● channels");

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --hide-remote conflicts with --no-remote.
#[test]
fn test_hide_remote_conflicts_with_no_remote() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--hide-remote", "--no-remote"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

/// Tests that --hide-remote and --show-remote cannot be used together.
#[test]
fn test_hide_and_show_remote_conflict_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--hide-remote", "--show-remote"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

/// Tests that --no-help-panel disables the help panel entirely.
#[test]
fn test_no_help_panel_disables_help_panel() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--no-help-panel"],
    )
    .start()
    .unwrap();

    s.wait().text("● files").until().unwrap();

    // Send Ctrl+H to try to open help panel (should not work)
    s.send().key("ctrl-h").unwrap();

    assert_frame_not_contains(&s, "▏ help");

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --hide-help-panel starts the interface with the help panel hidden.
#[test]
fn test_hide_help_panel_starts_with_help_hidden() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--hide-help-panel"],
    )
    .start()
    .unwrap();

    s.wait().text("● files").until().unwrap();

    // Send Ctrl+H to open help panel (should still work since it's just hidden)
    s.send().key("ctrl-h").unwrap();

    s.wait().text("▏ help").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --show-help-panel ensures the help panel is visible.
#[test]
fn test_show_help_panel_starts_with_help_visible() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--show-help-panel"],
    )
    .start()
    .unwrap();

    s.wait().text("▏ help").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --hide-help-panel conflicts with --no-help-panel.
#[test]
fn test_hide_help_panel_conflicts_with_no_help_panel() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--hide-help-panel", "--no-help-panel"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

/// Tests that --hide-help-panel and --show-help-panel cannot be used together.
#[test]
fn test_hide_and_show_help_panel_conflict_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--hide-help-panel", "--show-help-panel"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

/// Tests that --no-help-panel conflicts with --show-help-panel.
#[test]
fn test_no_help_panel_conflicts_with_show_help_panel() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--no-help-panel", "--show-help-panel"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

#[test]
fn test_tui_with_height_and_width() {
    let pt = phantom();

    // explicit UI flags should win over the minimal non-fullscreen preset,
    // and restoring the chrome keeps the frame-dimension assertions below
    // meaningful
    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "files",
            "--height",
            "20",
            "--width",
            "80",
            "--show-status-bar",
            "--results-border",
            "rounded",
            "--preview-border",
            "rounded",
        ],
    )
    .env(TESTING_ENV_VAR, "1")
    .start()
    .unwrap();

    s.wait().text("● files").until().unwrap();

    // Validate frame dimensions (20 rows × 80 columns). Phantom's screenshot
    // pads every row to the full terminal width with spaces, so we trim
    // trailing whitespace before measuring.
    let frame = stable_frame(&s);
    let trimmed_lines: Vec<&str> = frame.lines().map(str::trim_end).collect();
    let non_empty_lines: Vec<&str> = trimmed_lines
        .iter()
        .copied()
        .filter(|l| !l.is_empty())
        .collect();
    assert_eq!(
        non_empty_lines.len(),
        20,
        "Expected 20 rows, got {}",
        non_empty_lines.len()
    );
    let max_width = non_empty_lines
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0);
    assert_eq!(max_width, 80, "Expected 80 columns, got {}", max_width);

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that non-fullscreen mode defaults to the minimal color-only UI:
/// no borders, no status bar, no prompt, and the channel name shown as a
/// dimmed hint next to the result count.
#[test]
fn test_height_defaults_to_minimal_ui() {
    let pt = phantom();

    let s =
        tv_local_config_and_cable_with_args(&pt, &["files", "--height", "20"])
            .env(TESTING_ENV_VAR, "1")
            .start()
            .unwrap();

    // channel hint next to the result count
    s.wait().text("· files").until().unwrap();

    // no status bar, no borders, no results title (not even the
    // multi-source indicator), no prompt symbol
    assert_frame_not_contains_any(
        &s,
        &["CHANNEL", "╭", "─ files ─", " Results ", "⟨", "> "],
    );

    // the preview is separated from the results by a thin hairline
    let frame = stable_frame(&s);
    assert!(
        frame.contains('▏'),
        "Expected a preview separator in the frame:\n{}",
        frame
    );

    // without borders, the results should fill the whole viewport height
    // (there are far more than 18 entries in this repository)
    for (i, line) in frame.lines().take(20).enumerate() {
        let results_column = line.split('▏').next().unwrap();
        // row 2 is the blank line separating the input from the results
        if i == 1 {
            assert!(
                results_column.trim().is_empty(),
                "Row 2 should be a blank separator line:\n{}",
                frame
            );
            continue;
        }
        assert!(
            !results_column.trim().is_empty(),
            "Row {} of the results column should not be empty:\n{}",
            i + 1,
            frame
        );
    }

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that explicit UI flags win over the minimal non-fullscreen preset.
#[test]
fn test_height_minimal_ui_override() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--height", "20", "--results-border", "rounded"],
    )
    .env(TESTING_ENV_VAR, "1")
    .start()
    .unwrap();

    // the channel hint from the preset is still there
    s.wait().text("· files").until().unwrap();

    // the results panel gets its borders back
    let frame = stable_frame(&s);
    assert!(
        frame.contains('╭'),
        "Expected bordered results panel in the frame:\n{}",
        frame
    );

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests the minimal non-fullscreen UI in portrait orientation: the preview
/// sits below the results behind a horizontal hairline, with the preview
/// title embedded in the separator line.
#[test]
fn test_height_portrait_minimal_ui() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--height", "24", "--layout", "portrait"],
    )
    .env(TESTING_ENV_VAR, "1")
    .start()
    .unwrap();

    s.wait().text("· files").until().unwrap();

    // horizontal hairline with the preview title embedded in it, no borders
    let frame = stable_frame(&s);
    let separator_line = frame
        .lines()
        .find(|l| l.contains("────────"))
        .unwrap_or_else(|| {
            panic!(
                "Expected a horizontal preview separator in the frame:\n{}",
                frame
            )
        });
    assert!(
        separator_line.chars().any(char::is_alphanumeric),
        "Expected the preview title embedded in the separator line:\n{}",
        frame
    );
    assert_frame_not_contains_any(&s, &["CHANNEL", "╭", " Results ", "▔"]);

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that the preview is automatically hidden when the viewport is too
/// small to fit a useful preview pane below the results.
#[test]
fn test_minimal_ui_auto_hides_preview_when_cramped() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--height", "10", "--layout", "portrait"],
    )
    .env(TESTING_ENV_VAR, "1")
    .start()
    .unwrap();

    s.wait().text("· files").until().unwrap();

    // no room for a preview pane: no separator, results only
    assert_frame_not_contains_any(&s, &["────────", "▏"]);

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Same as above but in landscape: a narrow viewport leaves no room for a
/// useful preview pane next to the results.
#[test]
fn test_minimal_ui_auto_hides_preview_when_narrow() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--height", "20", "--width", "35"],
    )
    .env(TESTING_ENV_VAR, "1")
    .start()
    .unwrap();

    s.wait().text("· files").until().unwrap();

    assert_frame_not_contains_any(&s, &["▏"]);

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --no-preview disables the preview panel entirely.
#[test]
fn test_no_preview_disables_preview_panel() {
    let pt = phantom();

    let s =
        tv_local_config_and_cable_with_args(&pt, &["files", "--no-preview"])
            .start()
            .unwrap();
    s.wait().text("● files").until().unwrap();

    // Try to toggle preview - it shouldn't work since it's disabled entirely
    s.send().type_text("o").unwrap();

    assert_frame_not_contains_any(&s, &["▏", "Show Preview"]);

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --show-preview starts the interface with the preview panel visible.
#[test]
fn test_show_preview_starts_with_preview_visible() {
    let pt = phantom();

    let s =
        tv_local_config_and_cable_with_args(&pt, &["files", "--show-preview"])
            .start()
            .unwrap();

    s.wait().text("▏").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --no-status-bar disables the status bar entirely.
#[test]
fn test_no_status_bar_disables_status_bar() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--no-status-bar"],
    )
    .start()
    .unwrap();
    s.wait().text("· files").until().unwrap();

    assert_frame_not_contains(&s, "● files");

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --show-status-bar starts the interface with the status bar visible.
#[test]
fn test_show_status_bar_starts_with_status_visible() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--show-status-bar"],
    )
    .start()
    .unwrap();

    s.wait().text("● files").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --hide-preview-scrollbar hides the preview panel scrollbar.
#[test]
fn test_hide_preview_scrollbar_hides_scrollbar() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "files",
            "--hide-preview-scrollbar",
            "--preview-border",
            "rounded",
        ],
    )
    .start()
    .unwrap();

    s.wait().text("──╮").until().unwrap();
    assert_frame_not_contains(&s, "▲");

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --no-preview conflicts with preview-related flags.
#[test]
fn test_no_preview_conflicts_with_preview_flags() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--no-preview", "--preview-command", "cat {}"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

/// Tests that --no-status-bar conflicts with status-bar-related flags.
#[test]
fn test_no_status_bar_conflicts_with_status_bar_flags() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--no-status-bar", "--show-status-bar"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

#[test]
// FIXME: needs https://github.com/crossterm-rs/crossterm/pull/957
#[ignore = "needs https://github.com/crossterm-rs/crossterm/pull/957"]
fn test_tui_with_height_only() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "files",
            "--height",
            "15",
            "--show-status-bar",
            "--results-border",
            "rounded",
        ],
    )
    .env(TESTING_ENV_VAR, "1")
    .start()
    .unwrap();

    s.wait().text("● files").until().unwrap();

    let frame = stable_frame(&s);
    let non_empty_lines: Vec<&str> = frame
        .lines()
        .map(str::trim_end)
        .filter(|l| !l.is_empty())
        .collect();
    assert_eq!(
        non_empty_lines.len(),
        15,
        "Expected 15 rows, got {}",
        non_empty_lines.len()
    );

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that in minimal mode the remote control and the actions picker
/// take over the main picker area instead of opening popups.
#[test]
fn test_height_minimal_picker_takeover() {
    let pt = phantom();

    let s =
        tv_local_config_and_cable_with_args(&pt, &["files", "--height", "16"])
            .env(TESTING_ENV_VAR, "1")
            .start()
            .unwrap();

    s.wait().text("· files").until().unwrap();

    // remote control takes over in place (no popup, no logo)
    s.send().key("ctrl-t").unwrap();
    s.wait().text("· channels").until().unwrap();
    assert_frame_not_contains_any(&s, &["╭", " Channels "]);

    // esc returns to the channel picker
    s.send().key("esc").unwrap();
    s.wait().text("· files").until().unwrap();

    // the actions picker borrows the preview pane: the channel picker (and
    // the entry the action applies to) stays visible next to it
    s.send().key("ctrl-x").unwrap();
    s.wait().text("· actions").text("· files").until().unwrap();
    assert_frame_not_contains_any(&s, &["╭", " Actions ", " Search "]);

    s.send().key("esc").unwrap();
    s.wait().text_absent("· actions").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that multi-source channels show the current source name and dots
/// next to the result count, plus a cycle hint in the status bar.
#[test]
fn test_multi_source_indicator_next_to_count() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(&pt, &["files"])
        .env(TESTING_ENV_VAR, "1")
        .start()
        .unwrap();

    // the files channel has two sources; the active one is "Default"
    s.wait().text("· ● ○ Default").until().unwrap();
    s.wait().text("source ctrl-s").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}
