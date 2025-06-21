mod common;

use common::*;

#[test]
fn tv_remote_control_shows() {
    let mut tester = PtyTester::new();
    let mut child = tester
        .spawn_command_tui(tv_local_config_and_cable_with_args(&["dirs"]));

    // open remote control mode
    tester.send(&ctrl('t'));

    tester.assert_tui_output_contains("──Remote Control──");

    // exit remote then app
    tester.send(&ctrl('c'));
    tester.send(&ctrl('c'));

    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

#[test]
fn tv_remote_control_zaps() {
    let mut tester = PtyTester::new();
    let mut child = tester
        .spawn_command_tui(tv_local_config_and_cable_with_args(&["dirs"]));

    // open remote control mode
    tester.send(&ctrl('t'));
    tester.send("files");
    tester.send(ENTER);

    tester.assert_tui_output_contains("── files ──");

    // exit remote then app
    tester.send(&ctrl('c'));
    tester.send(&ctrl('c'));

    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}
