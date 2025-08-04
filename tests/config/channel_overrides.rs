use super::super::common::*;

/// Tests that channel keybindings are properly merged with user config
#[test]
fn test_channel_keybindings_merge_with_user_config() {
    let mut tester = PtyTester::new();
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

    let cmd = tv_with_args(&[
        "custom-files",
        "--config-file",
        temp_config.config_file.to_str().unwrap(),
        "--cable-dir",
        temp_config.cable_dir.to_str().unwrap(),
    ]);

    let mut child = tester.spawn_command_tui(cmd);

    // Verify channel loads with merged keybindings
    tester.assert_tui_frame_contains("custom-files");

    // Try to toggle the preview off and on (it's initially on)
    tester.send(&ctrl('p'));
    tester.assert_not_tui_frame_contains("Preview ON");
    tester.send(&ctrl('c'));
    tester.assert_tui_frame_contains("Preview ON");

    tester.send(&ctrl('r'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests channel UI configuration merging
#[test]
fn test_channel_ui_merging() {
    let mut tester = PtyTester::new();
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

    let cmd = tv_with_args(&[
        "git-commits",
        "--config-file",
        temp_config.config_file.to_str().unwrap(),
        "--cable-dir",
        temp_config.cable_dir.to_str().unwrap(),
    ]);

    let mut child = tester.spawn_command_tui(cmd);

    // Verify channel loads with merged UI config
    tester.assert_tui_frame_contains_all(&[
        "git-commits",
        "git>",
        "Search commits...",
        "CHANNEL PREVIEW HEADER",
        // rounded border angle
        "╭",
    ]);

    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests channel source command variations and output parsing
#[test]
fn test_channel_source_command_variations() {
    let mut tester = PtyTester::new();
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

    let cmd = tv_with_args(&[
        "advanced-source",
        "--cable-dir",
        temp_config.cable_dir.to_str().unwrap(),
    ]);

    let mut child = tester.spawn_command_tui(cmd);

    // Verify advanced source configuration works
    tester.assert_tui_frame_contains_all(&[
        "Search commits",
        "commit>",
        "commit-abc123",
        "commit-def456",
        "commit-ghi789",
    ]);

    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests channel configuration with environment variables
#[test]
fn test_channel_environment_variables() {
    let mut tester = PtyTester::new();
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

    let cmd = tv_with_args(&[
        "env-aware",
        "--cable-dir",
        temp_config.cable_dir.to_str().unwrap(),
    ]);

    let mut child = tester.spawn_command_tui(cmd);

    // Verify environment-aware channel loads
    tester.assert_tui_frame_contains_all(&[
        "env-aware",
        "env-item-1",
        "env-item-2",
        "Preview with theme: ansi for",
    ]);

    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that CLI completely overrides channel prototype settings
#[test]
fn test_cli_completely_overrides_channel() {
    let mut tester = PtyTester::new();
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
    let cmd = tv_with_args(&[
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
    ]);

    let mut child = tester.spawn_command_tui(cmd);

    // Should show CLI values, not channel values
    tester.assert_tui_frame_contains_all(&[
        "override-me",
        "cli>",
        "CLI Header",
        "cli-item-1",
        "cli-item-3",
    ]);

    // Channel values should not appear
    tester.assert_tui_frame_contains_none(&[
        "channel>",
        "Channel Header",
        "channel-item-1",
    ]);

    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}
