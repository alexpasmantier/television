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
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--autocomplete-prompt", "git log --oneline"],
    )
    .start()
    .unwrap();

    s.wait().text("CHANNEL  git-log").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that --autocomplete-prompt conflicts with explicit channel argument.
#[test]
fn test_autocomplete_prompt_and_channel_argument_conflict_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["files", "--autocomplete-prompt", "git log --oneline"],
    )
    .start()
    .unwrap();

    s.wait().text("cannot be used with").until().unwrap();
}

/// Tests that --autocomplete-prompt works with a working directory path argument.
#[test]
fn test_autocomplete_prompt_with_working_directory() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(
        &pt,
        &["--autocomplete-prompt", "ls", "/etc"],
    )
    .start()
    .unwrap();
    // Main assertion: no CLI parsing error — wait for the status bar to
    // appear, which confirms the TUI launched successfully.
    s.wait().text("CHANNEL  ").until().unwrap();

    s.send().key("ctrl-c").unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests that the `list-channels` subcommand lists available channels.
#[test]
fn test_list_channels_subcommand_lists_channels() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(&pt, &["list-channels"])
        .start()
        .unwrap();

    s.wait().text("files").until().unwrap();
}

/// Tests that the `init` subcommand generates a completion script for zsh.
#[test]
fn test_init_subcommand_generates_completion_script() {
    let pt = phantom();

    // The zsh init script is a few hundred lines — make sure the terminal
    // has enough scrollback to preserve the `#compdef tv` marker on line 1.
    let s = tv_local_config_and_cable_with_args(&pt, &["init", "zsh"])
        .scrollback(1000)
        .start()
        .unwrap();

    s.wait().exit_code(0).until().unwrap();

    let scrollback = s.scrollback(None).unwrap();
    assert!(
        scrollback.contains("compdef"),
        "expected scrollback to contain 'compdef', got:\n{scrollback}"
    );
}

/// Tests that the `init` subcommand rejects unsupported shells.
#[test]
fn test_init_subcommand_invalid_shell_errors() {
    let pt = phantom();

    let s = tv_local_config_and_cable_with_args(&pt, &["init", "bogus"])
        .start()
        .unwrap();

    s.wait().text("invalid value").until().unwrap();
}

/// Tests that `tv list-channels` handles broken pipe (EPIPE) gracefully.
///
/// When piping to a command that exits without reading (e.g., `tv list-channels | (exit 0)`),
/// tv should exit cleanly rather than panicking.
#[test]
fn test_list_channels_broken_pipe() -> io::Result<()> {
    let mut child = Command::new(TV_BIN_PATH)
        .args(LOCAL_CONFIG_AND_CABLE)
        .args(["list-channels"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    // Close the read end of the stdout pipe immediately, causing a broken pipe
    // when tv tries to write its output.
    drop(child.stdout.take());

    let status = child.wait()?;
    assert!(
        status.success(),
        "tv list-channels should handle broken pipe gracefully, but exited with: {status:?}",
    );

    Ok(())
}

/// Tests that `tv init <shell>` handles broken pipe (EPIPE) gracefully.
///
/// When piping to a command that exits without reading (e.g., `tv init zsh | (exit 0)`),
/// tv should exit cleanly rather than panicking.
#[test]
fn test_init_shell_broken_pipe() -> io::Result<()> {
    let mut child = Command::new(TV_BIN_PATH)
        .args(LOCAL_CONFIG_AND_CABLE)
        .args(["init", "zsh"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    drop(child.stdout.take());

    let status = child.wait()?;
    assert!(
        status.success(),
        "tv init zsh should handle broken pipe gracefully, but exited with: {status:?}",
    );

    Ok(())
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
