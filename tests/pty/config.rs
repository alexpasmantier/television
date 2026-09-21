//! Config and cable directory loading, and how CLI, channel and user config merge.

use tempfile::TempDir;

use crate::common::*;

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

    s.wait().text("● files").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

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

#[test]
fn test_channel_keybindings_merge_with_user_config() {
    let pt = phantom();
    let temp_config = TempConfig::init();

    temp_config
        .write_config(
            r#"
                [keybindings]
                "ctrl-c" = "quit"
                "ctrl-r" = "reload_source"
                "ctrl-p" = "toggle_preview"
                "f1" = "toggle_help"
            "#,
        )
        .unwrap();

    // Create channel similar to files.toml but with additional keybindings
    temp_config
        .write_channel(
            "custom-files",
            r#"
                [metadata]
                name = "custom-files"
                description = "Enhanced files channel with custom keybindings"
                requirements = ["fd", "bat"]

                [source]
                command = ["fd -t f", "fd -t f -H"]

                [preview]
                command = "bat -n --color=always '{}'"
                env = { BAT_THEME = "ansi" }

                [ui.preview_panel]
                header = "Preview ON"

                [keybindings]
                "ctrl-c" = "toggle_preview"
                "ctrl-r" = "quit"
            "#,
        )
        .unwrap();

    let s = tv_with_args(
        &pt,
        &[
            "custom-files",
            "--config-file",
            temp_config.config_file.to_str().unwrap(),
            "--cable-dir",
            temp_config.cable_dir.to_str().unwrap(),
        ],
    )
    .start()
    .unwrap();

    s.wait()
        .text("custom-files")
        .text("Preview ON")
        .until()
        .unwrap();

    // Try to toggle the preview off and on (it's initially on)
    s.send().key("ctrl-p").unwrap();
    s.wait().text_absent("Preview ON").until().unwrap();
    s.send().key("ctrl-c").unwrap();
    s.wait().text("Preview ON").until().unwrap();

    s.send().key("ctrl-r").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_channel_ui_merging() {
    let pt = phantom();
    let temp_config = TempConfig::init();

    let config_content = r#"
        [ui.preview_panel]
        size = 50
        border_type = "plain"
        header = "CONFIG PREVIEW HEADER"
    "#;

    let channel_content = r#"
        [metadata]
        name = "git-commits"

        [source]
        command = "echo 'commit-1'; echo 'commit-2'; echo 'commit-3'"

        [preview]
        command = "echo 'Previewing commit: {}'"

        [ui]
        orientation = "landscape"

        [ui.preview_panel]
        size = 70  # Git diffs need more space
        header = "CHANNEL PREVIEW HEADER"
        border_type = "rounded"

        [ui.input_bar]
        header = "Search commits..."
        prompt = "git> "

        [ui.results_panel]
        border_type = "thick"
"#;
    temp_config.write_config(config_content).unwrap();
    temp_config
        .write_channel("git-commits", channel_content)
        .unwrap();

    let s = tv_with_args(
        &pt,
        &[
            "git-commits",
            "--config-file",
            temp_config.config_file.to_str().unwrap(),
            "--cable-dir",
            temp_config.cable_dir.to_str().unwrap(),
        ],
    )
    .start()
    .unwrap();

    s.wait()
        .text("git-commits")
        .text("git>")
        .text("Search commits...")
        .text("CHANNEL PREVIEW HEADER")
        // rounded border angle
        .text("╭")
        .until()
        .unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_channel_source_command_variations() {
    let pt = phantom();
    let temp_config = TempConfig::init();

    let channel_content = r#"
        [metadata]
        name = "advanced-source"
        description = "Channel with advanced source configuration"

        [source]
        command = "echo 'commit-abc123 Fix critical bug'; echo 'commit-def456 Add new feature'; echo 'commit-ghi789 Update documentation'"
        output = "{split: :0}"  # Extract first part before space
        display = "Commit: {0} - {1..}"  # Custom display format

        [preview]
        command = "echo 'Details for commit: {0}'; echo 'Author: Test User'; echo 'Date: 2024-01-01'"

        [ui.input_bar]
        header = "Search commits"
        prompt = "commit> "

        [ui.preview_panel]
        header = "Commit Details: {0}"
        size = 55
    "#;

    temp_config
        .write_channel("advanced source", channel_content)
        .unwrap();

    let s = tv_with_args(
        &pt,
        &[
            "advanced-source",
            "--cable-dir",
            temp_config.cable_dir.to_str().unwrap(),
        ],
    )
    .start()
    .unwrap();

    s.wait()
        .text("Search commits")
        .text("commit>")
        .text("commit-abc123")
        .text("commit-def456")
        .text("commit-ghi789")
        .until()
        .unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_channel_environment_variables() {
    let pt = phantom();
    let temp_config = TempConfig::init();

    let channel_content = r#"
        [metadata]
        name = "env-aware"
        description = "Channel that uses environment variables"

        [source]
        command = "echo 'env-item-1'; echo 'env-item-2'"

        [preview]
        command = "echo \"Preview with theme: $BAT_THEME for {}\""
        env = { BAT_THEME = "ansi", CUSTOM_VAR = "test-value" }

        [ui.input_bar]
        header = "Environment-aware channel"
    "#;

    temp_config
        .write_channel("env-aware", channel_content)
        .unwrap();

    let s = tv_with_args(
        &pt,
        &[
            "env-aware",
            "--cable-dir",
            temp_config.cable_dir.to_str().unwrap(),
        ],
    )
    .start()
    .unwrap();

    s.wait()
        .text("env-aware")
        .text("env-item-1")
        .text("env-item-2")
        .text("Preview with theme: ansi for")
        .until()
        .unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_cli_completely_overrides_channel() {
    let pt = phantom();
    let temp_config = TempConfig::init();

    let channel_content = r#"
        [metadata]
        name = "override-me"
        description = "Channel designed to be overridden"

        [source]
        command = "echo 'channel-item-1'; echo 'channel-item-2'"

        [preview]
        command = "echo 'Channel preview: {}'"

        [ui]
        ui_scale = 10

        [ui.input_bar]
        prompt = "channel> "
        header = "Channel Header"

        [ui.preview_panel]
        size = 40
        header = "Channel Preview"
    "#;

    temp_config
        .write_channel("override-me", channel_content)
        .unwrap();

    // CLI should override everything
    let s = tv_with_args(
        &pt,
        &[
            "override-me",
            "--cable-dir",
            temp_config.cable_dir.to_str().unwrap(),
            "--source-command",
            "echo 'cli-item-1'; echo 'cli-item-2'; echo 'cli-item-3'",
            "--preview-command",
            "echo 'CLI preview: {}'",
            "--ui-scale",
            "80",
            "--input-prompt",
            "cli> ",
            "--input-header",
            "CLI Header",
            "--preview-size",
            "75",
            "--preview-header",
            "CLI Preview",
        ],
    )
    .start()
    .unwrap();

    s.wait()
        .text("override-me")
        .text("cli>")
        .text("CLI Header")
        .text("cli-item-1")
        .text("cli-item-3")
        .until()
        .unwrap();

    assert_frame_not_contains_any(
        &s,
        &["channel>", "Channel Header", "channel-item-1"],
    );

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

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
            "--input",
            "Cargo.toml",
        ],
    )
    .start()
    .unwrap();
    s.wait().text("position-prompt>").until().unwrap();

    let frame = stable_frame(&s);
    let prompt_index = frame
        .find("position-prompt>")
        .expect("Expected input prompt in frame");
    // anchor on a results entry (the old "Default" header anchor now
    // matches the multi-source indicator in the input row itself)
    let results_index = frame
        .find("Cargo.toml")
        .expect("Expected a results entry in frame");

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
