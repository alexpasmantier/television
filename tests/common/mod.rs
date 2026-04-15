#![allow(dead_code)]

use std::fs;
use std::io;
use std::path::PathBuf;

use phantom_test::{Phantom, Session, SessionBuilder};
use television::cable::CABLE_DIR_NAME;
use television::config::CONFIG_FILE_NAME;
use tempfile::{TempDir, tempdir};

pub const CI_ENV_VAR: &str = "TV_CI";

pub fn is_ci() -> bool {
    std::env::var(CI_ENV_VAR).is_ok()
}

pub const DEFAULT_CONFIG_FILE: &str = "./.config/config.toml";
#[cfg(unix)]
pub const DEFAULT_CABLE_DIR: &str = "./cable/unix";
#[cfg(windows)]
pub const DEFAULT_CABLE_DIR: &str = "./cable/windows";

pub const TARGET_DIR: &str = "./tests/target_dir";

pub const DEFAULT_COLS: u16 = 120;
pub const DEFAULT_ROWS: u16 = 30;

/// How long to wait for the screen to stabilize before asserting negative
/// conditions. Tunable via `TV_TEST_STABLE_MS`.
pub fn stable_ms() -> u64 {
    std::env::var("TV_TEST_STABLE_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300)
}

/// Timeout used when waiting for a condition (text to appear, exit code,
/// etc.). Sized to comfortably absorb first-frame latency of a tv process
/// under the capped parallelism we run tests at (`--test-threads=4`), with
/// some extra headroom on CI.
///
/// Tunable via `TV_TEST_WAIT_MS`.
pub fn wait_timeout_ms() -> u64 {
    if let Some(v) = std::env::var("TV_TEST_WAIT_MS")
        .ok()
        .and_then(|v| v.parse().ok())
    {
        return v;
    }
    if is_ci() { 15_000 } else { 5_000 }
}

pub const TV_BIN_PATH: &str = match option_env!("TV_BIN_PATH") {
    Some(v) => v,
    None => "./target/debug/tv",
};

pub const LOCAL_CONFIG_AND_CABLE: &[&str] = &[
    "--cable-dir",
    DEFAULT_CABLE_DIR,
    "--config-file",
    DEFAULT_CONFIG_FILE,
];

/// Create a new phantom engine for a test.
///
/// Each test gets its own engine thread, matching phantom-test's recommended
/// idiom (see phantom's own `bash_tests.rs`).
pub fn phantom() -> Phantom {
    Phantom::new().expect("failed to create phantom engine")
}

/// Build a tv session preconfigured with the current working directory and
/// default terminal size. Returns a [`SessionBuilder`] that still needs
/// `.start()` to be called.
pub fn tv(pt: &Phantom) -> SessionBuilder<'_> {
    let cwd = std::env::current_dir().expect("failed to get cwd");
    pt.run(TV_BIN_PATH)
        .size(DEFAULT_COLS, DEFAULT_ROWS)
        .cwd(cwd.to_str().expect("cwd is not valid utf-8"))
}

/// Build a tv session preconfigured with the given arguments.
pub fn tv_with_args<'p>(pt: &'p Phantom, args: &[&str]) -> SessionBuilder<'p> {
    tv(pt).args(args)
}

/// Build a tv session using the repository's local config and cable
/// directory, plus extra arguments.
pub fn tv_local_config_and_cable_with_args<'p>(
    pt: &'p Phantom,
    extra_args: &[&str],
) -> SessionBuilder<'p> {
    let mut combined: Vec<&str> =
        Vec::with_capacity(LOCAL_CONFIG_AND_CABLE.len() + extra_args.len());
    combined.extend_from_slice(LOCAL_CONFIG_AND_CABLE);
    combined.extend_from_slice(extra_args);
    tv(pt).args(&combined)
}

/// Wait for `s` to exit with code 0 and return the primary-screen output
/// (what tv wrote to stdout after leaving alt-screen, e.g. the selection
/// from `--select-1`, `--take-1-fast`, `--expect`, or an external action).
///
/// Prefer this over `wait().text(..)` when the condition you care about is
/// *only visible after exit*. `wait().text()` polls the live screen on a
/// 50ms interval, which for a tv process that prints its output and exits
/// within a few ms is a race — under contention the poller can miss the
/// alt-to-primary-screen transition entirely and then wait out its full
/// timeout on a dead session.
pub fn exit_and_output(s: &Session) -> String {
    s.wait()
        .exit_code(0)
        .timeout_ms(wait_timeout_ms())
        .until()
        .expect("tv did not exit with code 0 within timeout");
    s.output().expect("failed to read tv output")
}

/// Assert that, once the screen has stabilized, it does not contain any of
/// the given texts. Useful when confirming the absence of UI elements after
/// an action that should not cause them to appear.
pub fn assert_frame_not_contains_any(s: &Session, texts: &[&str]) {
    s.wait().stable(stable_ms()).until().unwrap();
    let screen = s.screenshot().unwrap();
    let text = screen.text();
    for &needle in texts {
        assert!(
            !text.contains(needle),
            "expected screen not to contain '{needle}', but got:\n{text}"
        );
    }
}

/// Assert that, once the screen has stabilized, it does not contain the
/// given text.
pub fn assert_frame_not_contains(s: &Session, text: &str) {
    assert_frame_not_contains_any(s, &[text]);
}

/// Return a stabilized screenshot of the current screen as a string.
pub fn stable_frame(s: &Session) -> String {
    s.wait().stable(stable_ms()).until().unwrap();
    s.screenshot().unwrap().text().to_string()
}

/// A temporary configuration directory for tv to use during tests.
///
/// # Example
/// ```rs
/// // initialize a temporary directory with the right structure
/// let temp_config = TempConfig::init();
///
/// let config_content: &str = r#"
/// [ui]
/// theme = "default"
/// "#;
///
/// let channel_content: &str = r#"
/// [metadata]
/// name = "my-channel"
/// description = "..."
/// ...
/// "#;
///
/// temp_config.write_config(config_content)?;
/// temp_config.write_channel("my-channel", channel_content)?;
/// ```
pub struct TempConfig {
    pub config_file: PathBuf,
    pub cable_dir: PathBuf,
    _tempdir: TempDir,
}

impl TempConfig {
    /// Create a temporary configuration directory.
    pub fn init() -> Self {
        let dir = tempdir().expect("tempdir creation failed");
        let cable_dir = dir.path().join(CABLE_DIR_NAME);
        fs::create_dir_all(&cable_dir)
            .expect("cable directory creation failed");
        let config_file = dir.path().join(CONFIG_FILE_NAME);
        fs::write(&config_file, "").expect("failed touching config file");
        Self {
            config_file,
            cable_dir,
            _tempdir: dir,
        }
    }

    /// Write the config file in this temporary configuration directory.
    pub fn write_config(&self, contents: &str) -> io::Result<()> {
        fs::write(&self.config_file, contents)
    }

    /// Write a new channel inside this temporary configuration directory.
    pub fn write_channel(&self, name: &str, contents: &str) -> io::Result<()> {
        fs::write(self.cable_dir.join(name).with_extension("toml"), contents)
    }
}
