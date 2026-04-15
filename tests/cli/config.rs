//! Tests for CLI directory/config options: [PATH], --config-file, --cable-dir.
//!
//! These tests verify Television's configuration and directory handling capabilities,
//! ensuring that users can customize their setup and work in different directories.

use tempfile::TempDir;

use super::super::common::*;

/// Tests that the PATH positional argument correctly sets the working directory.
#[test]
fn test_path_as_positional_argument_sets_working_directory() {
    let pt = phantom();
    let tmp_dir = TempDir::new().unwrap();

    // Create initial files to be detected
    std::fs::write(tmp_dir.path().join("UNIQUE16CHARIDfile.txt"), "").unwrap();

    // Starts the files channel in the specified temporary directory
    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "files",
            "--input",
            "UNIQUE16CHARID",
            tmp_dir.path().to_str().unwrap(),
        ],
    )
    .start()
    .unwrap();

    s.wait().text("── files ──").until().unwrap();
    s.wait().text("UNIQUE16CHARIDfile.txt").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that the --config-file flag loads a custom configuration file.
#[test]
fn test_config_file_flag_loads_custom_config() {
    let pt = phantom();

    // This bypasses the default config locations and uses our test configuration
    let s = tv_with_args(
        &pt,
        &[
            "files",
            "--config-file",
            ".config/config.toml",
            "--cable-dir",
            DEFAULT_CABLE_DIR,
        ],
    )
    .start()
    .unwrap();

    s.wait().text("files").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that the --config-file flag fails to load a custom configuration file.
#[test]
fn test_config_file_flag_fails_to_load_custom_config() {
    let pt = phantom();

    let s = tv_with_args(
        &pt,
        &[
            "files",
            "--config-file",
            ".config/config1.toml",
            "--cable-dir",
            DEFAULT_CABLE_DIR,
        ],
    )
    .start()
    .unwrap();

    // CLI should exit with error message, not show TUI
    s.wait().text("File does not exist").until().unwrap();
}

/// Tests that the --cable-dir flag loads channels from a custom directory.
#[test]
fn test_cable_dir_flag_loads_custom_cable_dir() {
    let pt = phantom();

    let s = tv_with_args(
        &pt,
        &[
            "files",
            "--cable-dir",
            "cable/unix",
            "--config-file",
            DEFAULT_CONFIG_FILE,
        ],
    )
    .start()
    .unwrap();

    s.wait().text("── files ──").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that the --cable-dir flag fails to load channels from a custom directory.
#[test]
fn test_cable_dir_flag_fails_to_load_custom_cable_dir() {
    let pt = phantom();

    let s = tv_with_args(
        &pt,
        &[
            "files",
            "--cable-dir",
            "cable/unix1",
            "--config-file",
            DEFAULT_CONFIG_FILE,
        ],
    )
    .start()
    .unwrap();

    // CLI should exit with error message, not show TUI
    s.wait().text("Directory does not exist").until().unwrap();
}
