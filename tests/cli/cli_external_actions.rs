//! Tests for external actions functionality.
//!
//! These tests verify that external actions defined in channel TOML files work correctly,
//! including keybinding integration and command execution.

use super::common::*;
use std::{fs, thread::sleep, time::Duration};
use tempfile::TempDir;

// ANSI escape sequences for function keys
const F8_KEY: &str = "\x1b[19~";
const F9_KEY: &str = "\x1b[20~";

/// Tests that external actions execute properly when triggered by keybindings.
#[test]
fn test_external_action_lsman_with_f9() {
    let mut tester = PtyTester::new();

    // Create a temporary directory for the custom cable
    let temp_dir = TempDir::new().unwrap();
    let cable_dir = temp_dir.path();

    // Create a custom files.toml with external actions
    let files_toml_content = r#"
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
f8 = "thebatman"
f9 = "lsman"

[actions.thebatman]
description = "cats the file"
command = "bat '{}'"
env = { BAT_THEME = "ansi" }

[actions.lsman]
description = "show stats"
command = "ls '{}'"
"#;

    let files_toml_path = cable_dir.join("files.toml");
    fs::write(&files_toml_path, files_toml_content).unwrap();

    // Use the LICENSE file as input since it exists in the repo
    let mut cmd = tv();
    cmd.args(["--cable-dir", cable_dir.to_str().unwrap()]);
    cmd.args(["--config-file", DEFAULT_CONFIG_FILE]);
    cmd.args(["files", "--input", "LICENSE"]);

    let mut child = tester.spawn_command_tui(cmd);

    // Wait for the UI to load - we should see LICENSE in the selection
    sleep(DEFAULT_DELAY);
    tester.assert_tui_frame_contains("LICENSE");

    // Send F9 to trigger the "lsman" action (mapped to ls command)
    tester.send(F9_KEY);

    // Give time for the action to execute and television to exit
    sleep(Duration::from_millis(500));

    // The external action should have executed "ls 'LICENSE'" and exited
    tester.assert_raw_output_contains("LICENSE");

    // Process should exit successfully after executing the external command
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

/// Tests that external actions execute properly with F8 keybinding.
#[test]
fn test_external_action_thebatman_with_f8() {
    let mut tester = PtyTester::new();

    // Create a temporary directory for the custom cable
    let temp_dir = TempDir::new().unwrap();
    let cable_dir = temp_dir.path();

    // Create a custom files.toml with external actions
    let files_toml_content = r#"
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
f8 = "thebatman"
f9 = "lsman"

[actions.thebatman]
description = "cats the file"
command = "bat '{}'"
env = { BAT_THEME = "ansi" }

[actions.lsman]
description = "show stats"
command = "ls '{}'"
"#;

    let files_toml_path = cable_dir.join("files.toml");
    fs::write(&files_toml_path, files_toml_content).unwrap();

    // Use the LICENSE file as input since it exists in the repo
    let mut cmd = tv();
    cmd.args(["--cable-dir", cable_dir.to_str().unwrap()]);
    cmd.args(["--config-file", DEFAULT_CONFIG_FILE]);
    cmd.args(["files", "--input", "LICENSE"]);

    let mut child = tester.spawn_command_tui(cmd);

    // Wait for the UI to load
    sleep(DEFAULT_DELAY);

    // Send F8 to trigger the "thebatman" action (mapped to bat command)
    tester.send(F8_KEY);

    // Give time for the action to execute
    sleep(Duration::from_millis(500));

    // The command should execute and television should exit
    tester.assert_raw_output_contains("Copyright (c)");

    // Check that the process has finished
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}
