//! Tests for CLI frecency behavior: --frecency and --global-frecency flags.
//!
//! These tests verify Television's frecency scoring system that boosts previously
//! selected entries in future searches based on frequency and recency of access.
//! They ensure that selected items appear higher in results lists and that
//! frecency data persists across sessions.

use super::common::*;
use std::fs;
use tempfile::TempDir;

/// Helper function to create a temporary data directory for frecency storage
fn create_temp_data_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}

/// Helper function to create a temporary data directory with empty frecency.json
fn create_temp_data_dir_with_empty_frecency() -> TempDir {
    let temp_dir = create_temp_data_dir();
    let frecency_file = temp_dir.path().join("frecency.json");

    // Create empty frecency file to ensure consistent starting state
    fs::write(&frecency_file, "[]")
        .expect("Failed to create empty frecency file");

    temp_dir
}

/// Helper function to create tv command with frecency enabled and custom data directory
fn tv_frecency_with_data_dir(
    data_dir: &str,
    args: &[&str],
) -> portable_pty::CommandBuilder {
    let mut cmd = tv_local_config_and_cable_with_args(args);
    cmd.env("TELEVISION_DATA", data_dir);
    cmd.arg("--frecency");
    cmd
}

/// Tests that frecency ranking works: previously selected entries appear higher in results.
#[test]
fn test_frecency_ranking_boosts_selected_entries() {
    let temp_dir = create_temp_data_dir_with_empty_frecency();
    let data_dir = temp_dir.path().to_str().unwrap();

    // Create test data with multiple entries
    let test_entries = ["apple.txt", "banana.txt", "cherry.txt", "date.txt"];

    // First session: select "cherry.txt" (third item)
    {
        let mut tester = PtyTester::new();
        let cmd = tv_frecency_with_data_dir(
            data_dir,
            &[
                "--source-command",
                &format!("printf '{}'", test_entries.join("\n")),
            ],
        );

        let mut child = tester.spawn_command_tui(cmd);

        // Wait for UI to load and verify initial order
        tester.assert_tui_frame_contains("> apple.txt");
        tester.send(&ctrl('n')); // Move down to banana.txt
        tester.assert_tui_frame_contains("> banana.txt");
        tester.send(&ctrl('n')); // Move down to cherry.txt
        tester.assert_tui_frame_contains("> cherry.txt");

        // Select cherry.txt
        tester.send(ENTER);

        // Wait for selection to complete and exit
        PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
    }

    // Second session: verify cherry.txt appears higher due to frecency
    {
        let mut tester = PtyTester::new();
        let cmd = tv_frecency_with_data_dir(
            data_dir,
            &[
                "--source-command",
                &format!("printf '{}'", test_entries.join("\n")),
            ],
        );

        let mut child = tester.spawn_command_tui(cmd);

        // Wait for UI to load and verify initial order
        tester.assert_tui_frame_contains("> cherry.txt");
        tester.send(&ctrl('n')); // Move down to apple.txt
        tester.assert_tui_frame_contains("> apple.txt");
        tester.send(&ctrl('n')); // Move down to banana.txt
        tester.assert_tui_frame_contains("> banana.txt");

        // Exit cleanly
        tester.send(&ctrl('c'));
        PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
    }
}

/// Tests that frecency data persists across multiple sessions.
#[test]
fn test_frecency_persistence_across_sessions() {
    let temp_dir = create_temp_data_dir_with_empty_frecency();
    let data_dir = temp_dir.path().to_str().unwrap();
    let frecency_file = format!("{}/frecency.json", data_dir);

    // Create test data with multiple entries
    let test_entries = ["persistent.txt", "other.txt"];

    // First session: select an entry to create frecency data
    {
        let mut tester = PtyTester::new();
        let cmd = tv_frecency_with_data_dir(
            data_dir,
            &[
                "--source-command",
                &format!("printf '{}'", test_entries.join("\n")),
            ],
        );

        let mut child = tester.spawn_command_tui(cmd);

        // Wait for UI to load and verify initial order
        tester.assert_tui_frame_contains("> persistent.txt");

        // Select the first entry (persistent.txt)
        tester.send(ENTER);

        // Wait for selection to complete and exit
        PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
    }

    // Verify frecency file was created
    assert!(
        std::path::Path::new(&frecency_file).exists(),
        "Frecency file should be created at: {}",
        frecency_file
    );

    // Read and verify frecency file contains expected data
    let frecency_content = fs::read_to_string(&frecency_file)
        .expect("Should be able to read frecency file");

    assert!(
        frecency_content.contains("persistent.txt"),
        "Frecency file should contain selected entry. Content:\n{}",
        frecency_content
    );

    // Second session: verify frecency data is loaded and applied
    {
        let mut tester = PtyTester::new();
        let cmd = tv_frecency_with_data_dir(
            data_dir,
            &[
                "--source-command",
                "printf 'other.txt\npersistent.txt'", // Note: reversed order
            ],
        );

        let mut child = tester.spawn_command_tui(cmd);

        // Wait for UI to load and verify persistent.txt appears first due to frecency
        tester.assert_tui_frame_contains("> persistent.txt");
        tester.send(&ctrl('n')); // Move down to other.txt
        tester.assert_tui_frame_contains("> other.txt");

        // Exit cleanly
        tester.send(&ctrl('c'));
        PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
    }
}

