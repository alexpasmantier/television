//! Input bar, status bar, help panel and other UI elements.

use television::tui::TESTING_ENV_VAR;

use crate::common::*;

#[test]
fn test_toggle_status_bar_keybinding() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--keybindings", "ctrl-k = \"toggle_status_bar\""],
    )
    .start()
    .unwrap();
    s.wait().text("● files").until().unwrap();

    // Send Ctrl+K to toggle status bar off
    s.send().key("ctrl-k").unwrap();

    s.wait().text_absent("● files").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_toggle_help_keybinding() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(&pt, &["files"])
        .start()
        .unwrap();
    s.wait().text("● files").until().unwrap();

    // Send Ctrl+H to open help panel
    s.send().key("ctrl-h").unwrap();

    s.wait().text("▏ help").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

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
