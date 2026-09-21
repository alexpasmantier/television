//! Layout and geometry: `--layout`, `--ui-scale`, `--height`, `--width`, `--inline`, minimal UI.

use television::tui::TESTING_ENV_VAR;

use crate::common::*;

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
        &[
            "files",
            "--height",
            "20",
            "--width",
            "35",
            "--input",
            "Cargo.toml",
        ],
    )
    .env(TESTING_ENV_VAR, "1")
    .start()
    .unwrap();

    s.wait().text("Cargo.toml").until().unwrap();

    assert_frame_not_contains_any(&s, &["▏"]);

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
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

/// Tests that on a narrow viewport the count line drops the source
/// indicator as a whole instead of starving the query field or clipping
/// itself mid-segment.
#[test]
fn test_narrow_input_keeps_query_visible() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(&pt, &["files"])
        .env(TESTING_ENV_VAR, "1")
        .size(30, 20)
        .start()
        .unwrap();

    s.wait().stable(500).until().unwrap();

    s.send().type_text("changelog").unwrap();
    s.wait().text("changelog").until().unwrap();
    assert_frame_not_contains(&s, "● ○ Default");

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_inline_and_height_conflict_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--inline", "--height", "20"],
    )
    .env(TESTING_ENV_VAR, "1")
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

#[test]
fn test_width_without_height_or_inline_errors() {
    let pt = phantom();

    let s =
        tv_local_config_and_cable_with_args(&pt, &["files", "--width", "80"])
            .env(TESTING_ENV_VAR, "1")
            .start()
            .unwrap();

    s.wait().text("can only be used").until().unwrap();
}
