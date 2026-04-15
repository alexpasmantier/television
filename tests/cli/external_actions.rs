//! Tests for external actions functionality.
//!
//! These tests verify that external actions defined in channel TOML files work correctly,
//! including keybinding integration and command execution.

use std::fs;

use tempfile::TempDir;

use super::super::common::*;

/// Helper to create a custom cable directory with external actions.
fn write_toml_config(
    cable_dir: &std::path::Path,
    filename: &str,
    content: &str,
) {
    let toml_path = cable_dir.join(filename);
    fs::write(&toml_path, content).unwrap();
}

const FILES_TOML_WITH_ACTIONS: &str = r#"
[metadata]
name = "files"
description = "A channel to select files and directories"
requirements = ["fd", "bat"]

[source]
command = ["fd -t f", "fd -t f -H"]

[preview]
command = "bat -n --color=always '{}'"
env = { BAT_THEME = "ansi" }

[keybindings]
shortcut = "f1"
f8 = "actions:thebatman"
f9 = "actions:lsman"

[actions.thebatman]
description = "show file content"
command = "cat '{}'"
mode = "execute"

[actions.lsman]
description = "show stats"
command = "ls '{}'"
mode = "execute"
"#;

/// Tests that external actions execute properly when triggered by keybindings.
#[test]
fn test_external_action_lsman_with_f9() {
    let pt = phantom();

    let cable_dir = TempDir::new().unwrap().path().join("custom_cable");
    fs::create_dir_all(&cable_dir).unwrap();
    write_toml_config(&cable_dir, "files.toml", FILES_TOML_WITH_ACTIONS);

    let s = tv_with_args(
        &pt,
        &[
            "--cable-dir",
            cable_dir.to_str().unwrap(),
            "--config-file",
            DEFAULT_CONFIG_FILE,
            "files",
            "--input",
            "LICENSE",
        ],
    )
    .start()
    .unwrap();

    s.wait().text("LICENSE").until().unwrap();

    // Send F9 to trigger the "lsman" action (mapped to ls command)
    s.send().key("f9").unwrap();

    // The external action should have executed "ls 'LICENSE'" and exited
    s.wait().text("LICENSE").timeout_ms(2000).until().unwrap();

    s.wait().exit_code(0).until().unwrap();
}

/// Tests that external actions execute properly with F8 keybinding.
#[test]
fn test_external_action_thebatman_with_f8() {
    let pt = phantom();

    let cable_dir = TempDir::new().unwrap().path().join("custom_cable_f8");
    fs::create_dir_all(&cable_dir).unwrap();
    write_toml_config(&cable_dir, "files.toml", FILES_TOML_WITH_ACTIONS);

    let s = tv_with_args(
        &pt,
        &[
            "--cable-dir",
            cable_dir.to_str().unwrap(),
            "--config-file",
            DEFAULT_CONFIG_FILE,
            "files",
            "--input",
            "LICENSE",
        ],
    )
    .start()
    .unwrap();

    s.wait().text("LICENSE").until().unwrap();

    // Send F8 to trigger the "thebatman" action (mapped to cat command)
    s.send().key("f8").unwrap();

    // The command should execute and television should exit
    s.wait()
        .text("Copyright (c)")
        .timeout_ms(2000)
        .until()
        .unwrap();

    s.wait().exit_code(0).until().unwrap();
}
