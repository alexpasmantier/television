mod common;

use common::*;

#[test]
fn tv_remote_control_shows() {
    let mut tester = PtyTester::new();
    let mut child = tester
        .spawn_command_tui(tv_local_config_and_cable_with_args(&["dirs"]));

    // open remote control mode
    tester.write_input(&ctrl('t'));

    tester.assert_tui_output_contains("──Remote Control──");

    // exit remote then app
    tester.write_input(&ctrl('c'));
    tester.write_input(&ctrl('c'));

    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

#[test]
fn tv_remote_control_zaps() {
    let mut tester = PtyTester::new();
    let mut child = tester
        .spawn_command_tui(tv_local_config_and_cable_with_args(&["dirs"]));

    // open remote control mode
    tester.write_input(&ctrl('t'));
    tester.write_input("files");
    tester.write_input(ENTER);

    tester.assert_tui_output_contains("── files ──");

    // exit remote then app
    tester.write_input(&ctrl('c'));
    tester.write_input(&ctrl('c'));

    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}
