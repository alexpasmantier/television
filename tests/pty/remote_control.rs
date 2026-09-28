//! The remote control panel.

use crate::common::*;

#[test]
fn test_remote_control_shows() {
    let pt = phantom();
    let s = tv_local_config_and_cable_with_args(&pt, &["dirs"])
        .start()
        .unwrap();
    s.wait().text("● dirs").until().unwrap();

    // open remote control mode
    s.send().key("ctrl-t").unwrap();

    // FIXME: me being lazy
    s.wait().text("● channels").until().unwrap();

    // exit remote mode; wait for the remote panel to disappear before
    // sending the app-level quit to avoid races.
    s.send().key("ctrl-c").unwrap();
    s.wait().text_absent("● channels").until().unwrap();
    s.send().key("ctrl-c").unwrap();

    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_remote_control_zaps() {
    let pt = phantom();
    let s = tv_local_config_and_cable_with_args(&pt, &["dirs"])
        .start()
        .unwrap();
    s.wait().text("● dirs").until().unwrap();

    // open remote control mode
    s.send().key("ctrl-t").unwrap();
    s.send().type_text("files").unwrap();
    s.send().key("enter").unwrap();

    s.wait().text("● files").until().unwrap();

    // exit remote then app
    s.send().key("ctrl-c").unwrap();
    s.send().key("ctrl-c").unwrap();

    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_toggle_remote_control_keybinding() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(&pt, &["files"])
        .start()
        .unwrap();
    s.wait().text("● files").until().unwrap();

    // Send Ctrl+T to open remote control panel
    s.send().key("ctrl-t").unwrap();

    s.wait().text("● channels").until().unwrap();

    // Send Ctrl+C to exit remote control mode; wait for the panel to
    // disappear before sending the app-level quit to avoid races.
    s.send().key("ctrl-c").unwrap();
    s.wait().text_absent("● channels").until().unwrap();

    // Send Ctrl+C again to exit the application
    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_toggle_preview_disabled_in_remote_control_mode() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(&pt, &["files"])
        .start()
        .unwrap();

    // Verify preview is initially visible
    s.wait().text("▏").until().unwrap();

    // Enter remote control mode
    s.send().key("ctrl-t").unwrap();

    s.wait()
        .text("● channels")
        .text("● channels")
        .until()
        .unwrap();

    // Try to toggle preview - this should NOT work in remote control mode
    s.send().key("ctrl-o").unwrap();

    // Verify we're still in remote control mode and preview is still visible
    // (the toggle should have been ignored)
    s.wait()
        .text("● channels")
        .text("● channels")
        .until()
        .unwrap();

    // Exit remote control mode
    s.send().key("ctrl-t").unwrap();

    // Verify we're back in channel mode
    s.wait().text_absent("● channels").until().unwrap();
    s.wait().text("▏").until().unwrap();

    // Verify preview toggle works again in channel mode
    s.send().key("ctrl-o").unwrap();
    s.wait().text_absent("▏").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_no_remote_hides_remote_panel() {
    let pt = phantom();

    let s =
        tv_local_config_and_cable_with_args(&pt, &["files", "--no-remote"])
            .start()
            .unwrap();

    s.wait().text("● files").until().unwrap();

    // with the remote disabled, ctrl-t is a no-op
    s.send().key("ctrl-t").unwrap();
    assert_frame_not_contains(&s, "● channels");

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_show_remote_flag_shows_remote_panel() {
    let pt = phantom();

    let s =
        tv_local_config_and_cable_with_args(&pt, &["files", "--show-remote"])
            .start()
            .unwrap();

    s.wait().text("● channels").until().unwrap();

    // Send Ctrl+C to exit remote control; wait for it to close before
    // sending the app-level quit to avoid races.
    s.send().key("ctrl-c").unwrap();
    s.wait().text_absent("● channels").until().unwrap();
    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_hide_remote_flag_hides_remote_panel() {
    let pt = phantom();

    let s =
        tv_local_config_and_cable_with_args(&pt, &["files", "--hide-remote"])
            .start()
            .unwrap();

    s.wait().text("● files").until().unwrap();
    assert_frame_not_contains(&s, "● channels");

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_hide_remote_conflicts_with_no_remote() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--hide-remote", "--no-remote"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

#[test]
fn test_hide_and_show_remote_conflict_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--hide-remote", "--show-remote"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}
