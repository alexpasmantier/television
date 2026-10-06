//! Subcommands: `--version`, `list-channels`, `init`, `cable`.

use std::{
    fs, io,
    process::{Command, Stdio},
};

use tempfile::{TempDir, tempdir};

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

const REMOTE_FOO: &str = r#"[metadata]
name = "foo"
description = """Foo channel

Longer notes that `list` leaves out."""

[source]
command = "echo foo"
"#;

const REMOTE_BAR: &str = r#"[metadata]
name = "bar"
requirements = ["tv-missing-binary"]

[source]
command = "echo bar"
"#;

/// A data directory with the remote channel cache already filled, so that
/// `tv cable` doesn't hit the network.
fn data_dir_with_remote_cache() -> TempDir {
    let dir = tempdir().unwrap();
    let cache = dir
        .path()
        .join("gh-cable-cache")
        .join(env!("CARGO_PKG_VERSION"));
    fs::create_dir_all(&cache).unwrap();
    fs::write(cache.join("foo.toml"), REMOTE_FOO).unwrap();
    fs::write(cache.join("bar.toml"), REMOTE_BAR).unwrap();
    dir
}

fn tv_cable(data_dir: &TempDir, config: &TempConfig, args: &[&str]) -> String {
    let output = Command::new(TV_BIN_PATH)
        .env("TELEVISION_DATA", data_dir.path())
        .arg("--config-file")
        .arg(&config.config_file)
        .arg("--cable-dir")
        .arg(&config.cable_dir)
        .arg("cable")
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "tv cable {args:?} failed: {output:?}"
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn test_cable_list_shows_status() {
    let data_dir = data_dir_with_remote_cache();
    let config = TempConfig::init();
    config
        .write_channel("foo", &format!("{REMOTE_FOO}# edited\n"))
        .unwrap();

    assert_eq!(
        tv_cable(&data_dir, &config, &["list"]),
        "bar  available  (needs tv-missing-binary)\nfoo  modified   Foo channel\n"
    );
    assert_eq!(
        tv_cable(&data_dir, &config, &["list", "--modified", "--installed"]),
        "foo  modified  Foo channel\n"
    );
}

#[test]
fn test_cable_install_and_remove() {
    let data_dir = data_dir_with_remote_cache();
    let config = TempConfig::init();
    let foo = config.cable_dir.join("foo.toml");
    let bar = config.cable_dir.join("bar.toml");
    config.write_channel("foo", REMOTE_FOO).unwrap();

    // `--all` skips bar (missing requirements)
    tv_cable(&data_dir, &config, &["install", "--all"]);
    assert_eq!(fs::read_to_string(&foo).unwrap(), REMOTE_FOO);
    assert!(!bar.exists());

    // naming bar installs it regardless of requirements
    tv_cable(&data_dir, &config, &["install", "bar"]);
    assert_eq!(fs::read_to_string(&bar).unwrap(), REMOTE_BAR);

    tv_cable(&data_dir, &config, &["remove", "foo", "bar"]);
    assert!(!foo.exists() && !bar.exists());
}
