//! Tests for CLI directory/config options: [PATH], --config-file, --cable-dir.
//!
//! These tests verify Television's configuration and directory handling capabilities,
//! ensuring that users can customize their setup and work in different directories.

use super::super::common::*;
use std::path::Path;

/// Tests that the PATH positional argument correctly sets the working directory.
#[test]
fn test_path_as_positional_argument_sets_working_directory() {
    let mut tester = PtyTester::new();
    let tmp_dir = Path::new(TARGET_DIR);

    // Create initial files to be detected
    std::fs::write(tmp_dir.join("UNIQUE16CHARIDfile.txt"), "").unwrap();

    // Starts the files channel in the specified temporary directory
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--input",
        "UNIQUE16CHARID",
        tmp_dir.to_str().unwrap(),
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the TUI launched successfully with the files channel
    tester.assert_tui_frame_contains(
        "╭───────────────────────── files ──────────────────────────╮",
    );
    // Verify that the test file is present
    tester.assert_tui_frame_contains("UNIQUE16CHARIDfile.txt");

    // Send Ctrl+C to exit cleanly
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that the --config-file flag loads a custom configuration file.
#[test]
fn test_config_file_flag_loads_custom_config() {
    let mut tester = PtyTester::new();

    // This bypasses the default config locations and uses our test configuration
    let cmd = tv_with_args(&[
        "files",
        "--config-file",
        ".config/config.toml",
        "--cable-dir",
        DEFAULT_CABLE_DIR,
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the TUI started with the custom config
    tester.assert_tui_frame_contains("files");

    // Send Ctrl+C to exit cleanly
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that the --config-file flag fails to load a custom configuration file.
#[test]
fn test_config_file_flag_fails_to_load_custom_config() {
    let mut tester = PtyTester::new();

    // This bypasses the default config locations and uses our test configuration
    let cmd = tv_with_args(&[
        "files",
        "--config-file",
        ".config/config1.toml",
        "--cable-dir",
        DEFAULT_CABLE_DIR,
    ]);
    tester.spawn_command(cmd);

    // CLI should exit with error message, not show TUI
    tester.assert_raw_output_contains("File does not exist");
}

/// Tests that the --cable-dir flag loads channels from a custom directory.
#[test]
fn test_cable_dir_flag_loads_custom_cable_dir() {
    let mut tester = PtyTester::new();

    // This loads channels from our test cable directory instead of the default location
    let cmd = tv_with_args(&[
        "files",
        "--cable-dir",
        "cable/unix",
        "--config-file",
        DEFAULT_CONFIG_FILE,
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the channel was found and loaded successfully
    tester.assert_tui_frame_contains(
        "╭───────────────────────── files ──────────────────────────╮",
    );

    // Send Ctrl+C to exit cleanly
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that the --cable-dir flag fails to load channels from a custom directory.
#[test]
fn test_cable_dir_flag_fails_to_load_custom_cable_dir() {
    let mut tester = PtyTester::new();

    // This bypasses the default config locations and uses our test configuration
    let cmd = tv_with_args(&[
        "files",
        "--cable-dir",
        "cable/unix1",
        "--config-file",
        DEFAULT_CONFIG_FILE,
    ]);
    tester.spawn_command(cmd);

    // CLI should exit with error message, not show TUI
    tester.assert_raw_output_contains("Directory does not exist");
}
