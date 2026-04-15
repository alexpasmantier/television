//! Tests for CLI operating modes, path detection, and mode switching.
//!
//! These tests verify Television's two primary operating modes (Channel Mode and Ad-hoc Mode)
//! and the intelligent path detection logic that automatically switches between them.
//! This is fundamental to how Television interprets CLI arguments.

use super::super::common::*;

/// Tests that basic Channel Mode activation works with a channel name.
#[test]
fn test_channel_mode_with_channel_name() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(&pt, &["dirs"])
        .start()
        .unwrap();

    s.wait()
        .text("── dirs ──")
        .text("CHANNEL  dirs")
        .until()
        .unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that Channel Mode works with both channel name and working directory specified.
#[test]
fn test_channel_mode_with_channel_and_path() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(&pt, &["dirs", "./cable/"])
        .start()
        .unwrap();

    s.wait()
        .text("── dirs ──")
        .text("CHANNEL  dirs")
        .text("unix/")
        .until()
        .unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that CLI flags can override channel defaults in Channel Mode.
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

/// Tests that basic Ad-hoc Mode activation works with --source-command.
#[test]
fn test_adhoc_mode_with_source_command() {
    let pt = phantom();

    let s =
        tv_local_config_and_cable_with_args(&pt, &["--source-command", "ls"])
            .start()
            .unwrap();

    s.wait().text("CHANNEL  Custom").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that Ad-hoc Mode requires --source-command for dependent flags.
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

/// Tests that smart path detection automatically switches to Channel Mode.
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
        .text("CHANNEL  files")
        .text("unix/files.toml")
        .until()
        .unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that fallback to default channel works when no arguments are provided.
#[test]
fn test_no_arguments_uses_default_channel() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(&pt, &[])
        .start()
        .unwrap();

    s.wait().text("CHANNEL  files").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}
