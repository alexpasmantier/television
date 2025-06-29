//! Tests for CLI UI/layout options: --layout, --input-header, --ui-scale, --no-remote, --no-status-bar.
//!
//! These tests verify Television's user interface customization capabilities,
//! ensuring users can adapt the layout and appearance to their preferences and needs.

use super::common::*;

/// Tests that --layout landscape arranges panels side-by-side.
#[test]
fn test_layout_landscape() {
    let mut tester = PtyTester::new();

    // This sets the interface to use side-by-side panel arrangement
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--layout",
        "landscape",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the TUI started successfully in landscape layout
    tester.assert_tui_frame_contains(
        "╭───────────────────────── files ──────────────────────────╮╭─",
    ); // Should be in landscape layout

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

/// Tests that --layout portrait arranges panels vertically stacked.
#[test]
fn test_layout_portrait() {
    let mut tester = PtyTester::new();

    // This sets the interface to use vertically stacked panel arrangement
    let cmd = tv_local_config_and_cable_with_args(&[
        "files", "--layout", "portrait",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the TUI started successfully in portrait layout
    tester.assert_tui_frame_contains("╭─────────────────────────────────────────────────────── files ────────────────────────────────────────────────────────╮");
    tester.assert_tui_frame_contains("Hide Preview"); // Should be in portrait layout

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

/// Tests that --input-header customizes the text above the search input in Channel Mode.
#[test]
fn test_input_header_in_channel_mode() {
    let mut tester = PtyTester::new();

    // This overrides the channel's default input header with custom text
    let mut cmd = tv_local_config_and_cable_with_args(&["files"]);
    cmd.args(["--input-header", "UNIQUE16CHARID"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the custom input header is displayed
    tester.assert_tui_frame_contains("UNIQUE16CHARID");
    tester.assert_tui_frame_contains("CHANNEL  files");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

/// Tests that --input-header works in Ad-hoc Mode.
#[test]
fn test_input_header_in_adhoc_mode() {
    let mut tester = PtyTester::new();

    // This provides a custom input header for an ad-hoc channel
    let mut cmd =
        tv_local_config_and_cable_with_args(&["--source-command", "ls"]);
    cmd.args(["--input-header", "UNIQUE16CHARID"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the custom input header is displayed
    tester.assert_tui_frame_contains("UNIQUE16CHARID");
    tester.assert_tui_frame_contains("CHANNEL  custom");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

/// Tests that --ui-scale adjusts the overall interface size.
#[test]
fn test_ui_scale() {
    let mut tester = PtyTester::new();

    // This scales the entire interface to 80% of normal size
    let cmd =
        tv_local_config_and_cable_with_args(&["files", "--ui-scale", "80"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the interface is scaled (smaller panels visible in output)
    tester.assert_tui_frame_contains(
        "╭─────────────────── files ────────────────────╮╭─",
    );

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

/// Tests that --no-remote hides the remote control panel.
#[test]
fn test_no_remote_hides_remote_panel() {
    let mut tester = PtyTester::new();

    // This disables the remote control panel display
    let cmd = tv_local_config_and_cable_with_args(&["files", "--no-remote"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the remote control panel is hidden
    tester.assert_not_tui_frame_contains("── Channels ──");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

/// Tests that --no-status-bar hides the bottom status bar.
#[test]
fn test_no_status_bar_hides_status_bar() {
    let mut tester = PtyTester::new();

    // This disables the bottom status bar display
    let cmd =
        tv_local_config_and_cable_with_args(&["files", "--no-status-bar"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the status bar is hidden
    tester.assert_not_tui_frame_contains("CHANNEL  files");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

/// Tests that --hide-status-bar starts the interface with the status bar hidden.
#[test]
fn test_hide_status_bar_flag_hides_status_bar() {
    let mut tester = PtyTester::new();

    // Start with the files channel and hide the status bar
    let cmd =
        tv_local_config_and_cable_with_args(&["files", "--hide-status-bar"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the status bar is hidden
    tester.assert_not_tui_frame_contains("CHANNEL  files");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

/// Tests that --show-status-bar ensures the status bar is visible.
#[test]
fn test_show_status_bar_flag_shows_status_bar() {
    let mut tester = PtyTester::new();

    // Start with the files channel and force the status bar visible
    let cmd =
        tv_local_config_and_cable_with_args(&["files", "--show-status-bar"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the status bar is visible
    tester.assert_tui_frame_contains("CHANNEL  files");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

/// Tests that --hide-status-bar conflicts with --no-status-bar.
#[test]
fn test_hide_status_bar_conflicts_with_no_status_bar() {
    let mut tester = PtyTester::new();

    // This should fail because the flags are mutually exclusive
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--hide-status-bar",
        "--no-status-bar",
    ]);
    tester.spawn_command(cmd);

    // CLI should exit with error message, not show TUI
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that --hide-status-bar and --show-status-bar cannot be used together.
#[test]
fn test_hide_and_show_status_bar_conflict_errors() {
    let mut tester = PtyTester::new();

    // This should fail because the flags are mutually exclusive
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--hide-status-bar",
        "--show-status-bar",
    ]);
    tester.spawn_command(cmd);

    // CLI should exit with error message, not show TUI
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that --show-remote starts the interface with the remote control panel visible.
#[test]
fn test_show_remote_flag_shows_remote_panel() {
    let mut tester = PtyTester::new();

    // Start with the files channel and show the remote control panel
    let cmd = tv_local_config_and_cable_with_args(&["files", "--show-remote"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the remote control panel is visible (channel indicators)
    tester.assert_tui_frame_contains("(1) (2) (3)");

    // Send Ctrl+C to remote control
    tester.send(&ctrl('c'));

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

/// Tests that --hide-remote prevents the remote control panel from showing at startup.
#[test]
fn test_hide_remote_flag_hides_remote_panel() {
    let mut tester = PtyTester::new();

    // Start with the files channel and hide the remote control panel
    let cmd = tv_local_config_and_cable_with_args(&["files", "--hide-remote"]);
    let mut child = tester.spawn_command_tui(cmd);

    tester.assert_not_tui_frame_contains("(1) (2) (3)");
    tester.assert_not_tui_frame_contains("── Channels ──");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

/// Tests that --hide-remote conflicts with --no-remote.
#[test]
fn test_hide_remote_conflicts_with_no_remote() {
    let mut tester = PtyTester::new();

    // This should fail because the flags are mutually exclusive
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--hide-remote",
        "--no-remote",
    ]);
    tester.spawn_command(cmd);

    // CLI should exit with error message, not show TUI
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that --hide-remote and --show-remote cannot be used together.
#[test]
fn test_hide_and_show_remote_conflict_errors() {
    let mut tester = PtyTester::new();

    // This should fail because the flags are mutually exclusive
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--hide-remote",
        "--show-remote",
    ]);
    tester.spawn_command(cmd);

    // CLI should exit with error message, not show TUI
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that --no-help-panel disables the help panel entirely.
#[test]
fn test_no_help_panel_disables_help_panel() {
    let mut tester = PtyTester::new();

    // This disables the help panel entirely
    let cmd =
        tv_local_config_and_cable_with_args(&["files", "--no-help-panel"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Send Ctrl+H to try to open help panel (should not work)
    tester.send(&ctrl('h'));

    // Verify help panel is not displayed
    tester.assert_not_tui_frame_contains("───── Help ─────");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

/// Tests that --hide-help-panel starts the interface with the help panel hidden.
#[test]
fn test_hide_help_panel_starts_with_help_hidden() {
    let mut tester = PtyTester::new();

    // Start with the files channel and hide the help panel
    let cmd =
        tv_local_config_and_cable_with_args(&["files", "--hide-help-panel"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Send Ctrl+H to open help panel (should still work since it's just hidden)
    tester.send(&ctrl('h'));

    // Verify help panel can be toggled visible
    tester.assert_tui_frame_contains("───── Help ─────");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

/// Tests that --show-help-panel ensures the help panel is visible.
#[test]
fn test_show_help_panel_starts_with_help_visible() {
    let mut tester = PtyTester::new();

    // Start with the files channel and force the help panel visible
    let cmd =
        tv_local_config_and_cable_with_args(&["files", "--show-help-panel"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify help panel is initially visible
    tester.assert_tui_frame_contains("───── Help ─────");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

/// Tests that --hide-help-panel conflicts with --no-help-panel.
#[test]
fn test_hide_help_panel_conflicts_with_no_help_panel() {
    let mut tester = PtyTester::new();

    // This should fail because the flags are mutually exclusive
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--hide-help-panel",
        "--no-help-panel",
    ]);
    tester.spawn_command(cmd);

    // CLI should exit with error message, not show TUI
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that --hide-help-panel and --show-help-panel cannot be used together.
#[test]
fn test_hide_and_show_help_panel_conflict_errors() {
    let mut tester = PtyTester::new();

    // This should fail because the flags are mutually exclusive
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--hide-help-panel",
        "--show-help-panel",
    ]);
    tester.spawn_command(cmd);

    // CLI should exit with error message, not show TUI
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that --no-help-panel conflicts with --show-help-panel.
#[test]
fn test_no_help_panel_conflicts_with_show_help_panel() {
    let mut tester = PtyTester::new();

    // This should fail because the flags are mutually exclusive
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--no-help-panel",
        "--show-help-panel",
    ]);
    tester.spawn_command(cmd);

    // CLI should exit with error message, not show TUI
    tester.assert_raw_output_contains("cannot be used with");
}
