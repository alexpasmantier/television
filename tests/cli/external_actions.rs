//! Tests for external actions functionality.
//!
//! These tests verify that external actions defined in channel TOML files work correctly,
//! including keybinding integration and command execution.

use super::super::common::*;
use std::{fs, thread::sleep, time::Duration};

// ANSI escape sequences for function keys
const F8_KEY: &str = "\x1b[19~";
const F9_KEY: &str = "\x1b[20~";

/// Helper to create a custom cable directory with external actions.
fn write_toml_config(
    cable_dir: &std::path::Path,
    filename: &str,
    content: &str,
) {
    let toml_path = cable_dir.join(filename);
    fs::write(&toml_path, content).unwrap();
}

/// Tests that external actions execute properly when triggered by keybindings.
#[test]
fn test_external_action_lsman_with_f9() {
    let mut tester = PtyTester::new();

    // Use TARGET_DIR for consistency with other tests
    let cable_dir = std::path::Path::new(TARGET_DIR).join("custom_cable");
    fs::create_dir_all(&cable_dir).unwrap();

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

    write_toml_config(&cable_dir, "files.toml", files_toml_content);

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

    let cable_dir = std::path::Path::new(TARGET_DIR).join("custom_cable_f8");
    fs::create_dir_all(&cable_dir).unwrap();

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

    write_toml_config(&cable_dir, "files.toml", files_toml_content);

    // Use the LICENSE file as input since it exists in the repo
    let mut cmd = tv();
    cmd.args(["--cable-dir", cable_dir.to_str().unwrap()]);
    cmd.args(["--config-file", DEFAULT_CONFIG_FILE]);
    cmd.args(["files", "--input", "LICENSE"]);

    let mut child = tester.spawn_command_tui(cmd);

    // Wait for the UI to load
    sleep(DEFAULT_DELAY);

    // Send F8 to trigger the "thebatman" action (mapped to cat command)
    tester.send(F8_KEY);

    // Give time for the action to execute
    sleep(Duration::from_millis(500));

    // The command should execute and television should exit
    tester.assert_raw_output_contains("Copyright (c)");

    // Check that the process has finished
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}
