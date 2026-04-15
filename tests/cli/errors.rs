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
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--preview-header", "HEADER"],
    )
    .start()
    .unwrap();

    // CLI should exit with error message, not show TUI
    s.wait()
        .text("preview-header requires a preview command")
        .until()
        .unwrap();
}

/// Tests that --source-display without --source-command fails in Ad-hoc Mode.
#[test]
fn test_source_display_without_source_command_in_adhoc_mode_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--source-display", "{upper}"],
    )
    .start()
    .unwrap();

    s.wait()
        .text("source-display requires a source command")
        .until()
        .unwrap();
}

/// Tests that --source-output without --source-command fails in Ad-hoc Mode.
#[test]
fn test_source_output_without_source_command_in_adhoc_mode_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--source-output", "echo {}"],
    )
    .start()
    .unwrap();

    s.wait()
        .text("source-output requires a source command")
        .until()
        .unwrap();
}

/// Tests that multiple selection flags cannot be used together.
#[test]
fn test_multiple_selection_flags_conflict_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--source-command", "ls", "--select-1", "--take-1"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

/// Tests that --no-preview conflicts with preview-related flags.
#[test]
fn test_preview_and_no_preview_conflict_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "--source-command",
            "ls",
            "--no-preview",
            "--preview-header",
            "HEADER",
        ],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

/// Tests that channel argument and --autocomplete-prompt cannot be used together.
#[test]
fn test_channel_and_autocomplete_prompt_conflict_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--autocomplete-prompt", "git log --oneline"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

/// Tests that --watch conflicts with auto-selection flags.
#[test]
fn test_watch_and_selection_flags_conflict_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--source-command", "ls", "--watch", "1.0", "--select-1"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

/// Tests that --inline conflicts with --height.
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

/// Tests that --width cannot be used without --height or --inline.
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
