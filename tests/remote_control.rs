mod common;

use common::*;

#[test]
fn tv_remote_control_shows() {
    let pt = phantom();
    let s = tv_local_config_and_cable_with_args(&pt, &["dirs"])
        .start()
        .unwrap();
    s.wait().text("── dirs ──").until().unwrap();

    // open remote control mode
    s.send().key("ctrl-t").unwrap();

    // FIXME: me being lazy
    s.wait().text("(1) (2) (3)").until().unwrap();

    // exit remote mode; wait for the remote panel to disappear before
    // sending the app-level quit to avoid races.
    s.send().key("ctrl-c").unwrap();
    s.wait().text_absent("(1) (2) (3)").until().unwrap();
    s.send().key("ctrl-c").unwrap();

    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn tv_remote_control_zaps() {
    let pt = phantom();
    let s = tv_local_config_and_cable_with_args(&pt, &["dirs"])
        .start()
        .unwrap();
    s.wait().text("── dirs ──").until().unwrap();

    // open remote control mode
    s.send().key("ctrl-t").unwrap();
    s.send().type_text("files").unwrap();
    s.send().key("enter").unwrap();

    s.wait().text("── files ──").until().unwrap();

    // exit remote then app
    s.send().key("ctrl-c").unwrap();
    s.send().key("ctrl-c").unwrap();

    s.wait().exit_code(0).until().unwrap();
}
