//! Subcommands: `--version`, `list-channels`, `init`.

use std::{
    io,
    process::{Command, Stdio},
};

use crate::common::*;

/// Really just a sanity check
#[test]
fn test_version() {
    let pt = phantom();
    let s = tv_with_args(&pt, &["--version"]).start().unwrap();

    s.wait().text("television").until().unwrap();
    s.wait().exit_code(0).until().unwrap();
}

/// Tests the `tv list-channels` command.
///
/// We expect this to list all available channels in the cable directory.
#[test]
fn test_list_channels() {
    let pt = phantom();
    let s = tv_local_config_and_cable_with_args(&pt, &["list-channels"])
        .start()
        .unwrap();

    // Check what's in the cable directory
    let cable_dir_filenames = std::fs::read_dir(DEFAULT_CABLE_DIR)
        .expect("Failed to read cable directory")
        .filter_map(Result::ok)
        .filter_map(|entry| {
            // this is pretty lazy and can be improved later on
            entry.path().extension().and_then(|ext| {
                if ext == "toml" {
                    entry
                        .path()
                        .file_stem()
                        .and_then(|stem| stem.to_str().map(String::from))
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    // Checking for the last channel alphabetically ensures the entire list
    // has been printed.
    s.wait()
        .text(cable_dir_filenames.iter().max().unwrap())
        .timeout_ms(wait_timeout_ms())
        .until()
        .unwrap();

    s.wait().exit_code(0).until().unwrap();
}

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
