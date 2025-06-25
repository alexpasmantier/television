//! Tests for CLI UI/layout options: --layout, --input-header, --ui-scale, --no-remote, --no-status-bar.
//!
//! These tests verify Television's user interface customization capabilities,
//! ensuring users can adapt the layout and appearance to their preferences and needs.

#[path = "../common/mod.rs"]
mod common;
use common::*;

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
    tester.assert_not_tui_frame_contains("remote");

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
