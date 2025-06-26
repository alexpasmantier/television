//! Tests for CLI monitoring options: --watch.
//!
//! These tests verify Television's live monitoring capabilities,
//! ensuring users can enable real-time data updates.

#[path = "../common/mod.rs"]
mod common;
use common::*;

/// Tests that --watch enables live monitoring with automatic source command re-execution.
#[test]
fn test_watch_reloads_source_command() {
    let mut tester = PtyTester::new();
    let tmp_dir = std::env::temp_dir();

    // Create initial file to be detected
    std::fs::write(tmp_dir.join("file1.txt"), "").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(200));

    // This monitors the temp directory and updates every 0.5 seconds
    let cmd = tv_local_config_and_cable_with_args(&[
        "--watch",
        "0.5",
        "--source-command",
        "ls",
        tmp_dir.to_str().unwrap(),
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the initial file is detected
    tester.assert_tui_frame_contains("file1.txt");

    // Create a second file
    std::fs::write(tmp_dir.join("control.txt"), "").unwrap();

    // Verify the new file does not appear immediately
    tester.assert_not_tui_frame_contains("control.txt");

    // Wait longer than watch interval
    std::thread::sleep(std::time::Duration::from_millis(700));

    // Verify the new file appears after the watch interval
    tester.assert_tui_frame_contains("control.txt");

    // Remove the control file
    if std::fs::metadata(tmp_dir.join("control.txt")).is_ok() {
        std::fs::remove_file(tmp_dir.join("control.txt")).unwrap();
    }

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

/// Tests that --tick-rate accepts a valid positive number.
#[test]
fn test_tick_rate_valid_value_starts_application() {
    let mut tester = PtyTester::new();

    // Start Television with a custom tick rate
    let cmd =
        tv_local_config_and_cable_with_args(&["files", "--tick-rate", "30"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the TUI launched successfully
    tester.assert_tui_frame_contains("CHANNEL  files");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

/// Tests that --tick-rate rejects non-positive numbers.
#[test]
fn test_tick_rate_invalid_value_errors() {
    let mut tester = PtyTester::new();

    // Use an invalid tick rate value
    let cmd =
        tv_local_config_and_cable_with_args(&["files", "--tick-rate", "-5"]);
    tester.spawn_command(cmd);

    // CLI should exit with error message, not show TUI
    tester.assert_raw_output_contains("unexpected argument");
}
