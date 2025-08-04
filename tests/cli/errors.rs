//! Tests for CLI mutually exclusive/dependency error cases for all flags.
//!
//! These tests ensure Television properly validates CLI arguments and provides clear
//! error messages when users specify incompatible or invalid flag combinations.
//! This is critical for user experience and preventing unexpected behavior.

use television::tui::TESTING_ENV_VAR;

use super::super::common::*;

/// Tests that preview flags without --preview-command fail in Ad-hoc Mode.
#[test]
fn test_preview_flags_without_preview_command_in_adhoc_mode_errors() {
    let mut tester = PtyTester::new();

    // This should fail because Ad-hoc Mode requires --preview-command for preview flags
    let cmd =
        tv_local_config_and_cable_with_args(&["--preview-header", "HEADER"]);
    tester.spawn_command(cmd);

    // CLI should exit with error message, not show TUI
    // This validates that the dependency checking works correctly
    tester.assert_raw_output_contains(
        "preview-header requires a preview command",
    );
}

/// Tests that --source-display without --source-command fails in Ad-hoc Mode.
#[test]
fn test_source_display_without_source_command_in_adhoc_mode_errors() {
    let mut tester = PtyTester::new();

    // This should fail because there's no source command to provide data to format
    let cmd =
        tv_local_config_and_cable_with_args(&["--source-display", "{upper}"]);
    tester.spawn_command(cmd);

    // Verify the appropriate error message is shown
    tester.assert_raw_output_contains(
        "source-display requires a source command",
    );
}

/// Tests that --source-output without --source-command fails in Ad-hoc Mode.
#[test]
fn test_source_output_without_source_command_in_adhoc_mode_errors() {
    let mut tester = PtyTester::new();

    // This fails because there's no source command to generate entries for output formatting
    let cmd =
        tv_local_config_and_cable_with_args(&["--source-output", "echo {}"]);
    tester.spawn_command(cmd);

    // Confirm the dependency error is properly reported
    tester
        .assert_raw_output_contains("source-output requires a source command");
}

/// Tests that multiple selection flags cannot be used together.
#[test]
fn test_multiple_selection_flags_conflict_errors() {
    let mut tester = PtyTester::new();

    // This should fail because --select-1 and --take-1 have conflicting behaviors
    let cmd = tv_local_config_and_cable_with_args(&[
        "--source-command",
        "ls",
        "--select-1",
        "--take-1",
    ]);
    tester.spawn_command(cmd);

    // Verify the conflict is detected and reported
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that --no-preview conflicts with preview-related flags.
#[test]
fn test_preview_and_no_preview_conflict_errors() {
    let mut tester = PtyTester::new();

    // This should fail because --no-preview disables the preview panel entirely
    let cmd = tv_local_config_and_cable_with_args(&[
        "--source-command",
        "ls",
        "--no-preview",
        "--preview-header",
        "HEADER",
    ]);
    tester.spawn_command(cmd);

    // Confirm the logical conflict is caught
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that channel argument and --autocomplete-prompt cannot be used together.
#[test]
fn test_channel_and_autocomplete_prompt_conflict_errors() {
    let mut tester = PtyTester::new();

    // This should fail because both specify a channel selection method
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--autocomplete-prompt",
        "git log --oneline",
    ]);
    tester.spawn_command(cmd);

    // Verify the ambiguous channel selection is rejected
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that --watch conflicts with auto-selection flags.
#[test]
fn test_watch_and_selection_flags_conflict_errors() {
    let mut tester = PtyTester::new();

    // This should fail because watch mode continuously updates while --select-1 would exit immediately
    let cmd = tv_local_config_and_cable_with_args(&[
        "--source-command",
        "ls",
        "--watch",
        "1.0",
        "--select-1",
    ]);
    tester.spawn_command(cmd);

    // Confirm the logical incompatibility is detected
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that --inline conflicts with --height.
#[test]
fn test_inline_and_height_conflict_errors() {
    unsafe { std::env::set_var(TESTING_ENV_VAR, "1") };
    let mut tester = PtyTester::new();

    // This should fail because --inline and --height are mutually exclusive
    let cmd = tv_local_config_and_cable_with_args(&[
        "files", "--inline", "--height", "20",
    ]);
    tester.spawn_command(cmd);

    // Confirm the logical incompatibility is detected
    tester.assert_raw_output_contains("cannot be used with");
    unsafe { std::env::remove_var(TESTING_ENV_VAR) };
}

/// Tests that --width cannot be used without --height or --inline.
#[test]
fn test_width_without_height_or_inline_errors() {
    unsafe { std::env::set_var(TESTING_ENV_VAR, "1") };
    let mut tester = PtyTester::new();

    // This should fail because --width requires --height or --inline
    let cmd = tv_local_config_and_cable_with_args(&["files", "--width", "80"]);
    tester.spawn_command(cmd);

    // Confirm the logical incompatibility is detected
    tester.assert_raw_output_contains("can only be used");
    unsafe { std::env::remove_var(TESTING_ENV_VAR) };
}
