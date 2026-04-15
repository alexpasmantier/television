//! Tests for CLI selection behavior: --select-1, --take-1, --take-1-fast, and their conflicts.
//!
//! These tests verify Television's automatic selection behaviors that allow scripts and
//! automated workflows to get results without user interaction. They also ensure that
//! conflicting selection modes are properly detected and rejected.

use super::super::common::*;

/// Tests that --select-1 automatically selects and returns when only one entry matches.
#[test]
fn test_select_1_auto_selects_single_entry() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--source-command", "echo UNIQUE16CHARID", "--select-1"],
    )
    .start()
    .unwrap();

    let output = exit_and_output(&s);
    assert!(
        output.contains("UNIQUE16CHARID"),
        "expected output to contain 'UNIQUE16CHARID', got:\n{output}"
    );
}

/// Tests that --select-1 respects the initial --input filter.
#[test]
fn test_select_1_respects_initial_input() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "--source-command",
            "printf 'television\\ntelescope\\n'",
            "--select-1",
            "--input",
            "telev",
        ],
    )
    .start()
    .unwrap();

    let output = exit_and_output(&s);
    assert!(
        output.contains("television"),
        "expected output to contain 'television', got:\n{output}"
    );
}

/// Tests that --take-1 automatically selects the first entry after loading completes.
#[test]
fn test_take_1_auto_selects_first_entry() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--source-command", "echo UNIQUE16CHARID", "--take-1"],
    )
    .start()
    .unwrap();

    let output = exit_and_output(&s);
    assert!(
        output.contains("UNIQUE16CHARID"),
        "expected output to contain 'UNIQUE16CHARID', got:\n{output}"
    );
}

/// Tests that --take-1-fast immediately selects the first entry as it appears.
#[test]
fn test_take_1_fast_auto_selects_first_entry_immediately() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--source-command", "echo UNIQUE16CHARID", "--take-1-fast"],
    )
    .start()
    .unwrap();

    let output = exit_and_output(&s);
    assert!(
        output.contains("UNIQUE16CHARID"),
        "expected output to contain 'UNIQUE16CHARID', got:\n{output}"
    );
}

/// Tests that `--take-1` can return before source completion when entries are flushed
/// periodically.
#[test]
fn test_take_1_fast_flushes_before_source_completion() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "--source-command",
            "sleep 0.2 ; echo UNIQUE16CHARID_FIRST ; sleep 4 ; echo UNIQUE16CHARID_SECOND",
            "--take-1",
        ],
    )
    .start()
    .unwrap();

    // Should exit (and so output the first entry) before the source command
    // finishes its 4s sleep.
    let output = exit_and_output(&s);
    assert!(
        output.contains("UNIQUE16CHARID_FIRST"),
        "expected output to contain 'UNIQUE16CHARID_FIRST', got:\n{output}"
    );
}

/// Tests that --select-1 and --take-1 cannot be used together.
#[test]
fn test_select_1_and_take_1_conflict_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--select-1", "--take-1"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

/// Tests that --select-1 and --take-1-fast cannot be used together.
#[test]
fn test_select_1_and_take_1_fast_conflict_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--select-1", "--take-1-fast"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

/// Tests that --take-1 and --take-1-fast cannot be used together.
#[test]
fn test_take_1_and_take_1_fast_conflict_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--take-1", "--take-1-fast"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

/// Tests that --watch and --select-1 cannot be used together.
#[test]
fn test_watch_and_select_1_conflict_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--watch", "1.0", "--select-1"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

/// Tests that --watch and --take-1 cannot be used together.
#[test]
fn test_watch_and_take_1_conflict_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--watch", "1.0", "--take-1"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

/// Tests that --watch and --take-1-fast cannot be used together.
#[test]
fn test_watch_and_take_1_fast_conflict_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--watch", "1.0", "--take-1-fast"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

/// Tests that --expect works as intended.
#[test]
fn test_expect_with_selection() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--expect", "ctrl-c", "--input", "Cargo.toml"],
    )
    .start()
    .unwrap();

    // Wait until the results list has actually populated with Cargo.toml
    // as the single match. Matching "Cargo.toml" alone would spuriously
    // succeed on the `> Cargo.toml` input prompt before fd has produced
    // any entries, then ctrl-c would fire against an empty selection and
    // --expect would print nothing useful.
    s.wait()
        .text("1 / 1")
        .text("Cargo.toml")
        .timeout_ms(wait_timeout_ms())
        .until()
        .unwrap();

    s.send().key("ctrl-c").unwrap();

    // After ctrl-c, tv exits and prints the pressed key plus the selection
    // on the primary screen.
    let output = exit_and_output(&s);
    assert!(
        output.contains("ctrl-c") && output.contains("Cargo.toml"),
        "expected output to contain 'ctrl-c' and 'Cargo.toml', got:\n{output}"
    );
}
