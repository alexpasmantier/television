//! Keybinding overrides.

use crate::common::*;

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
    s.wait().text("● files").until().unwrap();

    // Test that ESC no longer quits (default behavior is overridden)
    s.send().key("escape").unwrap();
    // Still running — send our custom quit key
    // Test that Ctrl+C no longer quits (default behavior is overridden)
    s.send().key("ctrl-c").unwrap();

    // Test that our custom "a" key now quits the application
    s.send().type_text("'a'").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

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
    s.wait().text("● files").until().unwrap();

    // Note: we intentionally don't re-test esc=no_op here — that's already
    // covered by test_keybindings_override_default. Sending escape
    // immediately followed by another byte is also racy with crossterm's
    // escape-disambiguation window (a bare ESC followed quickly by another
    // key can be interpreted as an alt-key combo). This test focuses on
    // ctrl-x toggling remote control.

    // Test that Ctrl+X opens remote control panel (custom keybinding works)
    s.send().key("ctrl-x").unwrap();
    s.wait()
        .text("● channels")
        .timeout_ms(wait_timeout_ms())
        .until()
        .unwrap();
    s.send().key("ctrl-t").unwrap();
    s.wait()
        .text_absent("● channels")
        .timeout_ms(wait_timeout_ms())
        .until()
        .unwrap();

    // Use "a" to quit the application
    s.send().type_text("'a'").unwrap();
    s.wait().exit_code(0).until().unwrap();
}
