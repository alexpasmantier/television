//! Channel mode vs ad-hoc mode, path detection, `--autocomplete-prompt`.

use tempfile::TempDir;

use crate::common::*;

#[test]
fn test_channel_mode_with_channel_name() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(&pt, &["dirs"])
        .start()
        .unwrap();

    s.wait().text("● dirs").text("● dirs").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_channel_mode_with_channel_and_path() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(&pt, &["dirs", "./cable/"])
        .start()
        .unwrap();

    s.wait()
        .text("● dirs")
        .text("● dirs")
        .text("unix/")
        .until()
        .unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_channel_mode_with_channel_and_overrides() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--input-header", "UNIQUE16CHARID"],
    )
    .start()
    .unwrap();

    s.wait().text("UNIQUE16CHARID").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_adhoc_mode_with_source_command() {
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
fn test_adhoc_mode_missing_source_command_errors() {
    let pt = phantom();

    let s =
        tv_local_config_and_cable_with_args(&pt, &["--source-display", "{}"])
            .start()
            .unwrap();

    s.wait()
        .text("source-display requires a source command")
        .until()
        .unwrap();
}

#[test]
fn test_smart_path_detection_switches_to_adhoc_mode() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["./cable/", "--input", "files.toml"],
    )
    .start()
    .unwrap();

    s.wait()
        .text("● files")
        .text("unix/files.toml")
        .until()
        .unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_no_arguments_uses_default_channel() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(&pt, &[])
        .start()
        .unwrap();

    s.wait().text("● files").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_path_as_positional_argument_sets_working_directory() {
    let pt = phantom();
    let tmp_dir = TempDir::new().unwrap();

    // Create initial files to be detected
    std::fs::write(tmp_dir.path().join("UNIQUE16CHARIDfile.txt"), "").unwrap();

    // Starts the files channel in the specified temporary directory
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

    s.wait().text("● files").until().unwrap();
    s.wait().text("UNIQUE16CHARIDfile.txt").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_autocomplete_prompt_activates_channel_mode() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--autocomplete-prompt", "git log --oneline"],
    )
    .start()
    .unwrap();

    s.wait().text("● git-log").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_autocomplete_prompt_and_channel_argument_conflict_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--autocomplete-prompt", "git log --oneline"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

#[test]
fn test_autocomplete_prompt_with_working_directory() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--autocomplete-prompt", "ls", "/etc"],
    )
    .start()
    .unwrap();
    // Main assertion: no CLI parsing error — wait for the status bar to
    // appear, which confirms the TUI launched successfully.
    s.wait().text("help ctrl-h").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}
