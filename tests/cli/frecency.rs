//! Tests for CLI frecency behavior: --frecency and --global-frecency flags.
//!
//! These tests verify Television's frecency scoring system that boosts previously
//! selected entries in future searches based on frequency and recency of access.
//! They ensure that selected items appear higher in results lists and that
//! frecency data persists across sessions.

use super::super::common::*;
use std::fs;

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
    let temp_data = TempDataDir::init_with_empty_frecency();
    let data_dir = temp_data.path();

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
    let temp_data = TempDataDir::init_with_empty_frecency();
    let data_dir = temp_data.path();
    let frecency_file = &temp_data.frecency_file;

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
        frecency_file.exists(),
        "Frecency file should be created at: {:?}",
        frecency_file
    );

    // Read and verify frecency file contains expected data
    let frecency_content = temp_data
        .read_frecency()
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
    let temp_data = TempDataDir::init_with_empty_frecency();
    let data_dir = temp_data.path();

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
    let temp_data = TempDataDir::init_with_empty_frecency();
    let data_dir = temp_data.path();

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

/// Tests that frecency handles duplicate entries correctly by incrementing access count.
#[test]
fn test_frecency_duplicate_entry_handling() {
    let temp_data = TempDataDir::init_with_empty_frecency();
    let data_dir = temp_data.path();

    // Create test data with multiple entries
    let test_entries = ["duplicate.txt", "other1.txt", "other2.txt"];

    // First session: select "duplicate.txt" once
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
        tester.assert_tui_frame_contains("> duplicate.txt");

        // Select duplicate.txt (first time)
        tester.send(ENTER);

        // Wait for selection to complete and exit
        PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
    }

    // Second session: select "duplicate.txt" again to test increment
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

        // Wait for UI to load and verify duplicate.txt is boosted
        tester.assert_tui_frame_contains("> duplicate.txt");

        // Select duplicate.txt (second time)
        tester.send(ENTER);

        // Wait for selection to complete and exit
        PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
    }

    // Verify frecency file shows duplicate.txt has access_count of 2
    let frecency_content = temp_data
        .read_frecency()
        .expect("Should be able to read frecency file");

    // Parse JSON to verify access count
    let frecency_entries: serde_json::Value =
        serde_json::from_str(&frecency_content)
            .expect("Frecency file should contain valid JSON");

    let duplicate_entry = frecency_entries
        .as_array()
        .expect("Frecency data should be an array")
        .iter()
        .find(|entry| entry["entry"].as_str() == Some("duplicate.txt"))
        .expect("duplicate.txt should be in frecency data");

    assert_eq!(
        duplicate_entry["access_count"].as_u64(),
        Some(2),
        "duplicate.txt should have access_count of 2. Entry: {}",
        duplicate_entry
    );
}

/// Tests frecency behavior when the JSON file is corrupted or invalid.
#[test]
fn test_frecency_corrupted_file_handling() {
    let temp_data = TempDataDir::init();
    let data_dir = temp_data.path();

    // Create corrupted frecency file
    temp_data
        .write_frecency("{ invalid json content }")
        .expect("Failed to create corrupted frecency file");

    // Create test data
    let test_entries = ["recovery_test.txt", "normal_entry.txt"];

    // First session: should handle corrupted file gracefully
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

        // Wait for UI to load - should work despite corrupted file
        tester.assert_tui_frame_contains("> recovery_test.txt");
        tester.send(&ctrl('n')); // Move down to normal_entry.txt
        tester.assert_tui_frame_contains("> normal_entry.txt");

        // Select normal_entry.txt
        tester.send(ENTER);

        // Wait for selection to complete and exit
        PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
    }

    // Verify frecency file was recreated with valid JSON
    let frecency_content = temp_data
        .read_frecency()
        .expect("Should be able to read frecency file after recovery");

    assert!(
        frecency_content.starts_with('[') && frecency_content.ends_with(']'),
        "Frecency file should be valid JSON array after recovery. Content:\n{}",
        frecency_content
    );

    assert!(
        frecency_content.contains("normal_entry.txt"),
        "Frecency file should contain new entry after recovery. Content:\n{}",
        frecency_content
    );

    // Second session: verify frecency is working normally after recovery
    {
        let mut tester = PtyTester::new();
        let cmd = tv_frecency_with_data_dir(
            data_dir,
            &[
                "--source-command",
                "printf 'recovery_test.txt\nnormal_entry.txt'", // Reversed order
            ],
        );

        let mut child = tester.spawn_command_tui(cmd);

        // Wait for UI to load and verify normal_entry.txt appears first due to frecency
        tester.assert_tui_frame_contains("> normal_entry.txt");
        tester.send(&ctrl('n')); // Move down to recovery_test.txt
        tester.assert_tui_frame_contains("> recovery_test.txt");

        // Exit cleanly
        tester.send(&ctrl('c'));
        PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
    }
}

