//! Tests for CLI UI/behavioral integration: toggling panels, scrolling, clipboard, reload, etc.
//!
//! These tests verify Television's interactive UI behaviors and keyboard shortcuts,
//! ensuring users can effectively navigate and control the interface during operation.
//! These are integration tests that combine CLI setup with interactive behavior.

use tempfile::TempDir;

use super::super::common::*;

/// Tests that the toggle preview keybinding functionality works correctly.
#[test]
fn test_toggle_preview_keybinding() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(&pt, &["files"])
        .start()
        .unwrap();

    // Verify preview is initially visible (two panels side by side)
    s.wait().text("───╮╭───").until().unwrap();

    // Send Ctrl+O to toggle preview off
    s.send().key("ctrl-o").unwrap();

    // Verify preview is now hidden
    s.wait().text_absent("───╮╭───").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that the toggle remote control keybinding functionality works correctly.
#[test]
fn test_toggle_remote_control_keybinding() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(&pt, &["files"])
        .start()
        .unwrap();
    s.wait().text("── files ──").until().unwrap();

    // Send Ctrl+T to open remote control panel
    s.send().key("ctrl-t").unwrap();

    s.wait().text("(1) (2) (3)").until().unwrap();

    // Send Ctrl+C to exit remote control mode; wait for the panel to
    // disappear before sending the app-level quit to avoid races.
    s.send().key("ctrl-c").unwrap();
    s.wait().text_absent("(1) (2) (3)").until().unwrap();

    // Send Ctrl+C again to exit the application
    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that the toggle status bar keybinding functionality works correctly.
#[test]
fn test_toggle_status_bar_keybinding() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--keybindings", "ctrl-k = \"toggle_status_bar\""],
    )
    .start()
    .unwrap();
    s.wait().text("CHANNEL  files").until().unwrap();

    // Send Ctrl+K to toggle status bar off
    s.send().key("ctrl-k").unwrap();

    s.wait().text_absent("CHANNEL  files").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that the toggle help keybinding functionality works correctly.
#[test]
fn test_toggle_help_keybinding() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(&pt, &["files"])
        .start()
        .unwrap();
    s.wait().text("── files ──").until().unwrap();

    // Send Ctrl+H to open help panel
    s.send().key("ctrl-h").unwrap();

    s.wait().text("───── Help ─────").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that the preview scrolling keybindings functionality works correctly.
#[test]
fn test_scroll_preview_keybindings() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--input", "README.md"],
    )
    .start()
    .unwrap();
    s.wait().text("││   1").until().unwrap();

    // Send Page Down to scroll preview down
    s.send().key("pagedown").unwrap();
    s.send().key("pagedown").unwrap();

    s.wait().text_absent("││   1").until().unwrap();

    // Send Page Up to scroll preview up
    s.send().key("pageup").unwrap();
    s.send().key("pageup").unwrap();

    s.wait().text("││   1").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that the reload source keybinding functionality works correctly.
#[test]
fn test_reload_source_keybinding() {
    let pt = phantom();
    let tmp_dir = TempDir::new().unwrap();

    // Create initial file to be detected
    std::fs::write(tmp_dir.path().join("UNIQUE16CHARIDfile.txt"), "").unwrap();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "files",
            "--input",
            "UNIQUE16CHARID",
            tmp_dir.path().to_str().unwrap(),
        ],
    )
    .start()
    .unwrap();

    s.wait().text("UNIQUE16CHARIDfile.txt").until().unwrap();

    // add another file to be detected
    std::fs::write(tmp_dir.path().join("UNIQUE16CHARIDcontrol.txt"), "")
        .unwrap();

    // Send Ctrl+R to reload the source command
    s.send().key("ctrl-r").unwrap();

    // Verify the new file appears in the TUI as well as the existing one
    s.wait()
        .text("UNIQUE16CHARIDcontrol.txt")
        .text("UNIQUE16CHARIDfile.txt")
        .until()
        .unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that the cycle sources keybinding functionality works correctly.
#[test]
fn test_cycle_sources_keybinding() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(&pt, &["files"])
        .start()
        .unwrap();
    s.wait().text("── files ──").until().unwrap();

    // Send Ctrl+S to cycle to next source
    s.send().key("ctrl-s").unwrap();
    s.send().type_text(".config").unwrap();

    s.wait().text(".config/config.toml").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that preview toggle is disabled when in remote control mode.
#[test]
fn test_toggle_preview_disabled_in_remote_control_mode() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(&pt, &["files"])
        .start()
        .unwrap();

    // Verify preview is initially visible (two panels side by side)
    s.wait()
        .text("╭───────────────────────── files ──────────────────────────╮╭─")
        .text("───╮╭───")
        .until()
        .unwrap();

    // Enter remote control mode
    s.send().key("ctrl-t").unwrap();

    s.wait()
        .text("(1) (2) (3)")
        .text("Back to Channel:")
        .until()
        .unwrap();

    // Try to toggle preview - this should NOT work in remote control mode
    s.send().key("ctrl-o").unwrap();

    // Verify we're still in remote control mode and preview is still visible
    // (the toggle should have been ignored)
    s.wait()
        .text("(1) (2) (3)")
        .text("Back to Channel:")
        .text("╭───────────────────────── files ──────────────────────────╮╭─")
        .until()
        .unwrap();

    // Exit remote control mode
    s.send().key("ctrl-t").unwrap();

    // Verify we're back in channel mode
    s.wait().text_absent("Back to Channel:").until().unwrap();
    s.wait().text("───╮╭───").until().unwrap();

    // Verify preview toggle works again in channel mode
    s.send().key("ctrl-o").unwrap();
    s.wait().text_absent("───╮╭───").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}
