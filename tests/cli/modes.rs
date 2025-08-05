//! Tests for CLI operating modes, path detection, and mode switching.
//!
//! These tests verify Television's two primary operating modes (Channel Mode and Ad-hoc Mode)
//! and the intelligent path detection logic that automatically switches between them.
//! This is fundamental to how Television interprets CLI arguments.

use super::super::common::*;

/// Tests that basic Channel Mode activation works with a channel name.
#[test]
fn test_channel_mode_with_channel_name() {
    let mut tester = PtyTester::new();

    // This should activate Channel Mode using the "dirs" channel definition
    let mut child = tester
        .spawn_command_tui(tv_local_config_and_cable_with_args(&["dirs"]));

    // Verify we're in Channel Mode with the dirs channel active
    tester.assert_tui_frame_contains(
        "╭────────────────────────── dirs ──────────────────────────╮",
    );
    tester.assert_tui_frame_contains("CHANNEL  dirs");

    // Send Ctrl+C to exit the application
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that Channel Mode works with both channel name and working directory specified.
#[test]
fn test_channel_mode_with_channel_and_path() {
    let mut tester = PtyTester::new();

    // This should use the dirs channel in the cable directory
    let mut child =
        tester.spawn_command_tui(tv_local_config_and_cable_with_args(&[
            "dirs", "./cable/",
        ]));

    // Verify we're in Channel Mode with the dirs channel active
    tester.assert_tui_frame_contains(
        "╭────────────────────────── dirs ──────────────────────────╮",
    );
    tester.assert_tui_frame_contains("CHANNEL  dirs");
    tester.assert_tui_frame_contains("unix/");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that CLI flags can override channel defaults in Channel Mode.
#[test]
fn test_channel_mode_with_channel_and_overrides() {
    let mut tester = PtyTester::new();

    // This should use the files channel but override its default input header
    let mut cmd = tv_local_config_and_cable_with_args(&["files"]);
    cmd.args(["--input-header", "UNIQUE16CHARID"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the override took effect
    tester.assert_tui_frame_contains("UNIQUE16CHARID");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that basic Ad-hoc Mode activation works with --source-command.
#[test]
fn test_adhoc_mode_with_source_command() {
    let mut tester = PtyTester::new();

    // This should activate Ad-hoc Mode since no channel is specified
    let cmd = tv_local_config_and_cable_with_args(&["--source-command", "ls"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify we're in Ad-hoc Mode (shows "custom" instead of a channel name)
    tester.assert_tui_frame_contains("CHANNEL  Custom");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that Ad-hoc Mode requires --source-command for dependent flags.
#[test]
fn test_adhoc_mode_missing_source_command_errors() {
    let mut tester = PtyTester::new();

    // This should fail because Ad-hoc Mode requires --source-command for any source-related flags
    let cmd = tv_local_config_and_cable_with_args(&["--source-display", "{}"]);
    tester.spawn_command(cmd);

    // Verify the dependency error is reported
    tester.assert_raw_output_contains(
        "source-display requires a source command",
    );
}

/// Tests that smart path detection automatically switches to Channel Mode.
#[test]
fn test_smart_path_detection_switches_to_adhoc_mode() {
    let mut tester = PtyTester::new();

    // The "./cable/" should be detected as a path, triggering path detection logic
    // This uses the default channel (files) in the cable directory
    let cmd = tv_local_config_and_cable_with_args(&["./cable/"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify path detection worked and default channel was used
    tester.assert_tui_frame_contains("CHANNEL  files");
    tester.assert_tui_frame_contains("unix/files.toml");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that fallback to default channel works when no arguments are provided.
#[test]
fn test_no_arguments_uses_default_channel() {
    let mut tester = PtyTester::new();

    // This should use the default_channel from the configuration file
    let mut child =
        tester.spawn_command_tui(tv_local_config_and_cable_with_args(&[]));

    // Verify the default channel (files) was loaded
    tester.assert_tui_frame_contains("CHANNEL  files");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}
