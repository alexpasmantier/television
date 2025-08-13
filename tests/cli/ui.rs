//! Tests for CLI UI/layout options: --layout, --input-header, --ui-scale, --no-remote, --no-status-bar.
//!
//! These tests verify Television's user interface customization capabilities,
//! ensuring users can adapt the layout and appearance to their preferences and needs.

use television::tui::TESTING_ENV_VAR;

use super::super::common::*;

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
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
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
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that --layout portrait arranges panels vertically stacked.
// FIXME: this should be in a separate module that tests TUI interactions
#[test]
fn test_toggle_layout() {
    let mut tester = PtyTester::new();

    // This sets the interface to use vertically stacked panel arrangement
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--layout",
        "portrait",
        "--keybindings",
        "ctrl-l='toggle_layout'",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the TUI started successfully in portrait layout
    tester.assert_tui_frame_contains("╭─────────────────────────────────────────────────────── files ────────────────────────────────────────────────────────╮");

    // Toggle to landscape layout
    tester.send(&ctrl('l'));

    // Verify the TUI switched to landscape layout
    tester.assert_tui_frame_contains(
        "╭───────────────────────── files ──────────────────────────╮╭─",
    );

    // Toggle back to portrait layout
    tester.send(&ctrl('l'));

    // Verify the TUI switched back to portrait layout
    tester.assert_tui_frame_contains("╭─────────────────────────────────────────────────────── files ────────────────────────────────────────────────────────╮");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
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
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
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
    tester.assert_tui_frame_contains("CHANNEL  Custom");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that --input-prompt customizes the prompt symbol in Channel Mode.
#[test]
fn test_input_prompt_in_channel_mode() {
    let mut tester = PtyTester::new();

    // This overrides the channel's default input prompt with custom symbol
    let mut cmd = tv_local_config_and_cable_with_args(&["files"]);
    cmd.args(["--input-prompt", "❯ "]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the custom input prompt is displayed
    tester.assert_tui_frame_contains("❯ ");
    tester.assert_tui_frame_contains("CHANNEL  files");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that --input-prompt works in Ad-hoc Mode.
#[test]
fn test_input_prompt_in_adhoc_mode() {
    let mut tester = PtyTester::new();

    // This provides a custom input prompt for an ad-hoc channel
    let mut cmd =
        tv_local_config_and_cable_with_args(&["--source-command", "ls"]);
    cmd.args(["--input-prompt", "→ "]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the custom input prompt is displayed
    tester.assert_tui_frame_contains("→ ");
    tester.assert_tui_frame_contains("CHANNEL  Custom");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that the default input prompt "> " is used when no custom prompt is specified.
#[test]
fn test_default_input_prompt() {
    let mut tester = PtyTester::new();

    // This uses the default input prompt
    let cmd = tv_local_config_and_cable_with_args(&["files"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the default input prompt is displayed
    tester.assert_tui_frame_contains("> ");
    tester.assert_tui_frame_contains("CHANNEL  files");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
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
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
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
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
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
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
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
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
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
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
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
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
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
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
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
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
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

#[test]
fn test_tui_with_height_and_width() {
    unsafe { std::env::set_var(TESTING_ENV_VAR, "1") };
    let mut tester = PtyTester::new();

    // Test TUI with height 20 and width 80
    let mut child =
        tester.spawn_command_tui(tv_local_config_and_cable_with_args(&[
            "files", "--height", "20", "--width", "80",
        ]));

    // Check that 'files' appears in the frame content
    tester.assert_tui_frame_contains("CHANNEL  files");

    // Validate frame dimensions (20 rows × 80 columns)
    let frame = tester.get_tui_frame();
    let non_empty_lines: Vec<&str> =
        frame.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(
        non_empty_lines.len(),
        20,
        "Expected 20 rows, got {}",
        non_empty_lines.len()
    );
    let max_width = non_empty_lines
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0);
    assert_eq!(max_width, 80, "Expected 80 columns, got {}", max_width);

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
    unsafe { std::env::remove_var(TESTING_ENV_VAR) };
}

/// Tests that --no-preview disables the preview panel entirely.
#[test]
fn test_no_preview_disables_preview_panel() {
    let mut tester = PtyTester::new();

    // This disables the preview panel entirely
    let cmd = tv_local_config_and_cable_with_args(&["files", "--no-preview"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Try to toggle preview - it shouldn't work since it's disabled entirely
    tester.send("o"); // Toggle preview key

    // Verify no preview elements are shown (no scrollbar, no panel frame)
    tester.assert_tui_frame_contains_none(&["───╮╭───", "Show Preview"]);

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that --show-preview starts the interface with the preview panel visible.
#[test]
fn test_show_preview_starts_with_preview_visible() {
    let mut tester = PtyTester::new();

    // Start with the files channel and force the preview panel visible
    let cmd =
        tv_local_config_and_cable_with_args(&["files", "--show-preview"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify preview panel is initially visible (landscape layout shows side-by-side panels)
    tester.assert_tui_frame_contains_all(&["───╮╭───", "Hide Preview"]);

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that --no-status-bar disables the status bar entirely.
#[test]
fn test_no_status_bar_disables_status_bar() {
    let mut tester = PtyTester::new();

    // This disables the status bar entirely
    let cmd =
        tv_local_config_and_cable_with_args(&["files", "--no-status-bar"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the status bar is not shown
    tester.assert_not_tui_frame_contains("CHANNEL  files");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that --show-status-bar starts the interface with the status bar visible.
#[test]
fn test_show_status_bar_starts_with_status_visible() {
    let mut tester = PtyTester::new();

    // Start with the files channel and force the status bar visible
    let cmd =
        tv_local_config_and_cable_with_args(&["files", "--show-status-bar"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify status bar is initially visible
    tester.assert_tui_frame_contains("CHANNEL  files");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that --hide-preview-scrollbar hides the preview panel scrollbar.
#[test]
fn test_hide_preview_scrollbar_hides_scrollbar() {
    let mut tester = PtyTester::new();

    // This hides the preview scrollbar while keeping the preview panel functional
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--hide-preview-scrollbar",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // The preview panel should still be visible but without scrollbar indicators
    tester.assert_tui_frame_contains_all(&["Hide Preview", "───╮╭───"]);
    tester.assert_not_tui_frame_contains("▲");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that --no-preview conflicts with preview-related flags.
#[test]
fn test_no_preview_conflicts_with_preview_flags() {
    let mut tester = PtyTester::new();

    // This should fail because --no-preview conflicts with --preview-command
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

/// Tests that --no-status-bar conflicts with status-bar-related flags.
#[test]
fn test_no_status_bar_conflicts_with_status_bar_flags() {
    let mut tester = PtyTester::new();

    // This should fail because --no-status-bar conflicts with --show-status-bar
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--no-status-bar",
        "--show-status-bar",
    ]);
    tester.spawn_command(cmd);

    // CLI should exit with error message, not show TUI
    tester.assert_raw_output_contains("cannot be used with");
}

#[test]
// FIXME: needs https://github.com/crossterm-rs/crossterm/pull/957
#[ignore = "needs https://github.com/crossterm-rs/crossterm/pull/957"]
fn test_tui_with_height_only() {
    unsafe { std::env::set_var(TESTING_ENV_VAR, "1") };
    let mut tester = PtyTester::new();

    // Test TUI with only height specified
    let mut child =
        tester.spawn_command_tui(tv_local_config_and_cable_with_args(&[
            "files", "--height", "15",
        ]));

    // Check that 'files' appears in the frame content
    tester.assert_tui_frame_contains("CHANNEL  files");

    // Validate frame height (15 rows)
    let frame = tester.get_tui_frame();
    let non_empty_lines: Vec<&str> =
        frame.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(
        non_empty_lines.len(),
        15,
        "Expected 15 rows, got {}",
        non_empty_lines.len()
    );

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
    unsafe { std::env::remove_var(TESTING_ENV_VAR) };
}
