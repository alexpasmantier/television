//! Tests for CLI selector configuration: --results-max-selections and multi-selection behavior.
//!
//! These tests verify Television's multi-selection functionality including selector modes
//! (`single`, `concatenate`, `one_to_one`), selection limits, shell escaping, and integration
//! with various channel configurations.

use super::super::common::*;

/// Tests disabling multi-selection with zero value.
#[test]
fn test_results_max_selections_zero_value() {
    let mut tester = PtyTester::new();

    let cmd = tv_local_config_and_cable_with_args(&[
        "--source-command",
        "echo -e 'apple.txt\\nbanana.txt\\ncherry.txt'",
        "--preview-command",
        "echo 'Selected: {}!'",
        "--results-max-selections",
        "0",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Wait for initial load
    tester.assert_tui_frame_contains("Custom Channel");
    tester.assert_tui_frame_contains("Selected: apple.txt!");

    // Try to select two items
    tester.send(&ctrl('i'));
    tester.send(&ctrl('i'));

    // Preview should be the same
    tester.assert_tui_frame_contains("Selected: apple.txt!");

    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests basic multi-selection functionality
#[test]
fn test_basic_multi_selection() {
    let mut tester = PtyTester::new();

    let cmd = tv_local_config_and_cable_with_args(&[
        "--source-command",
        "echo -e 'apple.txt\\nbanana.txt\\ncherry.txt'",
        "--preview-command",
        "echo 'Selected: {}!'",
        "--results-max-selections",
        "3",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Wait for initial load
    tester.assert_tui_frame_contains("Custom Channel");
    tester.assert_tui_frame_contains("Selected: apple.txt!");

    // Select first item (apple.txt)
    tester.send(&ctrl('i'));
    // Move select second item
    tester.send(&ctrl('i'));

    // Preview should now show both selected items
    tester.assert_tui_frame_contains("Selected: apple.txt banana.txt!");

    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that selection limit is enforced.
#[test]
fn test_selection_limit_enforcement() {
    let mut tester = PtyTester::new();

    // Set limit to 2 selections
    let cmd = tv_local_config_and_cable_with_args(&[
        "--source-command",
        "echo -e 'file1.txt\\nfile2.txt\\nfile3.txt\\nfile4.txt'",
        "--preview-command",
        "echo 'Count: {}!'",
        "--results-max-selections",
        "2",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    tester.assert_tui_frame_contains("Custom Channel");

    // Select first two items
    tester.send(&ctrl('i')); // Select file1.txt
    tester.send(&ctrl('i')); // Select file2.txt

    // Try to select third item - should be prevented or ignored
    tester.send(&ctrl('i')); // This should be ignored/prevented

    // Preview should still only show 2 items
    tester.assert_tui_frame_contains("Count: file1.txt file2.txt!");

    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests selector mode "single" - only first selected item is used.
#[test]
fn test_selector_mode_single() {
    let mut tester = PtyTester::new();
    let temp_config = TempConfig::init();

    // Create a channel with selector mode set to "single" for preview command
    temp_config
        .write_channel(
            "single-mode-test",
            r#"
                [metadata]
                name = "single-mode"
                description = "Test channel for single selector mode"

                [source]
                command = ["echo -e 'first.txt\nsecond.txt\nthird.txt'"]

                [preview.command]
                template = "echo 'SINGLE: {}!'"
                mode = "single"
                separator = ""
                shell_escaping = false
            "#,
        )
        .unwrap();

    let cmd = tv_with_args(&[
        "single-mode",
        "--config-file",
        temp_config.config_file.to_str().unwrap(),
        "--cable-dir",
        temp_config.cable_dir.to_str().unwrap(),
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    tester.assert_tui_frame_contains("single-mode");
    tester.assert_tui_frame_contains("SINGLE: first.txt!");

    // Select multiple items, but single mode should only use first
    tester.send(&ctrl('i')); // Select first
    tester.send(&ctrl('i')); // Select second

    // In single mode, should still only show first selected item
    tester.assert_tui_frame_contains("SINGLE: first.txt!");

    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests deselecting items by pressing ctrl+i again.
#[test]
fn test_deselecting_items() {
    let mut tester = PtyTester::new();

    let cmd = tv_local_config_and_cable_with_args(&[
        "--source-command",
        "echo -e 'item1.txt\\nitem2.txt\\nitem3.txt'",
        "--preview-command",
        "echo 'ITEMS: {}!'",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    tester.assert_tui_frame_contains("Custom Channel");

    // Select first item
    tester.send(&ctrl('i'));
    tester.assert_tui_frame_contains("ITEMS: item1.txt!");

    // Select second item
    tester.send(&ctrl('i'));
    tester.assert_tui_frame_contains("ITEMS: item1.txt item2.txt!");

    // Deselect first item by going back and pressing ctrl+i again
    tester.send(&ctrl('p')); // Go back to first item
    tester.send(&ctrl('p'));
    tester.send(&ctrl('i')); // Deselect it

    // Should now only show second item
    tester.assert_tui_frame_contains("ITEMS: item2.txt!");

    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests selector mode `one_to_one` - each selected item maps to one template placeholder.
#[test]
fn test_selector_mode_one_to_one() {
    let mut tester = PtyTester::new();
    let temp_config = TempConfig::init();

    // Create a channel with selector mode set to "one_to_one" for preview command
    temp_config
        .write_channel(
            "one-to-one-mode-test",
            r#"
                [metadata]
                name = "one-to-one-mode-test"
                description = "Test channel for one_to_one selector mode"

                [source]
                command = ["echo -e 'arg1.txt\narg2.txt\narg3.txt'"]

                [preview.command]
                template = "echo 'FIRST: {} SECOND: {} THIRD: {}'"
                mode = "one_to_one"
                separator = ""
                shell_escaping = false
            "#,
        )
        .unwrap();

    let cmd = tv_with_args(&[
        "one-to-one-mode-test",
        "--config-file",
        temp_config.config_file.to_str().unwrap(),
        "--cable-dir",
        temp_config.cable_dir.to_str().unwrap(),
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    tester.assert_tui_frame_contains("one-to-one-mode-test");

    // Select three items for one-to-one mapping
    tester.send(&ctrl('i')); // Select arg1.txt
    tester.send(&ctrl('i')); // Select arg2.txt
    tester.send(&ctrl('i')); // Select arg3.txt

    // In one-to-one mode, each argument should map to its placeholder
    tester.assert_tui_frame_contains("FIRST: arg1.txt");
    tester.assert_tui_frame_contains("SECOND: arg2.txt");
    tester.assert_tui_frame_contains("THIRD: arg3.txt");

    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Test that all preview template types (header, footer, command) process multiple selected entries correctly
#[test]
fn test_preview_templates_multi_selection() {
    let mut tester = PtyTester::new();
    let temp_config = TempConfig::init();

    // Create a channel with all three template types configured for one-to-one mode
    temp_config
        .write_channel(
            "comprehensive-template-test",
            r#"
                [metadata]
                name = "comprehensive-template-test"
                description = "Test channel for all template types multi-selection"

                [ui]
                results_max_selections = 3

                [source]
                command = ["echo -e 'item1.txt\nitem2.txt\nitem3.txt'"]

                [ui.preview_panel.header]
                template = "Header: {} and {}"
                mode = "one_to_one"
                separator = ""
                shell_escaping = false

                [ui.preview_panel.footer]
                template = "Footer: {} | {}"
                mode = "one_to_one"
                separator = ""
                shell_escaping = false

                [preview.command]
                template = "echo 'Command: {} then {}'"
                mode = "one_to_one"
                separator = ""
                shell_escaping = false
            "#,
        )
        .unwrap();

    let cmd = tv_with_args(&[
        "comprehensive-template-test",
        "--config-file",
        temp_config.config_file.to_str().unwrap(),
        "--cable-dir",
        temp_config.cable_dir.to_str().unwrap(),
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    tester.assert_tui_frame_contains("comprehensive-template-test");

    // Select 2 items using ctrl+i
    tester.send(&ctrl('i')); // Select item1.txt and move down
    tester.send(&ctrl('i')); // Select item2.txt and move down

    // All template types should show both selected files in one-to-one mode
    tester.assert_tui_frame_contains("Header: item1.txt and item2.txt");
    tester.assert_tui_frame_contains("Footer: item1.txt | item2.txt");
    tester.assert_tui_frame_contains("Command: item1.txt then item2.txt");

    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}
