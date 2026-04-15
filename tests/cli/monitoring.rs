//! Tests for CLI monitoring options: --watch.
//!
//! These tests verify Television's live monitoring capabilities,
//! ensuring users can enable real-time data updates.

use tempfile::TempDir;

use super::super::common::*;

/// Tests that --watch enables live monitoring with automatic source command re-execution.
#[test]
fn test_watch_reloads_source_command() {
    let pt = phantom();
    let tmp_dir = TempDir::new().unwrap();

    // Create initial file to be detected
    std::fs::write(tmp_dir.path().join("UNIQUE16CHARIDfile.txt"), "").unwrap();

    // This monitors the temp directory and updates every 0.5 seconds
    let s = tv_local_config_and_cable_with_args(
        &pt,
        &[
            "--watch",
            "0.5",
            "--source-command",
            "ls",
            "--input",
            "UNIQUE16CHARID",
            tmp_dir.path().to_str().unwrap(),
        ],
    )
    .start()
    .unwrap();

    s.wait().text("UNIQUE16CHARIDfile.txt").until().unwrap();

    // Create a second file
    std::fs::write(tmp_dir.path().join("UNIQUE16CHARIDcontrol.txt"), "")
        .unwrap();

    // The watch should pick up the new file on its next tick
    s.wait()
        .text("UNIQUE16CHARIDcontrol.txt")
        .timeout_ms(3000)
        .until()
        .unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --tick-rate accepts a valid positive number.
#[test]
fn test_tick_rate_valid_value_starts_application() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--tick-rate", "50"],
    )
    .start()
    .unwrap();

    s.wait().text("CHANNEL  files").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --tick-rate rejects non-positive numbers.
#[test]
fn test_tick_rate_invalid_value_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--tick-rate", "-5"],
    )
    .start()
    .unwrap();

    s.wait().text("unexpected argument").until().unwrap();
}
