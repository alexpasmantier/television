//! Tests for CLI special modes: --autocomplete-prompt and related conflicts.
//!
//! These tests verify Television's intelligent channel detection and shell integration
//! features, ensuring that the autocomplete prompt can automatically select appropriate
//! channels based on command analysis.

use std::{
    io,
    process::{Command, Stdio},
};

use super::super::common::*;

/// Tests that --autocomplete-prompt automatically detects and activates the appropriate channel.
#[test]
fn test_autocomplete_prompt_activates_channel_mode() {
    let mut tester = PtyTester::new();

    // This should analyze the git command and automatically select the git-log channel
    let cmd = tv_local_config_and_cable_with_args(&[
        "--autocomplete-prompt",
        "git log --oneline",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Verify the git-log channel was automatically detected and activated
    tester.assert_tui_frame_contains("CHANNEL  git-log");

    // Send Ctrl+C to exit
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that --autocomplete-prompt conflicts with explicit channel argument.
#[test]
fn test_autocomplete_prompt_and_channel_argument_conflict_errors() {
    let mut tester = PtyTester::new();

    // This should fail because both specify how to choose a channel
    let cmd = tv_local_config_and_cable_with_args(&[
        "files",
        "--autocomplete-prompt",
        "git log --oneline",
    ]);
    tester.spawn_command(cmd);

    // CLI should exit with error message, not show TUI
    tester.assert_raw_output_contains("cannot be used with");
}

/// Tests that --autocomplete-prompt works with a working directory path argument.
#[test]
fn test_autocomplete_prompt_with_working_directory() {
    let mut tester = PtyTester::new();

    // This should work: --autocomplete-prompt with a path argument
    let cmd = tv_local_config_and_cable_with_args(&[
        "--autocomplete-prompt",
        "ls",
        "/etc",
    ]);
    let mut child = tester.spawn_command_tui(cmd);

    // Send Ctrl+C to exit (the test is mainly to ensure no CLI parsing error)
    tester.send(&ctrl('c'));
    PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
}

/// Tests that the `list-channels` subcommand lists available channels.
#[test]
fn test_list_channels_subcommand_lists_channels() {
    let mut tester = PtyTester::new();

    // This should show the channel list
    let cmd = tv_local_config_and_cable_with_args(&["list-channels"]);
    tester.spawn_command(cmd);

    // CLI should exit with channel list
    tester.assert_raw_output_contains("files");
}

/// Tests that the `init` subcommand generates a completion script for zsh.
#[test]
fn test_init_subcommand_generates_completion_script() {
    let mut tester = PtyTester::new();

    // This should generate a completion script for zsh
    let cmd = tv_local_config_and_cable_with_args(&["init", "zsh"]);
    tester.spawn_command(cmd);

    // CLI should exit with completion script for zsh
    tester.assert_raw_output_contains("__tv_path_completion");
}

/// Tests that the `init` subcommand rejects unsupported shells.
#[test]
fn test_init_subcommand_invalid_shell_errors() {
    let mut tester = PtyTester::new();

    // This should fail because the shell is not supported
    let cmd = tv_local_config_and_cable_with_args(&["init", "bogus"]);
    tester.spawn_command(cmd);

    // CLI should exit with error message, not show TUI
    tester.assert_raw_output_contains("invalid value");
}

#[test]
fn test_tv_pipes_correctly() -> io::Result<()> {
    if is_ci() {
        // Skip this test in CI environments (where we don't have a proper TTY
        // and subprocess handling with portable-pty isn't available yet).
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

    // Pipe tv stdout into cat stdin
    let mut cat = Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // Write tv output to cat stdin in a separate thread to avoid deadlock
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

    let subprocess_output = cat.wait_with_output()?; // Waits for cat to finish

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
