mod common;

use common::*;

#[test]
fn custom_input_header_and_preview_size() {
    let mut tester = PtyTester::new();
    let mut cmd = tv_local_config_and_cable_with_args(&["files"]);
    cmd.args(["--input-header", "toasted bagels"]);
    let mut child = tester.spawn_command_tui(cmd);

    tester.assert_tui_frame_contains("── toasted bagels ──");

    tester.send(&ctrl('c'));

    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

#[test]
fn no_help() {
    let mut tester = PtyTester::new();
    let cmd = tv_local_config_and_cable_with_args(&["--no-help"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Check that the help panel is not shown
    tester.assert_not_tui_frame_contains("current mode:");

    // Exit the application
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

#[test]
fn no_preview() {
    let mut tester = PtyTester::new();
    let cmd = tv_local_config_and_cable_with_args(&["--no-preview"]);
    let mut child = tester.spawn_command_tui(cmd);

    // Check that the preview panel is not shown
    // FIXME: this is a bit lazy, should be more robust
    tester.assert_tui_frame_contains("╭─────────────────────────────────────────────────────── files ────────────────────────────────────────────────────────╮");

    // Exit the application
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

#[test]
fn no_remote() {
    let mut tester = PtyTester::new();
    let cmd = tv_local_config_and_cable_with_args(&["--no-remote"]);
    let mut child = tester.spawn_command_tui(cmd);

    tester.send(&ctrl('t'));
    // Check that the remote control is not shown
    tester.assert_not_tui_frame_contains("(1) (2) (3)");

    // Exit the application
    tester.send(&ctrl('c'));
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
    tester.send(ESC);
    tester.assert_tui_running(&mut child);

    tester.send(&ctrl('c'));
    tester.assert_tui_running(&mut child);

    tester.send("a");

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

    tester.send(ESC);
    tester.assert_tui_running(&mut child);

    tester.send(&ctrl('t'));
    tester.assert_tui_frame_contains("(1) (2) (3)");

    tester.send("a");
    tester.send("a");
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

#[test]
fn watch() {
    let mut tester = PtyTester::new();
    let mut cmd =
        tv_local_config_and_cable_with_args(&["--watch", "0.5", "files"]);
    // launch tv in a temporary directory
    let tmp_dir = std::env::temp_dir();
    cmd.args([tmp_dir.to_str().unwrap()]);
    // write a couple of empty files to the temporary directory
    std::fs::write(tmp_dir.join("file1.txt"), "").unwrap();
    std::fs::write(tmp_dir.join("file2.txt"), "").unwrap();

    let mut child = tester.spawn_command_tui(cmd);

    // Check that the files channel is shown and is populated
    tester.assert_tui_frame_contains("file1.txt");

    std::fs::write(tmp_dir.join("file3.txt"), "").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(600));

    // Check that the new file is shown in the UI
    tester.assert_tui_frame_contains("file3.txt");

    // Exit the application
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}
