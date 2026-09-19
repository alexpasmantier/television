//! Opening channels, channel shortcuts, source command options, reloading and cycling sources.

use tempfile::TempDir;

use crate::common::*;

#[test]
fn test_ctrl_c() {
    let pt = phantom();
    let s = tv_local_config_and_cable_with_args(&pt, &["files"])
        .start()
        .unwrap();

    s.wait().text("● files").until().unwrap();
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
                .text(&format!("● {}", $channel_name))
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

    s.wait().text("● files").until().unwrap();

    // switch to the "dirs" channel
    s.send().key("f2").unwrap();
    s.wait().text("● dirs").until().unwrap();

    // switch back to the "files" channel
    s.send().key("f1").unwrap();
    s.wait().text("● files").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_source_command_in_adhoc_mode() {
    let pt = phantom();

    let s =
        tv_local_config_and_cable_with_args(&pt, &["--source-command", "ls"])
            .start()
            .unwrap();

    s.wait().text("● Custom").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_source_command_override_in_channel_mode() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "files",
            "--source-command",
            "fd -t f . ./cable/",
            "--input",
            "files.toml",
        ],
    )
    .start()
    .unwrap();

    s.wait().text("./cable/unix/files.toml").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_source_display_with_source_command() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "files",
            "--input",
            "television",
            "--source-display",
            "{upper}",
        ],
    )
    .start()
    .unwrap();

    s.wait().text("TELEVISION").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_source_output_with_source_command() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "--source-command",
            "echo UNIQUE16CHARID",
            "--source-output",
            "echo AA{}BB",
            "--take-1",
        ],
    )
    .start()
    .unwrap();

    s.wait()
        .text("AAUNIQUE16CHARIDBB")
        .timeout_ms(wait_timeout_ms())
        .until()
        .unwrap();
}

#[test]
fn test_source_display_without_source_command_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--source-display", "{upper}"],
    )
    .start()
    .unwrap();

    s.wait()
        .text("source-display requires a source command")
        .until()
        .unwrap();
}

#[test]
fn test_source_output_without_source_command_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--source-output", "echo {}"],
    )
    .start()
    .unwrap();

    s.wait()
        .text("source-output requires a source command")
        .until()
        .unwrap();
}

#[test]
fn test_reload_source_keybinding() {
    let pt = phantom();
    let tmp_dir = TempDir::new().unwrap();

    // Create initial file to be detected
    std::fs::write(tmp_dir.path().join("UNIQUE16CHARIDfile.txt"), "").unwrap();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "files",
            "--input",
            "UNIQUE16CHARID",
            tmp_dir.path().to_str().unwrap(),
        ],
    )
    .start()
    .unwrap();

    s.wait().text("UNIQUE16CHARIDfile.txt").until().unwrap();

    // add another file to be detected
    std::fs::write(tmp_dir.path().join("UNIQUE16CHARIDcontrol.txt"), "")
        .unwrap();

    // Send Ctrl+R to reload the source command
    s.send().key("ctrl-r").unwrap();

    // Verify the new file appears in the TUI as well as the existing one
    s.wait()
        .text("UNIQUE16CHARIDcontrol.txt")
        .text("UNIQUE16CHARIDfile.txt")
        .until()
        .unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_cycle_sources_keybinding() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(&pt, &["files"])
        .start()
        .unwrap();
    s.wait().text("● files").until().unwrap();

    // Send Ctrl+S to cycle to next source
    s.send().key("ctrl-s").unwrap();
    s.send().type_text(".config").unwrap();

    s.wait().text(".config/config.toml").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}
