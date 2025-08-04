//! Tests for CLI input/interaction options: --input, --keybindings, --exact.
//!
//! These tests verify Television's input handling and user interaction features,
//! ensuring users can customize their interaction experience and search behavior.

use std::path::Path;

use super::super::common::*;

const TARGET_DIR: &str = "tests/target_dir";

/// Tests that the --input flag pre-fills the search box with specified text.
#[test]
fn test_input_prefills_search_box() {
    let mut tester = PtyTester::new();

    // This should start the files channel with "UNIQUE16CHARID" already typed in the search box
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--input",
        "UNIQUE16CHARID",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the search box contains the pre-filled text
    tester.assert_tui_frame_contains("│> UNIQUE16CHARID");

    // Send Ctrl+C to exit the application gracefully
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that custom keybindings override default keyboard shortcuts.
#[test]
fn test_keybindings_override_default() {
    let mut tester = PtyTester::new();

    // This adds a new mapping for the quit action
    let mut child =
        tester.spawn_command_tui(tv_local_config_and_cable_with_args(&[
            "--keybindings",
            "a=\"quit\";ctrl-c=false;esc=false",
        ]));

    // Test that ESC no longer quits (default behavior is overridden)
    tester.send(ESC);
    tester.assert_tui_running(&mut child);

    // Test that Ctrl+C no longer quits (default behavior is overridden)
    tester.send(&ctrl('c'));
    tester.assert_tui_running(&mut child);

    // Test that our custom "a" key now quits the application
    tester.send("'a'");
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that multiple keybinding overrides can be specified simultaneously.
#[test]
fn test_multiple_keybindings_override() {
    let mut tester = PtyTester::new();

    // This sets up two custom keybindings: "a" for quit and Ctrl+T for remote control
    let mut child =
        tester.spawn_command_tui(tv_local_config_and_cable_with_args(&[
            "--keybindings",
            "a=\"quit\";ctrl-t=\"toggle_remote_control\";esc=false",
        ]));

    // Verify ESC doesn't quit (default overridden)
    tester.send(ESC);
    tester.assert_tui_running(&mut child);

    // Test that Ctrl+T opens remote control panel (custom keybinding works)
    tester.send(&ctrl('t'));
    tester.assert_tui_frame_contains("(1) (2) (3)"); // Remote control indicators

    // Use "a" to quit remote control mode, then "a" again to quit the application
    tester.send("'a'"); // Exit remote control
    tester.send("'a'"); // Exit application
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that the --exact flag enables exact substring matching instead of fuzzy matching.
#[test]
fn test_exact_matching_enabled() {
    let mut tester = PtyTester::new();
    let tmp_dir = Path::new(TARGET_DIR);

    // Create initial file to be detected
    std::fs::write(tmp_dir.join("UNIQUE16CHARIDfile.txt"), "").unwrap();

    // This enables exact substring matching instead of the default fuzzy matching
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--exact",
        "--input",
        "UNIQUE16CHARIDfile",
        tmp_dir.to_str().unwrap(),
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the TUI started successfully with exact matching enabled
    tester.assert_tui_frame_contains("UNIQUE16CHARIDfile.txt");

    // Send Ctrl+C to exit the application
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

#[test]
fn test_exact_matching_enabled_fails() {
    let mut tester = PtyTester::new();
    let tmp_dir = Path::new(TARGET_DIR);

    // Create initial file to be detected
    std::fs::write(tmp_dir.join("UNIQUE16CHARIDfile.txt"), "").unwrap();

    // This enables exact substring matching instead of the default fuzzy matching
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--exact",
        "--input",
        "UNIQUE16CHARIDfl",
        tmp_dir.to_str().unwrap(),
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the TUI started successfully with exact matching enabled and no results
    tester.assert_tui_frame_contains("│> UNIQUE16CHARIDfl");
    tester.assert_tui_frame_contains("0 / 0");
    tester.assert_not_tui_frame_contains("UNIQUE16CHARIDfile.txt");

    // Send Ctrl+C to exit the application
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}
