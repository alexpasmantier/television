mod common;

use common::*;

#[test]
fn toggle_help() {
    let mut tester = PtyTester::new();
    let mut child =
        tester.spawn_command_tui(tv_local_config_and_cable_with_args(&[]));

    tester.send(&ctrl('g'));

    tester.assert_tui_output_contains("current mode:");

    // Exit the application
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}

#[test]
// FIXME: was lazy, this should be more robust
fn toggle_preview() {
    let mut tester = PtyTester::new();
    let mut child =
        tester.spawn_command_tui(tv_local_config_and_cable_with_args(&[]));

    let with_preview =
        "╭───────────────────────── files ──────────────────────────╮";
    tester.assert_tui_output_contains(with_preview);

    // Toggle preview
    tester.send(&ctrl('o'));

    let without_preview = "╭─────────────────────────────────────────────────────── files ────────────────────────────────────────────────────────╮";
    tester.assert_tui_output_contains(without_preview);

    // Toggle preview
    tester.send(&ctrl('o'));

    tester.assert_tui_output_contains(with_preview);

    // Exit the application
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}
