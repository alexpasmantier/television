mod common;

use common::*;

#[test]
fn tv_ctrl_c() {
    let mut tester = PtyTester::new();
    let mut child = tester
        .spawn_command_tui(tv_local_config_and_cable_with_args(&["files"]));

    tester.send(&ctrl('c'));

    // Check if the child process exited with a timeout
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Test that the various channels open correctly, spawn a UI that contains the
/// expected channel name, and exit cleanly when Ctrl-C is pressed.
macro_rules! test_channel {
    ($($name:ident: $channel_name:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let mut tester = PtyTester::new();
            let mut child = tester.spawn_command_tui(tv_local_config_and_cable_with_args(&[
                $channel_name,
            ]));

            tester.assert_tui_frame_contains(&format!(
                "── {} ──",
                $channel_name
            ));

            tester.send(&ctrl('c'));
            PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
        }
    )*
    }
}

test_channel! {
    test_channel_files: "files",
    test_channel_dirs: "dirs",
    test_channel_env: "env",
    test_channel_git_log: "git-log",
    test_channel_git_reflog: "git-reflog",
    test_channel_git_branch: "git-branch",
    test_channel_text: "text",
    test_channel_diff: "git-diff",
}

#[test]
fn test_channel_shortcuts() {
    let mut tester = PtyTester::new();
    let mut child = tester
        .spawn_command_tui(tv_local_config_and_cable_with_args(&["files"]));

    tester.assert_tui_frame_contains("CHANNEL  files");

    // switch to the "dirs" channel
    tester.send(&f(2));
    tester.assert_tui_frame_contains("CHANNEL  dirs");

    // switch back to the "files" channel
    tester.send(&f(1));
    tester.assert_tui_frame_contains("CHANNEL  files");

    // Send Ctrl-C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}
