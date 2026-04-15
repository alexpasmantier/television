//! Tests for CLI preview options: --preview-*, --no-preview, and their combinations.
//!
//! These tests verify Television's preview panel functionality, ensuring users can
//! customize preview behavior and that conflicting options are properly detected.
//! Preview features are essential for examining file contents and command outputs.

use super::super::common::*;

/// Tests that --preview-command works in Ad-hoc Mode.
#[test]
fn test_preview_command_in_adhoc_mode() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--source-command", "ls", "--preview-command", "cat {}"],
    )
    .start()
    .unwrap();

    s.wait()
        .text("╭───────────────────── Custom Channel ─────────────────────╮╭─")
        .until()
        .unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --preview-command can override channel defaults in Channel Mode.
#[test]
fn test_preview_command_override_in_channel_mode() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--preview-command", "cat {}"],
    )
    .start()
    .unwrap();

    s.wait()
        .text("╭───────────────────────── files ──────────────────────────╮╭─")
        .until()
        .unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --preview-header displays custom text above the preview panel.
#[test]
fn test_preview_header_with_preview_command() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--preview-header", "UNIQUE16CHARID"],
    )
    .start()
    .unwrap();

    s.wait().text("UNIQUE16CHARID").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --preview-footer displays custom text below the preview panel.
#[test]
fn test_preview_footer_with_preview_command() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--preview-footer", "UNIQUE16CHARID"],
    )
    .start()
    .unwrap();

    s.wait().text("UNIQUE16CHARID").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --preview-offset controls the scroll position in preview content.
#[test]
fn test_preview_offset_with_preview_command() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "files",
            "--input",
            "CODE_OF_CONDUCT.md",
            "-p",
            "cat -n {}",
            "--preview-offset",
            "50",
        ],
    )
    .start()
    .unwrap();

    s.wait().text("││     50").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --preview-size controls the width of the preview panel.
#[test]
fn test_preview_size_with_preview_command() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--preview-size", "60"],
    )
    .start()
    .unwrap();

    s.wait()
        .text("╭─────────────────── files ────────────────────╮")
        .until()
        .unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --preview-word-wrap enables preview panel word wrapping.
#[test]
fn test_preview_word_wrap_with_preview_command() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "files",
            "--preview-word-wrap",
            "-p",
            "echo 'Hello world'",
            "--preview-size",
            "10",
        ],
    )
    .start()
    .unwrap();

    s.wait().text("│ Hello    │").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --no-preview completely disables the preview panel.
#[test]
fn test_no_preview_disables_preview_panel() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--source-command", "ls", "--no-preview"],
    )
    .start()
    .unwrap();

    s.wait().text("CHANNEL  Custom").until().unwrap();
    assert_frame_not_contains(&s, "─ Preview ─");

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --no-preview conflicts with --preview-command.
#[test]
fn test_no_preview_conflicts_with_preview_command() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--no-preview", "--preview-command", "cat {}"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

/// Tests that --no-preview conflicts with --preview-header.
#[test]
fn test_no_preview_conflicts_with_preview_header() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "--source-command",
            "ls",
            "--no-preview",
            "--preview-header",
            "UNIQUE16CHARID",
        ],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

/// Tests that --no-preview conflicts with --preview-footer.
#[test]
fn test_no_preview_conflicts_with_preview_footer() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "--source-command",
            "ls",
            "--no-preview",
            "--preview-footer",
            "UNIQUE16CHARID",
        ],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

/// Tests that --no-preview conflicts with --preview-offset.
#[test]
fn test_no_preview_conflicts_with_preview_offset() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "--source-command",
            "ls",
            "--no-preview",
            "--preview-offset",
            "10",
        ],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

/// Tests that --no-preview conflicts with --preview-size.
#[test]
fn test_no_preview_conflicts_with_preview_size() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "--source-command",
            "ls",
            "--no-preview",
            "--preview-size",
            "60",
        ],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

/// Tests that preview flags require --preview-command in Ad-hoc Mode.
#[test]
fn test_preview_flags_without_preview_command_errors_in_adhoc_mode() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--source-command", "ls", "--preview-header", "HEADER"],
    )
    .start()
    .unwrap();

    s.wait()
        .text("preview-header requires a preview command")
        .until()
        .unwrap();
}

/// Tests that --hide-preview starts the interface with the preview panel hidden.
#[test]
fn test_hide_preview_flag_starts_with_preview_hidden() {
    let pt = phantom();

    let s =
        tv_local_config_and_cable_with_args(&pt, &["files", "--hide-preview"])
            .start()
            .unwrap();

    s.wait()
        .text("╭─────────────────────────────────────────────────────── files ────────────────────────────────────────────────────────╮")
        .until()
        .unwrap();
    assert_frame_not_contains(&s, "───╮╭───");

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --show-preview starts the interface with the preview panel visible.
#[test]
fn test_show_preview_flag_starts_with_preview_visible() {
    let pt = phantom();

    let s =
        tv_local_config_and_cable_with_args(&pt, &["files", "--show-preview"])
            .start()
            .unwrap();

    s.wait().text("───╮╭───").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --hide-preview conflicts with --no-preview.
#[test]
fn test_hide_preview_conflicts_with_no_preview() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--source-command", "ls", "--hide-preview", "--no-preview"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}
