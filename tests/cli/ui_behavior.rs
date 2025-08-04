//! Tests for CLI UI/behavioral integration: toggling panels, scrolling, clipboard, reload, etc.
//!
//! These tests verify Television's interactive UI behaviors and keyboard shortcuts,
//! ensuring users can effectively navigate and control the interface during operation.
//! These are integration tests that combine CLI setup with interactive behavior.

use super::super::common::*;
use std::path::Path;

/// Tests that the toggle preview keybinding functionality works correctly.
#[test]
fn test_toggle_preview_keybinding() {
    let mut tester = PtyTester::new();

    // Start with the files channel which has preview enabled by default
    let cmd = tv_local_config_and_cable_with_args(&["files"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify preview is initially visible (shows "Hide Preview:" option)
    tester.assert_tui_frame_contains("Hide Preview:");

    // Send Ctrl+O to toggle preview off
    tester.send(&ctrl('o'));

    // Verify preview is now hidden (shows "Show Preview:" option)
    tester.assert_tui_frame_contains("Show Preview:");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that the toggle remote control keybinding functionality works correctly.
#[test]
fn test_toggle_remote_control_keybinding() {
    let mut tester = PtyTester::new();

    // Start with the files channel
    let cmd = tv_local_config_and_cable_with_args(&["files"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Send Ctrl+T to open remote control panel
    tester.send(&ctrl('t'));

    // Verify remote control panel is displayed with channel indicators
    tester.assert_tui_frame_contains("(1) (2) (3)");

    // Send Ctrl+C to exit remote control mode
    tester.send(&ctrl('c'));

    // Send Ctrl+C again to exit the application
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that the toggle status bar keybinding functionality works correctly.
#[test]
fn test_toggle_status_bar_keybinding() {
    let mut tester = PtyTester::new();

    // Start with the files channel which shows status bar by default
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--keybindings",
        "ctrl-k = \"toggle_status_bar\"",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Send Ctrl+K to toggle status bar off
    tester.send(&ctrl('k'));

    // Verify status bar is hidden
    tester.assert_not_tui_frame_contains("CHANNEL  files");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that the toggle help keybinding functionality works correctly.
#[test]
fn test_toggle_help_keybinding() {
    let mut tester = PtyTester::new();

    // Start with the files channel
    let cmd = tv_local_config_and_cable_with_args(&["files"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Send Ctrl+H to open help panel
    tester.send(&ctrl('h'));

    // Verify help panel is displayed
    tester.assert_tui_frame_contains("───── Help ─────");

    // Send Ctrl+C to exit (help panel should close and app should exit)
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that the preview scrolling keybindings functionality works correctly.
#[test]
fn test_scroll_preview_keybindings() {
    let mut tester = PtyTester::new();

    // Start with the files channel which has preview enabled
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--input",
        "README.md",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Send Page Down to scroll preview down
    tester.send("\x1b[6~");
    tester.send("\x1b[6~");

    // Verify preview panel has moved
    tester.assert_not_tui_frame_contains("││   1");

    // Send Page Up to scroll preview up
    tester.send("\x1b[5~");
    tester.send("\x1b[5~");

    // Verify preview panel has moved
    tester.assert_tui_frame_contains("││   1");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that the reload source keybinding functionality works correctly.
#[test]
fn test_reload_source_keybinding() {
    let mut tester = PtyTester::new();
    let tmp_dir = Path::new(TARGET_DIR);

    // Create initial file to be detected
    std::fs::write(tmp_dir.join("UNIQUE16CHARIDfile.txt"), "").unwrap();

    // Start with the files channel
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--input",
        "UNIQUE16CHARID",
        tmp_dir.to_str().unwrap(),
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the initial file appears in the TUI
    tester.assert_tui_frame_contains("UNIQUE16CHARIDfile.txt");

    // add another file to be detected
    std::fs::write(tmp_dir.join("UNIQUE16CHARIDcontrol.txt"), "").unwrap();

    // Send Ctrl+R to reload the source command
    tester.send(&ctrl('r'));

    // Verify the new file appears in the TUI as well as the existing one
    tester.assert_tui_frame_contains("UNIQUE16CHARIDcontrol.txt");
    tester.assert_tui_frame_contains("UNIQUE16CHARIDfile.txt");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that the cycle sources keybinding functionality works correctly.
#[test]
fn test_cycle_sources_keybinding() {
    let mut tester = PtyTester::new();

    // Start with the files channel
    let cmd = tv_local_config_and_cable_with_args(&["files"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Send Ctrl+S to cycle to next source
    tester.send(&ctrl('s'));
    tester.send(".config");

    // Verify a different source is active (shows config file from different source)
    tester.assert_tui_frame_contains(".config/config.toml");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that preview toggle is disabled when in remote control mode.
#[test]
fn test_toggle_preview_disabled_in_remote_control_mode() {
    let mut tester = PtyTester::new();

    // Start with the files channel which has preview enabled by default
    let cmd = tv_local_config_and_cable_with_args(&["files"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify preview is initially visible
    tester.assert_tui_frame_contains(
        "╭───────────────────────── files ──────────────────────────╮╭─",
    );
    tester.assert_tui_frame_contains("Hide Preview:");

    // Enter remote control mode
    tester.send(&ctrl('t'));

    // Verify we're in remote control mode (shows channel indicators and "Back to Channel")
    tester.assert_tui_frame_contains("(1) (2) (3)");
    tester.assert_tui_frame_contains("Back to Channel:");

    // Verify the preview hint is correctly hidden in remote control mode
    tester.assert_not_tui_frame_contains("Hide Preview:");
    tester.assert_not_tui_frame_contains("Show Preview:");

    // Try to toggle preview - this should NOT work in remote control mode
    tester.send(&ctrl('o'));

    // Verify we're still in remote control mode and preview is still visible
    // (the toggle should have been ignored)
    tester.assert_tui_frame_contains("(1) (2) (3)");
    tester.assert_tui_frame_contains("Back to Channel:");
    tester.assert_tui_frame_contains(
        "╭───────────────────────── files ──────────────────────────╮╭─",
    );

    // Exit remote control mode
    tester.send(&ctrl('t'));

    // Verify we're back in channel mode and preview hint is shown again
    tester.assert_tui_frame_contains("Hide Preview:");
    tester.assert_not_tui_frame_contains("Back to Channel:");

    // Verify preview toggle works again in channel mode
    tester.send(&ctrl('o'));
    tester.assert_tui_frame_contains("Show Preview:");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}
