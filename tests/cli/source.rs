//! Tests for CLI source/data options: --source-command, --source-display, --source-output.
//!
//! These tests verify Television's source command functionality, which is fundamental to
//! generating and formatting the data that users interact with. Source commands define
//! what entries are available for selection and how they're processed.

use super::super::common::*;

/// Tests that --source-command works properly in Ad-hoc Mode.
#[test]
fn test_source_command_in_adhoc_mode() {
    let pt = phantom();

    let s =
        tv_local_config_and_cable_with_args(&pt, &["--source-command", "ls"])
            .start()
            .unwrap();

    s.wait().text("CHANNEL  Custom").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --source-command can override channel defaults in Channel Mode.
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

/// Tests that --source-display formats how entries appear in the results list.
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

/// Tests that --source-output formats the final output when an entry is selected.
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
        .timeout_ms(2000)
        .until()
        .unwrap();
}

/// Tests that --source-display requires --source-command in Ad-hoc Mode.
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

/// Tests that --source-output requires --source-command in Ad-hoc Mode.
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
