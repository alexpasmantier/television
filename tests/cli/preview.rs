//! Tests for CLI preview options: --preview-*, --no-preview, and their combinations.
//!
//! These tests verify Television's preview panel functionality, ensuring users can
//! customize preview behavior and that conflicting options are properly detected.
//! Preview features are essential for examining file contents and command outputs.

use super::super::common::*;

/// Tests that --preview-command works in Ad-hoc Mode.
#[test]
fn test_preview_command_in_adhoc_mode() {
    let mut tester = PtyTester::new();

    // This creates an ad-hoc channel with both source and preview commands
    let cmd = tv_local_config_and_cable_with_args(&[
        "--source-command",
        "ls",
        "--preview-command",
        "cat {}",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the preview panel is displayed
    tester.assert_tui_frame_contains(
        "╭───────────────────── Custom Channel ─────────────────────╮╭─",
    );
    tester.assert_tui_frame_contains("Hide Preview");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that --preview-command can override channel defaults in Channel Mode.
#[test]
fn test_preview_command_override_in_channel_mode() {
    let mut tester = PtyTester::new();

    // This overrides the files channel's default preview command
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--preview-command",
        "cat {}",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the preview panel is active with the override
    tester.assert_tui_frame_contains(
        "╭───────────────────────── files ──────────────────────────╮╭─",
    );
    tester.assert_tui_frame_contains("Hide Preview");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that --preview-header displays custom text above the preview panel.
#[test]
fn test_preview_header_with_preview_command() {
    let mut tester = PtyTester::new();

    // This adds a custom header above the preview panel
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--preview-header",
        "UNIQUE16CHARID",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the custom header is displayed
    tester.assert_tui_frame_contains("UNIQUE16CHARID");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that --preview-footer displays custom text below the preview panel.
#[test]
fn test_preview_footer_with_preview_command() {
    let mut tester = PtyTester::new();

    // This adds a custom footer below the preview panel
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--preview-footer",
        "UNIQUE16CHARID",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the custom footer is displayed
    tester.assert_tui_frame_contains("UNIQUE16CHARID");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that --preview-offset controls the scroll position in preview content.
#[test]
fn test_preview_offset_with_preview_command() {
    let mut tester = PtyTester::new();

    // This sets the preview to start displaying from line 10
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--input",
        "CODE_OF_CONDUCT.md",
        "-p",
        "cat -n {}",
        "--preview-offset",
        "50",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the preview panel is active
    tester.assert_tui_frame_contains("││     50");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that --preview-size controls the width of the preview panel.
#[test]
fn test_preview_size_with_preview_command() {
    let mut tester = PtyTester::new();

    // This sets the preview panel to 60% of screen width
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--preview-size",
        "60",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the preview panel is active
    tester.assert_tui_frame_contains(
        "╭─────────────────── files ────────────────────╮",
    );

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that --no-preview completely disables the preview panel.
#[test]
fn test_no_preview_disables_preview_panel() {
    let mut tester = PtyTester::new();

    // This creates an Ad-hoc Mode channel with preview explicitly disabled
    let cmd = tv_local_config_and_cable_with_args(&[
        "--source-command",
        "ls",
        "--no-preview",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify no preview panel is displayed
    tester.assert_not_tui_frame_contains("─ Preview ─");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that --no-preview conflicts with --preview-command.
#[test]
fn test_no_preview_conflicts_with_preview_command() {
    let mut tester = PtyTester::new();

    // This should fail because --no-preview and --preview-command are contradictory
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--no-preview",
        "--preview-command",
        "cat {}",
    ]);
    tester.spawn_command(cmd);

    // CLI should exit with error message, not show TUI
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that --no-preview conflicts with --preview-header.
#[test]
fn test_no_preview_conflicts_with_preview_header() {
    let mut tester = PtyTester::new();

    // This should fail because headers don't make sense without a preview panel
    let cmd = tv_local_config_and_cable_with_args(&[
        "--source-command",
        "ls",
        "--no-preview",
        "--preview-header",
        "UNIQUE16CHARID",
    ]);
    tester.spawn_command(cmd);

    // Verify the logical conflict is detected
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that --no-preview conflicts with --preview-footer.
#[test]
fn test_no_preview_conflicts_with_preview_footer() {
    let mut tester = PtyTester::new();

    // This should fail because footers don't make sense without a preview panel
    let cmd = tv_local_config_and_cable_with_args(&[
        "--source-command",
        "ls",
        "--no-preview",
        "--preview-footer",
        "UNIQUE16CHARID",
    ]);
    tester.spawn_command(cmd);

    // Verify the logical conflict is detected
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that --no-preview conflicts with --preview-offset.
#[test]
fn test_no_preview_conflicts_with_preview_offset() {
    let mut tester = PtyTester::new();

    // This should fail because offsets don't make sense without a preview panel
    let cmd = tv_local_config_and_cable_with_args(&[
        "--source-command",
        "ls",
        "--no-preview",
        "--preview-offset",
        "10",
    ]);
    tester.spawn_command(cmd);

    // Verify the logical conflict is detected
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that --no-preview conflicts with --preview-size.
#[test]
fn test_no_preview_conflicts_with_preview_size() {
    let mut tester = PtyTester::new();

    // This should fail because sizing doesn't make sense without a preview panel
    let cmd = tv_local_config_and_cable_with_args(&[
        "--source-command",
        "ls",
        "--no-preview",
        "--preview-size",
        "60",
    ]);
    tester.spawn_command(cmd);

    // Verify the logical conflict is detected
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that preview flags require --preview-command in Ad-hoc Mode.
#[test]
fn test_preview_flags_without_preview_command_errors_in_adhoc_mode() {
    let mut tester = PtyTester::new();

    // This should fail because Ad-hoc Mode requires --preview-command for preview flags
    let cmd = tv_local_config_and_cable_with_args(&[
        "--source-command",
        "ls",
        "--preview-header",
        "HEADER",
    ]);
    tester.spawn_command(cmd);

    // Verify the dependency error is reported
    tester.assert_raw_output_contains(
        "preview-header requires a preview command",
    );
}

/// Tests that --hide-preview starts the interface with the preview panel hidden.
#[test]
fn test_hide_preview_flag_starts_with_preview_hidden() {
    let mut tester = PtyTester::new();

    // Start the files channel with the preview panel hidden
    let cmd =
        tv_local_config_and_cable_with_args(&["files", "--hide-preview"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the preview panel is hidden (shows "Show Preview:" prompt)
    tester.assert_tui_frame_contains("╭─────────────────────────────────────────────────────── files ────────────────────────────────────────────────────────╮");
    tester.assert_tui_frame_contains("Show Preview:");
    tester.assert_not_tui_frame_contains("Hide Preview:");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that --show-preview starts the interface with the preview panel visible.
#[test]
fn test_show_preview_flag_starts_with_preview_visible() {
    let mut tester = PtyTester::new();

    // Start the files channel with the preview panel explicitly visible
    let cmd =
        tv_local_config_and_cable_with_args(&["files", "--show-preview"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the preview panel is visible (shows "Hide Preview:" prompt)
    tester.assert_tui_frame_contains("Hide Preview:");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that --hide-preview conflicts with --no-preview.
#[test]
fn test_hide_preview_conflicts_with_no_preview() {
    let mut tester = PtyTester::new();

    // This should fail because the flags are mutually exclusive
    let cmd = tv_local_config_and_cable_with_args(&[
        "--source-command",
        "ls",
        "--hide-preview",
        "--no-preview",
    ]);
    tester.spawn_command(cmd);

    // CLI should exit with error message
    tester.assert_raw_output_contains("cannot be used with");
}
