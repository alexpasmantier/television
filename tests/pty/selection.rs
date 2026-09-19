//! Selection and output: `--select-1`, `--take-1`, `--take-1-fast`, `--expect`, piping.

use std::{
    io,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use crate::common::*;

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
        .text(" 1/")
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

#[test]
fn test_tv_pipes_correctly() -> io::Result<()> {
    if is_ci() {
        dbg!("Skipping test_tv_pipes_correctly in CI environment");
        return Ok(());
    }
    let mut tv_command = Command::new(TV_BIN_PATH)
        .args(LOCAL_CONFIG_AND_CABLE)
        .args(["--input", "Cargo.toml"])
        .arg("--take-1")
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let tv_stdout =
        tv_command.stdout.take().expect("Failed to capture stdout");

    let mut cat = Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut subprocess_stdin = cat
        .stdin
        .take()
        .expect("Failed to capture subprocess stdin");
    std::thread::spawn(move || {
        let _ = io::copy(
            &mut io::BufReader::new(tv_stdout),
            &mut subprocess_stdin,
        );
    });

    // tv occasionally never completes under heavy parallel test load, which
    // would otherwise hang the whole suite: kill it after a generous
    // deadline so the pipe closes and the test fails instead
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut timed_out = false;
    while tv_command.try_wait()?.is_none() {
        if Instant::now() > deadline {
            tv_command.kill()?;
            timed_out = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(!timed_out, "tv did not complete within 30s");

    let subprocess_output = cat.wait_with_output()?;

    assert!(
        subprocess_output.status.success(),
        "cat failed: {}",
        String::from_utf8_lossy(&subprocess_output.stderr)
    );

    let output = String::from_utf8_lossy(&subprocess_output.stdout);
    assert!(!output.trim().is_empty(), "Output should not be empty");
    assert_eq!(
        output.trim(),
        "Cargo.toml",
        "Output should match input file name"
    );

    Ok(())
}
