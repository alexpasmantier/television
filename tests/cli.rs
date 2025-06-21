mod common;

use common::*;

#[test]
fn custom_input_header_and_preview_size() {
    let mut tester = PtyTester::new();
    let mut cmd = tv_local_config_and_cable_with_args(&["files"]);
    cmd.args(["--input-header", "toasted bagels"]);
    let mut child = tester.spawn_command_tui(cmd);

    tester.assert_tui_output_contains("── toasted bagels ──");

    tester.write_input(&ctrl('c'));

    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

#[test]
fn no_help() {
    let mut tester = PtyTester::new();
    let cmd = tv_local_config_and_cable_with_args(&["--no-help"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Check that the help panel is not shown
    tester.assert_not_tui_output_contains("current mode:");

    // Exit the application
    tester.write_input(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

#[test]
fn no_preview() {
    let mut tester = PtyTester::new();
    let cmd = tv_local_config_and_cable_with_args(&["--no-preview"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Check that the preview panel is not shown
    // FIXME: this is a bit lazy, should be more robust
    tester.assert_tui_output_contains("╭─────────────────────────────────────────────────────── files ────────────────────────────────────────────────────────╮");

    // Exit the application
    tester.write_input(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

#[test]
fn no_remote() {
    let mut tester = PtyTester::new();
    let cmd = tv_local_config_and_cable_with_args(&["--no-remote"]);
    let mut child = tester.spawn_command_tui(cmd);

    tester.write_input(&ctrl('t'));
    // Check that the remote control is not shown
    tester.assert_not_tui_output_contains("──Remote Control──");

    // Exit the application
    tester.write_input(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

#[test]
fn keybindings() {
    let mut tester = PtyTester::new();
    let mut child =
        tester.spawn_command_tui(tv_local_config_and_cable_with_args(&[
            "--keybindings",
            "quit=\"a\"",
        ]));

    // test that the default keybindings are overridden
    tester.write_input(ESC);
    tester.assert_tui_running(&mut child);

    tester.write_input(&ctrl('c'));
    tester.assert_tui_running(&mut child);

    tester.write_input("a");

    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

#[test]
fn multiple_keybindings() {
    let mut tester = PtyTester::new();
    let mut child =
        tester.spawn_command_tui(tv_local_config_and_cable_with_args(&[
            "--keybindings",
            "quit=\"a\";toggle_remote_control=\"ctrl-t\"",
        ]));

    tester.write_input(ESC);
    tester.assert_tui_running(&mut child);

    tester.write_input(&ctrl('t'));
    tester.assert_tui_output_contains("──Remote Control──");

    tester.write_input("a");
    tester.write_input("a");
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}