/// Tests that frecency only shows entries that exist in the current dataset.
#[test]
fn test_frecency_dataset_filtering() {
    let temp_dir = create_temp_data_dir_with_empty_frecency();
    let data_dir = temp_dir.path().to_str().unwrap();

    // Create test data with multiple entries
    let first_session_entries = ["common.txt", "exclusive.txt", "shared.txt"];
    let second_session_entries = ["common.txt", "shared.txt", "new.txt"];

    // First session: select entries from a larger dataset
    {
        let mut tester = PtyTester::new();
        let cmd = tv_frecency_with_data_dir(
            data_dir,
            &[
                "--source-command",
                &format!("printf '{}'", first_session_entries.join("\n")),
            ],
        );

        let mut child = tester.spawn_command_tui(cmd);

        // Wait for UI to load and verify initial order
        tester.assert_tui_frame_contains("> common.txt");
        tester.send(&ctrl('n')); // Move down to exclusive.txt
        tester.assert_tui_frame_contains("> exclusive.txt");

        // Select exclusive.txt (second item)
        tester.send(ENTER);

        // Wait for selection to complete and exit
        PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
    }

    // Second session: use a dataset that doesn't contain exclusive.txt
    {
        let mut tester = PtyTester::new();
        let cmd = tv_frecency_with_data_dir(
            data_dir,
            &[
                "--source-command",
                &format!("printf '{}'", second_session_entries.join("\n")), // exclusive.txt not present
            ],
        );

        let mut child = tester.spawn_command_tui(cmd);

        // Wait for UI to load and verify only current dataset entries are present
        tester.assert_tui_frame_contains("> common.txt");
        tester.send(&ctrl('n')); // Move down to shared.txt
        tester.assert_tui_frame_contains("> shared.txt");
        tester.send(&ctrl('n')); // Move down to new.txt
        tester.assert_tui_frame_contains("> new.txt");

        let frame = tester.get_tui_frame();

        // exclusive.txt should not appear in results since it's not in current dataset
        assert!(
            !frame.contains("exclusive.txt"),
            "exclusive.txt should not appear when not in current dataset. Frame:\n{}",
            frame
        );

        // Exit cleanly
        tester.send(&ctrl('c'));
        PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
    }
}

/// Tests global vs channel-specific frecency behavior.
#[test]
fn test_global_vs_channel_frecency() {
    let temp_dir = create_temp_data_dir_with_empty_frecency();
    let data_dir = temp_dir.path().to_str().unwrap();

    // Create test data with multiple entries
    let files_entries = ["file1.txt", "file2.txt"];
    let env_entries = ["file2.txt", "env_var"];

    // First session: select entry in "files" channel
    {
        let mut tester = PtyTester::new();
        let cmd = tv_frecency_with_data_dir(
            data_dir,
            &[
                "files",
                "--source-command",
                &format!("printf '{}'", files_entries.join("\n")),
            ],
        );

        let mut child = tester.spawn_command_tui(cmd);

        // Wait for UI to load and verify initial order
        tester.assert_tui_frame_contains("> file1.txt");
        tester.send(&ctrl('n')); // Move down to file2.txt
        tester.assert_tui_frame_contains("> file2.txt");

        // Select file2.txt
        tester.send(ENTER);

        // Wait for selection to complete and exit
        PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
    }

    // Second session: test channel-specific frecency (default behavior)
    {
        let mut tester = PtyTester::new();
        let cmd = tv_frecency_with_data_dir(
            data_dir,
            &[
                "env", // Different channel
                "--source-command",
                &format!("printf '{}'", env_entries.join("\n")),
            ],
        );

        let mut child = tester.spawn_command_tui(cmd);

        // Wait for UI to load and verify natural order (env_var first)
        tester.assert_tui_frame_contains("> file2.txt"); // Should be first in natural order
        tester.send(&ctrl('n')); // Move down to env_var
        tester.assert_tui_frame_contains("> env_var");

        let frame = tester.get_tui_frame();

        // In channel-specific mode, file2.txt should NOT be boosted since it was selected in different channel
        let file2_pos = frame
            .find("file2.txt")
            .expect("file2.txt should be present");
        let env_pos =
            frame.find("env_var").expect("env_var should be present");

        // file2.txt should appear first (natural order) since frecency doesn't apply across channels
        assert!(
            file2_pos < env_pos,
            "file2.txt should appear first in channel-specific mode. Frame:\n{}",
            frame
        );

        // Exit cleanly
        tester.send(&ctrl('c'));
        PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
    }

    // Third session: test global frecency
    {
        let mut tester = PtyTester::new();
        let cmd = tv_frecency_with_data_dir(
            data_dir,
            &[
                "env",               // Different channel
                "--global-frecency", // Enable global frecency
                "--source-command",
                &format!("printf '{}'", env_entries.join("\n")),
            ],
        );

        let mut child = tester.spawn_command_tui(cmd);

        // Wait for UI to load and verify file2.txt appears first due to global frecency
        tester.assert_tui_frame_contains("> file2.txt");
        tester.send(&ctrl('n')); // Move down to env_var
        tester.assert_tui_frame_contains("> env_var");

        // Exit cleanly
        tester.send(&ctrl('c'));
        PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
    }
}
