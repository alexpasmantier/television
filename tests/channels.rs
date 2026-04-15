mod common;

use common::*;

#[test]
fn tv_ctrl_c() {
    let pt = phantom();
    let s = tv_local_config_and_cable_with_args(&pt, &["files"])
        .start()
        .unwrap();

    s.wait().text("── files ──").until().unwrap();
    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Test that the various channels open correctly, spawn a UI that contains the
/// expected channel name, and exit cleanly when Ctrl-C is pressed.
macro_rules! test_channel {
    ($($name:ident: $channel_name:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let pt = phantom();
            let s = tv_local_config_and_cable_with_args(&pt, &[$channel_name])
                .start()
                .unwrap();

            s.wait()
                .text(&format!("── {} ──", $channel_name))
                .until()
                .unwrap();

            s.send().key("ctrl-c").unwrap();
            s.wait().exit_code(0).until().unwrap();
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
    let pt = phantom();
    let s = tv_local_config_and_cable_with_args(&pt, &["files"])
        .start()
        .unwrap();

    s.wait().text("CHANNEL  files").until().unwrap();

    // switch to the "dirs" channel
    s.send().key("f2").unwrap();
    s.wait().text("CHANNEL  dirs").until().unwrap();

    // switch back to the "files" channel
    s.send().key("f1").unwrap();
    s.wait().text("CHANNEL  files").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}
