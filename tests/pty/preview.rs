//! The preview panel: `--preview-*` flags, toggling and scrolling.

use crate::common::*;

#[test]
fn test_preview_command_in_adhoc_mode() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--source-command", "ls", "--preview-command", "cat {}"],
    )
    .start()
    .unwrap();

    s.wait().text("● Custom").text("▏").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_preview_command_override_in_channel_mode() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--preview-command", "cat {}"],
    )
    .start()
    .unwrap();

    s.wait().text("● files").text("▏").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

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

    s.wait().text("    50").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_preview_size_with_preview_command() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--preview-size", "60"],
    )
    .start()
    .unwrap();

    s.wait().text("▏").until().unwrap();

    // with a 60% preview, the separator sits well left of the default 50%
    let frame = stable_frame(&s);
    let separator_col = frame
        .lines()
        .find_map(|l| l.chars().position(|c| c == '▏'))
        .expect("Expected a preview separator in the frame");
    assert!(
        separator_col < 55,
        "Expected the separator left of column 55, got {}",
        separator_col
    );

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

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

    s.wait().text("Hello").until().unwrap();
    assert_frame_not_contains(&s, "Hello world");

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_no_preview_disables_preview_panel() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--source-command", "ls", "--no-preview"],
    )
    .start()
    .unwrap();

    s.wait().text("● Custom").until().unwrap();
    assert_frame_not_contains(&s, "─ Preview ─");

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

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

#[test]
fn test_hide_preview_flag_starts_with_preview_hidden() {
    let pt = phantom();

    let s =
        tv_local_config_and_cable_with_args(&pt, &["files", "--hide-preview"])
            .start()
            .unwrap();

    s.wait().text("● files").until().unwrap();
    assert_frame_not_contains(&s, "▏");

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_show_preview_flag_starts_with_preview_visible() {
    let pt = phantom();

    let s =
        tv_local_config_and_cable_with_args(&pt, &["files", "--show-preview"])
            .start()
            .unwrap();

    s.wait().text("▏").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

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

#[test]
fn test_toggle_preview_keybinding() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(&pt, &["files"])
        .start()
        .unwrap();

    // Verify preview is initially visible (two panels side by side)
    s.wait().text("▏").until().unwrap();

    // Send Ctrl+O to toggle preview off
    s.send().key("ctrl-o").unwrap();

    // Verify preview is now hidden
    s.wait().text_absent("▏").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_scroll_preview_keybindings() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--input", "README.md"],
    )
    .start()
    .unwrap();
    s.wait().text("▏    1 ").until().unwrap();

    // Send Page Down to scroll preview down
    s.send().key("pagedown").unwrap();
    s.send().key("pagedown").unwrap();

    s.wait().text_absent("▏    1 ").until().unwrap();

    // Send Page Up to scroll preview up
    s.send().key("pageup").unwrap();
    s.send().key("pageup").unwrap();

    s.wait().text("▏    1 ").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that scrolling the preview shows a dimmed percentage in the title
/// row (standing in for the scrollbar), which disappears when scrolled back.
#[test]
fn test_preview_scroll_percent_hint() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--input", "LICENSE"],
    )
    .start()
    .unwrap();
    s.wait().text("▏    1 ").until().unwrap();
    assert_frame_not_contains(&s, "%");

    s.send().key("pagedown").unwrap();
    s.wait().text("% ").until().unwrap();

    s.send().key("pageup").unwrap();
    s.wait().text_absent("% ").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

#[test]
fn test_hide_preview_scrollbar_hides_scrollbar() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "files",
            "--hide-preview-scrollbar",
            "--preview-border",
            "rounded",
        ],
    )
    .start()
    .unwrap();

    s.wait().text("──╮").until().unwrap();
    assert_frame_not_contains(&s, "▲");

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}
