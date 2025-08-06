//! Tests for CLI source/data options: --source-command, --source-display, --source-output.
//!
//! These tests verify Television's source command functionality, which is fundamental to
//! generating and formatting the data that users interact with. Source commands define
//! what entries are available for selection and how they're processed.

use super::super::common::*;

/// Tests that --source-command works properly in Ad-hoc Mode.
#[test]
fn test_source_command_in_adhoc_mode() {
    let mut tester = PtyTester::new();

    // This creates an Ad-hoc Mode channel that lists files in the current directory
    let cmd = tv_local_config_and_cable_with_args(&["--source-command", "ls"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify we're in Ad-hoc Mode with the custom source command active
    tester.assert_tui_frame_contains("CHANNEL  Custom");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that --source-command can override channel defaults in Channel Mode.
#[test]
fn test_source_command_override_in_channel_mode() {
    let mut tester = PtyTester::new();

    // This overrides the files channel's default source command with a custom one
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--source-command",
        "fd -t f . ./cable/",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the override is active
    tester.assert_tui_frame_contains("./cable/unix/files.toml");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that --source-display formats how entries appear in the results list.
#[test]
fn test_source_display_with_source_command() {
    let mut tester = PtyTester::new();

    // This displays all entries in uppercase using the {upper} template operation
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--input",
        "television",
        "--source-display",
        "{upper}",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify entries are displayed in uppercase format
    tester.assert_tui_frame_contains("TELEVISION");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that --source-output formats the final output when an entry is selected.
#[test]
fn test_source_output_with_source_command() {
    let mut tester = PtyTester::new();

    // This should auto-select "UNIQUE16CHARID" since it's the only result
    let cmd = tv_local_config_and_cable_with_args(&[
        "--source-command",
        "echo UNIQUE16CHARID",
        "--source-output",
        "echo AA{}BB",
        "--take-1",
    ]);
    tester.spawn_command(cmd);

    // Verify the output contains the selected entry
    tester.assert_raw_output_contains("AAUNIQUE16CHARIDBB");
}

/// Tests that --source-display requires --source-command in Ad-hoc Mode.
#[test]
fn test_source_display_without_source_command_errors() {
    let mut tester = PtyTester::new();

    // This should fail because there's no source command to provide entries to format
    let cmd =
        tv_local_config_and_cable_with_args(&["--source-display", "{upper}"]);
    tester.spawn_command(cmd);

    // CLI should exit with error message, not show TUI
    tester.assert_raw_output_contains(
        "source-display requires a source command",
    );
}

/// Tests that --source-output requires --source-command in Ad-hoc Mode.
#[test]
fn test_source_output_without_source_command_errors() {
    let mut tester = PtyTester::new();

    // This should fail because there's no source command to generate entries for output
    let cmd =
        tv_local_config_and_cable_with_args(&["--source-output", "echo {}"]);
    tester.spawn_command(cmd);

    // CLI should exit with error message, not show TUI
    tester
        .assert_raw_output_contains("source-output requires a source command");
}
