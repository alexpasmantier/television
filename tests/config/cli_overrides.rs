use std::thread::sleep;

use super::super::common::*;
use std::path::Path;

/// Tests CLI override of UI configuration from config file
#[test]
fn test_cli_ui_overrides_config_file() {
    let mut tester = PtyTester::new();
    let temp_config = TempConfig::init();

    let config_content = r#"
        [ui]
        ui_scale = 80

        [ui.input_bar]
        prompt = "config-prompt>"
        border_type = "plain"

        [ui.preview_panel]
        size = 40
        border_type = "thick"
    "#;

    temp_config.write_config(config_content).unwrap();

    // CLI should override all these UI settings
    let cmd = tv_with_args(&[
        "files",
        "--config-file",
        temp_config.config_file.to_str().unwrap(),
        "--cable-dir",
        DEFAULT_CABLE_DIR,
        "--ui-scale",
        "85",
        "--input-prompt",
        "cli-prompt>",
        "--preview-size",
        "70",
        "--input-border",
        "rounded",
        "--preview-border",
        "thick",
    ]);

    let mut child = tester.spawn_command_tui(cmd);

    // Verify the application starts with CLI overrides
    tester.assert_tui_frame_contains_all(&[
        "files",
        "cli-prompt>",
        // rounded border (input)
        "╭",
        // thick border (preview)
        "┏",
    ]);

    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests CLI overrides of channel-specific configuration
#[test]
fn test_cli_overrides_channel_source_and_preview() {
    let mut tester = PtyTester::new();
    let temp_config = TempConfig::init();

    let channel_content = r#"
        [metadata]
        name = "override-test"
        description = "Channel for override testing"

        [source]
        command = "echo 'channel-item-1'; echo 'channel-item-2'"

        [preview]
        command = "echo 'Channel preview: {}'"
    "#;
    temp_config
        .write_channel("override-test", channel_content)
        .unwrap();

    // CLI overrides both source and preview
    let cmd = tv_with_args(&[
        "override-test",
        "--cable-dir",
        temp_config.cable_dir.to_str().unwrap(),
        "--source-command",
        "echo 'cli-item-1'; echo 'cli-item-2'; echo 'cli-item-3'",
        "--preview-command",
        "echo 'CLI preview: {}'",
        "--preview-size",
        "60",
    ]);

    let mut child = tester.spawn_command_tui(cmd);

    // Should see CLI source command output instead of channel's
    tester.assert_tui_frame_contains_all(&[
        "override-test",
        "cli-item-1",
        "cli-item-2",
        "cli-item-3",
    ]);

    // Channel items should not appear
    tester.assert_not_tui_frame_contains("channel-item-1");

    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests CLI working directory parameter
#[test]
fn test_cli_working_directory_override() {
    let mut tester = PtyTester::new();
    let target_dir = Path::new(TARGET_DIR);

    // Create a test file in the target directory
    std::fs::write(target_dir.join("working-dir-test.txt"), "test content")
        .unwrap();

    let cmd = tv_with_args(&[
        "files",
        "--cable-dir",
        DEFAULT_CABLE_DIR,
        "--input",
        "working-dir-test",
        "--take-1-fast",
        target_dir.to_str().unwrap(), // PATH as positional argument
    ]);

    // Should exit with the found file
    let mut child = tester.spawn_command(cmd);
    sleep(DEFAULT_DELAY * 2);

    // Should find our test file in the target directory
    tester.assert_raw_output_contains("working-dir-test.txt");

    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests CLI input headers and footers override channel config
#[test]
fn test_cli_header_footer_overrides() {
    let mut tester = PtyTester::new();
    let temp_config = TempConfig::init();

    let channel_content = r#"
        [metadata]
        name = "header-test"
        description = "Header testing channel"

        [source]
        command = "echo 'header-item-1'; echo 'header-item-2'"

        [preview]
        command = "echo 'Channel preview: {}'"

        [ui.input_bar]
        header = "Channel Header"

        [ui.preview_panel]
        header = "Channel Preview Header"
        footer = "Channel Preview Footer"
    "#;
    temp_config
        .write_channel("header-test", channel_content)
        .unwrap();

    // CLI should override headers and footers
    let cmd = tv_with_args(&[
        "header-test",
        "--cable-dir",
        temp_config.cable_dir.to_str().unwrap(),
        "--input-header",
        "CLI Input Header",
        "--preview-header",
        "CLI Preview Header",
        "--preview-footer",
        "CLI Preview Footer",
    ]);

    let mut child = tester.spawn_command_tui(cmd);

    // Should show CLI headers instead of channel headers
    tester.assert_tui_frame_contains_all(&[
        "header-test",
        "header-item-1",
        "header-item-2",
        "CLI Preview Header",
        "CLI Preview Footer",
    ]);

    // Channel headers should not appear
    tester.assert_tui_frame_contains_none(&[
        "Channel Header",
        "Channel Preview Header",
        "Channel Preview Footer",
    ]);

    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that empty CLI arguments don't override non-empty config values
#[test]
fn test_empty_cli_args_dont_override() {
    let mut tester = PtyTester::new();
    let temp_config = TempConfig::init();

    let config_content = r#"
        [ui.input_bar]
        prompt = "config-prompt>"
    "#;
    temp_config.write_config(config_content).unwrap();

    // Run without CLI overrides - config values should be used
    let cmd = tv_with_args(&[
        "files",
        "--config-file",
        temp_config.config_file.to_str().unwrap(),
        "--cable-dir",
        DEFAULT_CABLE_DIR,
    ]);

    let mut child = tester.spawn_command_tui(cmd);

    // Should show config values
    tester.assert_tui_frame_contains("config-prompt>");

    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}
