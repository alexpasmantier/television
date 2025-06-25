//! Tests for CLI special modes: --autocomplete-prompt and related conflicts.
//!
//! These tests verify Television's intelligent channel detection and shell integration
//! features, ensuring that the autocomplete prompt can automatically select appropriate
//! channels based on command analysis.

#[path = "../common/mod.rs"]
mod common;
use common::*;

/// Tests that --autocomplete-prompt automatically detects and activates the appropriate channel.
#[test]
fn test_autocomplete_prompt_activates_channel_mode() {
    let mut tester = PtyTester::new();

    // This should analyze the git command and automatically select the git-log channel
    let cmd = tv_local_config_and_cable_with_args(&[
        "--autocomplete-prompt",
        "git log --oneline",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the git-log channel was automatically detected and activated
    tester.assert_tui_frame_contains("CHANNEL  git-log");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

/// Tests that --autocomplete-prompt conflicts with explicit channel argument.
#[test]
fn test_autocomplete_prompt_and_channel_argument_conflict_errors() {
    let mut tester = PtyTester::new();

    // This should fail because both specify how to choose a channel
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--autocomplete-prompt",
        "git log --oneline",
    ]);
    tester.spawn_command(cmd);

    // CLI should exit with error message, not show TUI
    tester.assert_raw_output_contains("cannot be used with");
}
