use tempfile::TempDir;

use super::super::common::*;

/// Tests CLI override of UI configuration from config file
#[test]
fn test_cli_ui_overrides_config_file() {
    let pt = phantom();
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
    let s = tv_with_args(
        &pt,
        &[
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
        ],
    )
    .start()
    .unwrap();

    // Verify the application starts with CLI overrides
    s.wait()
        .text("files")
        .text("cli-prompt>")
        // rounded border (input)
        .text("╭")
        // thick border (preview)
        .text("┏")
        .until()
        .unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests CLI overrides of channel-specific configuration
#[test]
fn test_cli_overrides_channel_source_and_preview() {
    let pt = phantom();
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
    let s = tv_with_args(
        &pt,
        &[
            "override-test",
            "--cable-dir",
            temp_config.cable_dir.to_str().unwrap(),
            "--source-command",
            "echo 'cli-item-1'; echo 'cli-item-2'; echo 'cli-item-3'",
            "--preview-command",
            "echo 'CLI preview: {}'",
            "--preview-size",
            "60",
        ],
    )
    .start()
    .unwrap();

    // Should see CLI source command output instead of channel's
    s.wait()
        .text("override-test")
        .text("cli-item-1")
        .text("cli-item-2")
        .text("cli-item-3")
        .until()
        .unwrap();

    // Channel items should not appear
    assert_frame_not_contains(&s, "channel-item-1");

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests CLI working directory parameter
#[test]
fn test_cli_working_directory_override() {
    let pt = phantom();
    let temp_dir = TempDir::new().unwrap();

    std::fs::write(
        temp_dir.path().join("working-dir-test.txt"),
        "test content",
    )
    .unwrap();

    let s = tv_with_args(
        &pt,
        &[
            "files",
            "--cable-dir",
            DEFAULT_CABLE_DIR,
            "--input",
            "working-dir-test",
            "--take-1",
            &temp_dir.path().to_string_lossy(),
        ],
    )
    .start()
    .unwrap();

    s.wait()
        .text("working-dir-test.txt")
        .timeout_ms(wait_timeout_ms())
        .until()
        .unwrap();

    s.wait().exit_code(0).until().unwrap();
}

/// Tests CLI input headers and footers override channel config
#[test]
fn test_cli_header_footer_overrides() {
    let pt = phantom();
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

    let s = tv_with_args(
        &pt,
        &[
            "header-test",
            "--cable-dir",
            temp_config.cable_dir.to_str().unwrap(),
            "--input-header",
            "CLI Input Header",
            "--preview-header",
            "CLI Preview Header",
            "--preview-footer",
            "CLI Preview Footer",
        ],
    )
    .start()
    .unwrap();

    // Should show CLI headers instead of channel headers
    s.wait()
        .text("header-test")
        .text("header-item-1")
        .text("header-item-2")
        .text("CLI Preview Header")
        .text("CLI Preview Footer")
        .until()
        .unwrap();

    // Channel headers should not appear
    assert_frame_not_contains_any(
        &s,
        &[
            "Channel Header",
            "Channel Preview Header",
            "Channel Preview Footer",
        ],
    );

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that empty CLI arguments don't override non-empty config values
#[test]
fn test_empty_cli_args_dont_override() {
    let pt = phantom();
    let temp_config = TempConfig::init();

    let config_content = r#"
        [ui.input_bar]
        prompt = "config-prompt>"
    "#;
    temp_config.write_config(config_content).unwrap();

    let s = tv_with_args(
        &pt,
        &[
            "files",
            "--config-file",
            temp_config.config_file.to_str().unwrap(),
            "--cable-dir",
            DEFAULT_CABLE_DIR,
        ],
    )
    .start()
    .unwrap();

    s.wait().text("config-prompt>").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests CLI override of input bar position
#[test]
fn test_cli_input_position_override() {
    let pt = phantom();

    let s = tv_with_args(
        &pt,
        &[
            "files",
            "--cable-dir",
            DEFAULT_CABLE_DIR,
            "--input-prompt",
            "position-prompt>",
            "--input-position",
            "bottom",
        ],
    )
    .start()
    .unwrap();
    s.wait().text("position-prompt>").until().unwrap();

    let frame = stable_frame(&s);
    let prompt_index = frame
        .find("position-prompt>")
        .expect("Expected input prompt in frame");
    let results_index = frame
        .find("Results")
        .expect("Expected results header in frame");

    assert!(
        prompt_index > results_index,
        "Expected input bar below results when using --input-position=bottom.\nFrame:\n{}",
        frame
    );

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_action_id_mismatch_validation_error() {
    let pt = phantom();
    let temp_config = TempConfig::init();

    let channel_content = r#"
        [metadata]
        name = "validation-test"
        description = "Channel for testing action validation"

        [source]
        command = "echo 'test-item-1'; echo 'test-item-2'"

        [keybindings]
        f12 = ["actions:edit_text", "toggle_preview"]

        [actions.edit]
        description = "Edit the selected file"
        command = "vi '{}'"
    "#;

    temp_config
        .write_channel("validation-test", channel_content)
        .unwrap();

    let s = tv_with_args(
        &pt,
        &[
            "validation-test",
            "--cable-dir",
            temp_config.cable_dir.to_str().unwrap(),
        ],
    )
    .start()
    .unwrap();

    s.wait()
        .text("Action 'actions:edit_text' referenced in keybinding not found in actions section")
        .until()
        .unwrap();
}
