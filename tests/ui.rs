mod common;

use common::*;

#[test]
// FIXME: was lazy, this should be more robust
fn toggle_preview() {
    let mut tester = PtyTester::new();
    let mut child =
        tester.spawn_command_tui(tv_local_config_and_cable_with_args(&[
            "files",
            "-p",
            "cat -n {}",
        ]));

    let with_preview =
        "╭───────────────────────── files ──────────────────────────╮";
    tester.assert_tui_frame_contains(with_preview);

    // Toggle preview
    tester.send(&ctrl('o'));

    let without_preview = "╭─────────────────────────────────────────────────────── files ────────────────────────────────────────────────────────╮";
    tester.assert_tui_frame_contains(without_preview);

    // Toggle preview
    tester.send(&ctrl('o'));

    tester.assert_tui_frame_contains(with_preview);

    // Exit the application
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
}
