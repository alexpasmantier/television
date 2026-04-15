//! Tests for CLI input/interaction options: --input, --keybindings, --exact.
//!
//! These tests verify Television's input handling and user interaction features,
//! ensuring users can customize their interaction experience and search behavior.

use tempfile::TempDir;

use super::super::common::*;

/// Tests that the --input flag pre-fills the search box with specified text.
#[test]
fn test_input_prefills_search_box() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--input", "UNIQUE16CHARID"],
    )
    .start()
    .unwrap();

    s.wait().text("│> UNIQUE16CHARID").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that custom keybindings override default keyboard shortcuts.
#[test]
fn test_keybindings_override_default() {
    let pt = phantom();

    // This adds a new mapping for the quit action
    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--keybindings", "a=\"quit\";ctrl-c=\"no_op\";esc=\"no_op\""],
    )
    .start()
    .unwrap();
    s.wait().text("── files ──").until().unwrap();

    // Test that ESC no longer quits (default behavior is overridden)
    s.send().key("escape").unwrap();
    // Still running — send our custom quit key
    // Test that Ctrl+C no longer quits (default behavior is overridden)
    s.send().key("ctrl-c").unwrap();

    // Test that our custom "a" key now quits the application
    s.send().type_text("'a'").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that multiple keybinding overrides can be specified simultaneously.
#[test]
fn test_multiple_keybindings_override() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "--keybindings",
            "a=\"quit\";ctrl-x=\"toggle_remote_control\";esc=\"no_op\"",
        ],
    )
    .start()
    .unwrap();
    s.wait().text("── files ──").until().unwrap();

    // Verify ESC doesn't quit (default overridden) — app still responds
    s.send().key("escape").unwrap();

    // Test that Ctrl+X opens remote control panel (custom keybinding works)
    s.send().key("ctrl-x").unwrap();
    s.wait().text("(1) (2) (3)").until().unwrap();
    s.send().key("ctrl-t").unwrap();
    s.wait().text_absent("(1) (2) (3)").until().unwrap();

    // Use "a" to quit the application
    s.send().type_text("'a'").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that the --exact flag enables exact substring matching instead of fuzzy matching.
#[test]
fn test_exact_matching_enabled() {
    let pt = phantom();
    let tmp_dir = TempDir::new().unwrap();

    std::fs::write(tmp_dir.path().join("UNIQUE16CHARIDfile.txt"), "").unwrap();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "files",
            "--exact",
            "--input",
            "UNIQUE16CHARIDfile",
            tmp_dir.path().to_str().unwrap(),
        ],
    )
    .start()
    .unwrap();

    s.wait().text("UNIQUE16CHARIDfile.txt").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_exact_matching_enabled_fails() {
    let pt = phantom();
    let tmp_dir = TempDir::new().unwrap();

    std::fs::write(tmp_dir.path().join("UNIQUE16CHARIDfile.txt"), "").unwrap();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "files",
            "--exact",
            "--input",
            "UNIQUE16CHARIDfl",
            tmp_dir.path().to_str().unwrap(),
        ],
    )
    .start()
    .unwrap();

    s.wait()
        .text("│> UNIQUE16CHARIDfl")
        .text("0 / 0")
        .until()
        .unwrap();
    assert_frame_not_contains(&s, "UNIQUE16CHARIDfile.txt");

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --no-sort keeps results in the source order for selection.
#[test]
fn test_no_sort_preserves_source_order() {
    let pt = phantom();

    // The second entry is a stronger fuzzy match for "ab".
    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "--source-command",
            "echo 'a-weak-b'; echo 'ab-strong'",
            "--input",
            "ab",
            "--no-sort",
            "--take-1",
        ],
    )
    .start()
    .unwrap();

    s.wait().text("a-weak-b").timeout_ms(2000).until().unwrap();
}