/// Tests frecency without the --frecency flag to ensure it's properly disabled.
#[test]
fn test_frecency_disabled_behavior() {
    let temp_data = TempDataDir::init_with_empty_frecency();
    let data_dir = temp_data.path();

    // Create test data
    let test_entries = ["first.txt", "second.txt", "third.txt"];

    // First session: select third.txt WITHOUT --frecency flag
    {
        let mut tester = PtyTester::new();
        let mut cmd = tv_local_config_and_cable_with_args(&[
            "--source-command",
            &format!("printf '{}'", test_entries.join("\n")),
        ]);
        cmd.env("TELEVISION_DATA", data_dir);
        // Note: NO --frecency flag

        let mut child = tester.spawn_command_tui(cmd);

        // Wait for UI to load and verify initial order
        tester.assert_tui_frame_contains("> first.txt");
        tester.send(&ctrl('n')); // Move down to second.txt
        tester.assert_tui_frame_contains("> second.txt");
        tester.send(&ctrl('n')); // Move down to third.txt
        tester.assert_tui_frame_contains("> third.txt");

        // Select third.txt
        tester.send(ENTER);

        // Wait for selection to complete and exit
        PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
    }

    // Verify selected entry was not added to frecency data
    if temp_data.frecency_file.exists() {
        let frecency_content = temp_data
            .read_frecency()
            .unwrap_or_else(|_| "[]".to_string());
        assert!(
            !frecency_content.contains("third.txt"),
            "Frecency file should not contain selected entry when --frecency flag is not used. Content:\n{}",
            frecency_content
        );
    }

    // Second session: verify order remains unchanged (no frecency boost)
    {
        let mut tester = PtyTester::new();
        let mut cmd = tv_local_config_and_cable_with_args(&[
            "--source-command",
            &format!("printf '{}'", test_entries.join("\n")),
        ]);
        cmd.env("TELEVISION_DATA", data_dir);
        // Note: Still NO --frecency flag

        let mut child = tester.spawn_command_tui(cmd);

        // Wait for UI to load and verify original order maintained
        tester.assert_tui_frame_contains("> first.txt");
        tester.send(&ctrl('n')); // Move down to second.txt
        tester.assert_tui_frame_contains("> second.txt");
        tester.send(&ctrl('n')); // Move down to third.txt
        tester.assert_tui_frame_contains("> third.txt");

        // Exit cleanly
        tester.send(&ctrl('c'));
        PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
    }
}

/// Tests that preview works correctly with frecency items at high offsets.
#[test]
fn test_frecency_preview_with_high_offset_navigation() {
    let temp_data = TempDataDir::init_with_empty_frecency();
    let data_dir = temp_data.path();

    // Create a test file that we can preview
    let test_file_content = "This is preview content\nLine 2\nLine 3";
    let test_file = temp_data.tempdir().path().join("preview_test.txt");
    fs::write(&test_file, test_file_content)
        .expect("Failed to create test file");

    let mut tester = PtyTester::new();
    let cmd =
        tv_frecency_with_data_dir(data_dir, &["files", "--global-frecency"]);

    let mut child = tester.spawn_command_tui(cmd);

    // Wait for UI to load with preview panel
    tester.assert_tui_frame_contains_all(&["files", "Preview"]);

    // Move to the last position (high offset) by pressing Ctrl+P (up) once
    // This should select the last item in the list
    tester.send(&ctrl('p'));

    tester.assert_not_tui_frame_contains("Select an entry to preview");

    // Exit cleanly
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}
