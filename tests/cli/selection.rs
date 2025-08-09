//! Tests for CLI selection behavior: --select-1, --take-1, --take-1-fast, and their conflicts.
//!
//! These tests verify Television's automatic selection behaviors that allow scripts and
//! automated workflows to get results without user interaction. They also ensure that
//! conflicting selection modes are properly detected and rejected.

use super::super::common::*;

/// Tests that --select-1 automatically selects and returns when only one entry matches.
#[test]
fn test_select_1_auto_selects_single_entry() {
    let mut tester = PtyTester::new();

    // This should auto-select "UNIQUE16CHARID" since it's the only result
    let cmd = tv_local_config_and_cable_with_args(&[
        "--source-command",
        "echo UNIQUE16CHARID",
        "--select-1",
    ]);
    tester.spawn_command(cmd);

    // Should auto-select and output the result without showing TUI
    tester.assert_raw_output_contains("UNIQUE16CHARID");
}

/// Tests that --take-1 automatically selects the first entry after loading completes.
#[test]
fn test_take_1_auto_selects_first_entry() {
    let mut tester = PtyTester::new();

    // This should auto-select "UNIQUE16CHARID" since it's the only result
    let cmd = tv_local_config_and_cable_with_args(&[
        "--source-command",
        "echo UNIQUE16CHARID",
        "--take-1",
    ]);
    tester.spawn_command(cmd);

    // Should auto-select and output the result without showing TUI
    tester.assert_raw_output_contains("UNIQUE16CHARID");
}

/// Tests that --take-1-fast immediately selects the first entry as it appears.
#[test]
fn test_take_1_fast_auto_selects_first_entry_immediately() {
    let mut tester = PtyTester::new();

    // This should auto-select "UNIQUE16CHARID" since it's the only result
    let cmd = tv_local_config_and_cable_with_args(&[
        "--source-command",
        "echo UNIQUE16CHARID",
        "--take-1-fast",
    ]);
    tester.spawn_command(cmd);

    // Should auto-select and output the result without showing TUI
    tester.assert_raw_output_contains("UNIQUE16CHARID");
}

/// Tests that --select-1 and --take-1 cannot be used together.
#[test]
fn test_select_1_and_take_1_conflict_errors() {
    let mut tester = PtyTester::new();

    // This should fail because both flags specify different selection behaviors
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--select-1",
        "--take-1",
    ]);
    tester.spawn_command(cmd);

    // CLI should exit with error message
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that --select-1 and --take-1-fast cannot be used together.
#[test]
fn test_select_1_and_take_1_fast_conflict_errors() {
    let mut tester = PtyTester::new();

    // This should fail because --select-1 is conditional while --take-1-fast is immediate
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--select-1",
        "--take-1-fast",
    ]);
    tester.spawn_command(cmd);

    // CLI should exit with error message
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that --take-1 and --take-1-fast cannot be used together.
#[test]
fn test_take_1_and_take_1_fast_conflict_errors() {
    let mut tester = PtyTester::new();

    // This should fail because both specify different timing for first entry selection
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--take-1",
        "--take-1-fast",
    ]);
    tester.spawn_command(cmd);

    // CLI should exit with error message
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that --watch and --select-1 cannot be used together.
#[test]
fn test_watch_and_select_1_conflict_errors() {
    let mut tester = PtyTester::new();

    // This should fail because watch mode updates continuously while --select-1 would exit
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--watch",
        "1.0",
        "--select-1",
    ]);
    tester.spawn_command(cmd);

    // CLI should exit with error message
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that --watch and --take-1 cannot be used together.
#[test]
fn test_watch_and_take_1_conflict_errors() {
    let mut tester = PtyTester::new();

    // This should fail because watch mode conflicts with immediate exit behavior
    let cmd = tv_local_config_and_cable_with_args(&[
        "files", "--watch", "1.0", "--take-1",
    ]);
    tester.spawn_command(cmd);

    // CLI should exit with error message
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that --watch and --take-1-fast cannot be used together.
#[test]
fn test_watch_and_take_1_fast_conflict_errors() {
    let mut tester = PtyTester::new();

    // This should fail because watch mode can't work with immediate exit
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--watch",
        "1.0",
        "--take-1-fast",
    ]);
    tester.spawn_command(cmd);

    // CLI should exit with error message
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that --expect works as intended.
#[test]
fn test_expect_with_selection() {
    let mut tester = PtyTester::new();

    // This should auto-select "UNIQUE16CHARID" and exit with it
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--expect",
        "ctrl-c",
        "--input",
        "Cargo.toml",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    tester.send(&ctrl('c'));

    let out = tester.read_raw_output();

    assert!(
        out.contains("ctrl-c\r\nCargo.toml"),
        "Expected output to contain 'ctrl-c\\r\\nCargo.toml' but got: '{:?}'",
        out
    );

    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}
