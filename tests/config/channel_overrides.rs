use super::super::common::*;

/// Tests that channel keybindings are properly merged with user config
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

/// Tests channel UI configuration merging
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

/// Tests channel source command variations and output parsing
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

/// Tests channel configuration with environment variables
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

/// Tests that CLI completely overrides channel prototype settings
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
