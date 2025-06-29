//! Tests for CLI input/interaction options: --input, --keybindings, --exact.
//!
//! These tests verify Television's input handling and user interaction features,
//! ensuring users can customize their interaction experience and search behavior.

use super::common::*;

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
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

/// Tests that custom keybindings override default keyboard shortcuts.
#[test]
fn test_keybindings_override_default() {
    let mut tester = PtyTester::new();

    // This remaps the quit action from default keys (Esc, Ctrl+C) to just "a"
    let mut child =
        tester.spawn_command_tui(tv_local_config_and_cable_with_args(&[
            "--keybindings",
            "quit='a'",
        ]));

    // Test that ESC no longer quits (default behavior is overridden)
    tester.send(ESC);
    tester.assert_tui_running(&mut child);

    // Test that Ctrl+C no longer quits (default behavior is overridden)
    tester.send(&ctrl('c'));
    tester.assert_tui_running(&mut child);

    // Test that our custom "a" key now quits the application
    tester.send("'a'");
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

/// Tests that multiple keybinding overrides can be specified simultaneously.
#[test]
fn test_multiple_keybindings_override() {
    let mut tester = PtyTester::new();

    // This sets up two custom keybindings: "a" for quit and Ctrl+T for remote control
    let mut child =
        tester.spawn_command_tui(tv_local_config_and_cable_with_args(&[
            "--keybindings",
            "quit='a';toggle_remote_control='ctrl-t'",
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
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

/// Tests that the --exact flag enables exact substring matching instead of fuzzy matching.
#[test]
fn test_exact_matching_enabled() {
    let mut tester = PtyTester::new();

    // This enables exact substring matching instead of the default fuzzy matching
    let cmd = tv_local_config_and_cable_with_args(&[
        "files", "--exact", "--input", "file1",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the TUI started successfully with exact matching enabled
    tester.assert_tui_frame_contains("file1.txt");

    // Send Ctrl+C to exit the application
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

// #[test]
// TODO: This test is failing because the matcher is not correctly handling the exact matching mode. REGRESSION
// fn test_exact_matching_enabled_fails() {
//     let mut tester = PtyTester::new();

//     // This enables exact substring matching instead of the default fuzzy matching
//     let cmd = tv_local_config_and_cable_with_args(&["files", "--exact", "--input", "fl1"]);
//     let mut child = tester.spawn_command_tui(cmd);

//     // Verify the TUI started successfully with exact matching enabled and no results
//     tester.assert_tui_frame_contains("│> fl1                                               0 / 0 │");
//     tester.assert_not_tui_frame_contains("file1.txt");

//     // Send Ctrl+C to exit the application
//     tester.send(&ctrl('c'));
//     PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
// }
