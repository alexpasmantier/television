#![allow(dead_code)]

use std::path::PathBuf;
use std::time::Duration;
use std::{fs, io};
use std::{io::Write, thread::sleep};

use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use television::cable::CABLE_DIR_NAME;
use television::config::CONFIG_FILE_NAME;
use tempfile::{TempDir, tempdir};

pub const CI_ENV_VAR: &str = "TV_CI";

pub fn is_ci() -> bool {
    // Check if the environment variable indicates a CI environment
    std::env::var("TV_CI").is_ok()
}

pub const DEFAULT_CONFIG_FILE: &str = "./.config/config.toml";
#[cfg(unix)]
pub const DEFAULT_CABLE_DIR: &str = "./cable/unix";
#[cfg(windows)]
pub const DEFAULT_CABLE_DIR: &str = "./cable/windows";

pub const TARGET_DIR: &str = "./tests/target_dir";

pub const DEFAULT_PTY_SIZE: PtySize = PtySize {
    rows: 30,
    cols: 120,
    pixel_width: 0,
    pixel_height: 0,
};

pub const DEFAULT_DELAY: Duration = Duration::from_millis(100);

/// A helper to test terminal user interfaces (TUIs) using a pseudo-terminal (pty).
///
/// This struct provides methods to spawn commands in a pty, read their output, and send input to
/// them.
///
/// # Example
/// ```ignore
/// fn test_custom_input_header_and_preview_size() {
///     let mut tester = PtyTester::new();
///
///     // Create the tv command using a custom input header
///     let mut cmd = tv_local_config_and_cable_with_args(&["files"]);
///     cmd.args(["--input-header", "toasted bagels"]);
///
///     // Spawn the command in the pty
///     let mut child = tester.spawn_command_tui(cmd);
///
///     // Assert that the TUI contains the expected output
///     tester.assert_tui_frame_contains("── toasted bagels ──");
///
///     // Send a Ctrl+C to exit the application
///     tester.send(&ctrl('c'));
///
///     // Assert that the child process exits successfully
///     PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY * 2);
/// }
/// ```
pub struct PtyTester {
    pair: portable_pty::PtyPair,
    cwd: std::path::PathBuf,
    delay: Duration,
    reader: Box<dyn std::io::Read + Send>,
    writer: Box<dyn std::io::Write + Send + 'static>,
    /// A large pre-allocated buffer to read from the pty.
    void_buffer: Vec<u8>,
    parser: vt100::Parser,
}

impl PtyTester {
    pub fn new() -> Self {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(DEFAULT_PTY_SIZE).unwrap();
        let reader = pair.master.try_clone_reader().unwrap();
        let writer = pair.master.take_writer().unwrap();
        let cwd = std::env::current_dir().unwrap();
        let delay = DEFAULT_DELAY;
        let size = pair.master.get_size().unwrap();
        let parser = vt100::Parser::new(size.rows, size.cols, 0);
        PtyTester {
            pair,
            cwd,
            delay,
            reader: Box::new(reader),
            writer: Box::new(writer),
            void_buffer: vec![0; 2usize.pow(20)], // 1 MiB buffer
            parser,
        }
    }

    #[allow(dead_code)]
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Spawns a command in the pty, returning a boxed child process.
    ///
    /// # Warning
    /// **If the command is expected to produce a TUI use `spawn_command_tui` instead.**
    ///
    /// # Example
    /// See [`PtyTester`]
    pub fn spawn_command(
        &mut self,
        mut cmd: CommandBuilder,
    ) -> Box<dyn portable_pty::Child + Send + Sync> {
        cmd.cwd(&self.cwd);
        let child = self.pair.slave.spawn_command(cmd).unwrap();
        sleep(self.delay * 3);

        child
    }

    /// Spawns a command for which we expect to get a TUI in the pty and returns a boxed child process.
    ///
    /// # Warning
    /// **If the command is not expected to produce a TUI use `spawn_command` instead.**
    ///
    /// # Example
    /// See [`PtyTester`]
    pub fn spawn_command_tui(
        &mut self,
        mut cmd: CommandBuilder,
    ) -> Box<dyn portable_pty::Child + Send + Sync> {
        cmd.cwd(&self.cwd);
        let mut child = self.pair.slave.spawn_command(cmd).unwrap();
        sleep(self.delay * 3);

        self.assert_tui_running(&mut child);
        // wait for the TUI to stabilize
        let _ = self.get_tui_frame();
        child
    }

    pub fn read_raw_output(&mut self) -> String {
        self.void_buffer.fill(0);
        let bytes_read = self.reader.read(&mut self.void_buffer).unwrap();
        String::from_utf8_lossy(&self.void_buffer[..bytes_read]).to_string()
    }

    /// Reads the output from the child process's pty.
    ///
    /// This method processes the output using a vt100 parser to handle terminal escape
    /// sequences and returns the contents of the screen as a string.
    fn read_tui_output(&mut self) -> String {
        self.void_buffer.fill(0);
        let bytes_read = self.reader.read(&mut self.void_buffer).unwrap();
        self.parser.process(&self.void_buffer[..bytes_read]);
        self.parser.screen().contents()
    }

    /// Writes input to the child process's stdin.
    ///
    /// This method sends the input string to the pty's writer and flushes it to ensure
    /// the input is sent immediately.
    ///
    /// Convenience methods and constants are provided for common control sequences like `ctrl`,
    /// `ENTER`, and `ESC`. (See [`ctrl`], [`ENTER`], and [`ESC`])
    pub fn send(&mut self, input: &str) {
        write!(self.writer, "{}", input).unwrap();
        self.writer.flush().unwrap();
        sleep(self.delay);
    }

    /// asserts that the TUI is running by checking if the child process is still active.
    pub fn assert_tui_running(
        &mut self,
        child: &mut Box<dyn portable_pty::Child + Send + Sync>,
    ) {
        // Check if the child process is still running
        match child.try_wait() {
            Ok(Some(_)) => {
                panic!(
                    "Child process exited prematurely with output:\n{}",
                    self.read_tui_output()
                );
            }
            Ok(None) => {
                // Process is still running, continue
            }
            Err(e) => {
                panic!("Error checking child process status: {}", e);
            }
        }
    }

    /// Waits for the child process to exit, asserting that it exits with a success status.
    pub fn assert_exit_ok(
        child: &mut Box<dyn portable_pty::Child + Send + Sync>,
        timeout: Duration,
    ) {
        let now = std::time::Instant::now();
        while now.elapsed() < timeout {
            match child.try_wait() {
                Ok(Some(status)) => {
                    assert!(
                        status.success(),
                        "Process exited with non-zero status: {:?}",
                        status
                    );
                    return;
                }
                Ok(None) => {
                    // Process is still running, continue waiting
                    sleep(timeout / 6);
                }
                Err(e) => {
                    panic!("Error waiting for process: {}", e);
                }
            }
        }
        panic!("Process did not exit in time: {:?}", child.try_wait());
    }

    /// How long to wait for the TUI to stabilize before asserting its output.
    const FRAME_STABILITY_TIMEOUT: Duration = Duration::from_millis(3000);

    /// Gets the current TUI frame, ensuring it has stabilized.
    ///
    /// # Example
    /// ```ignore
    /// let mut tester = PtyTester::new();
    ///
    /// let child = tester.spawn_command_tui(tv());
    ///
    /// let output = tester.get_tui_frame();
    ///
    /// println!("TUI Output:\n{}", output);
    ///
    /// /*
    ///     ╭───────────────────────── files ──────────────────────────╮
    ///     │>                                                1 / 138  │
    ///     ╰──────────────────────────────────────────────────────────╯
    ///     ╭──────────────────────── Results ─────────────────────────╮
    ///     │> CHANGELOG.md                                            │
    ///     │  CODE_OF_CONDUCT.md                                      │
    ///     │  CONTRIBUTING.md                                         │
    ///     │  Cargo.toml                                              │
    ///     │  cable/unix/dotfiles.toml                                │
    ///     │  cable/unix/env.toml                                     │
    ///     │  cable/unix/files.toml                                   │
    ///     │  cable/unix/fish-history.toml                            │
    ///     │  cable/unix/git-branch.toml                              │
    ///     ╰─────────── help: <Ctrl-g>  preview: <Ctrl-o> ────────────╯
    /// */
    /// ```
    pub fn get_tui_frame(&mut self) -> String {
        // wait for the UI to stabilize with a timeout
        let mut frame = String::new();
        let start_time = std::time::Instant::now();
        // wait till we get 2 consecutive frames that are the same
        let mut counter = 0;
        while counter < 2 {
            let new_frame = self.read_tui_output();
            if new_frame == frame {
                counter += 1;
            } else {
                frame = new_frame;
                counter = 0;
            }
            assert!(
                start_time.elapsed() < Self::FRAME_STABILITY_TIMEOUT,
                "UI did not stabilize within {:?}. Last frame:\n{}",
                Self::FRAME_STABILITY_TIMEOUT,
                frame
            );
            // Sleep briefly to allow the UI to update
            sleep(Duration::from_millis(20));
        }
        frame
    }

    /// Asserts that the frame contains the expected string.
    pub fn assert_tui_frame_contains(&mut self, expected: &str) {
        let frame = self.get_tui_frame();
        assert!(
            frame.contains(expected),
            "Expected output to contain\n'{}'\nbut got:\n{}",
            expected,
            frame
        );
    }

    /// Asserts that the frame contains all expected strings.
    pub fn assert_tui_frame_contains_all(&mut self, expected: &[&str]) {
        let frame = self.get_tui_frame();
        for &exp in expected {
            assert!(
                frame.contains(exp),
                "Expected output to contain\n'{}'\nbut got:\n{}",
                exp,
                frame
            );
        }
    }

    /// Asserts that the frame does not contain any of the expected strings.
    pub fn assert_tui_frame_contains_none(&mut self, expected: &[&str]) {
        let frame = self.get_tui_frame();
        for &exp in expected {
            assert!(
                !frame.contains(exp),
                "Expected output to not contain\n'{}'\nbut got:\n{}",
                exp,
                frame
            );
        }
    }

    /// Asserts that the frame does not contain the expected string.
    pub fn assert_not_tui_frame_contains(&mut self, expected: &str) {
        let frame = self.get_tui_frame();
        assert!(
            !frame.contains(expected),
            "Expected output to not contain\n'{}'\nbut got:\n'{}'",
            expected,
            frame
        );
    }

    pub fn assert_raw_output_contains(&mut self, expected: &str) {
        let output = self.read_raw_output();
        assert!(
            output.contains(expected),
            "Expected output to contain '{}', but got:\n{:?}",
            expected,
            output
        );
    }
}

pub fn ctrl(c: char) -> String {
    ((c as u8 & 0x1F) as char).to_string()
}

pub fn f(c: u8) -> String {
    let seq = match c {
        1 => "\x1bOP",    // F1
        2 => "\x1bOQ",    // F2
        3 => "\x1bOR",    // F3
        4 => "\x1bOS",    // F4
        5 => "\x1b[15~",  // F5
        6 => "\x1b[17~",  // F6
        7 => "\x1b[18~",  // F7
        8 => "\x1b[19~",  // F8
        9 => "\x1b[20~",  // F9
        10 => "\x1b[21~", // F10
        11 => "\x1b[23~", // F11
        12 => "\x1b[24~", // F12
        _ => {
            panic!("Invalid function key: {}", c);
        }
    };

    seq.to_string()
}

pub const ENTER: &str = "\r";
pub const ESC: &str = "\x1b";

pub const TV_BIN_PATH: &str = "./target/debug/tv";
pub const LOCAL_CONFIG_AND_CABLE: &[&str] = &[
    "--cable-dir",
    DEFAULT_CABLE_DIR,
    "--config-file",
    DEFAULT_CONFIG_FILE,
];

/// A command builder initialized with the tv binary path.
pub fn tv() -> CommandBuilder {
    CommandBuilder::new(TV_BIN_PATH)
}

/// A command builder initialized with the tv binary path and the provided arguments.
pub fn tv_with_args(args: &[&str]) -> CommandBuilder {
    let mut cmd = tv();
    cmd.args(args);
    cmd
}

/// A command builder initialized with the tv binary path, using the repository's local config and
/// cable directory, and the provided arguments.
pub fn tv_local_config_and_cable_with_args(args: &[&str]) -> CommandBuilder {
    let mut cmd = tv();
    cmd.args(LOCAL_CONFIG_AND_CABLE);
    cmd.args(args);
    cmd
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
    tempdir: TempDir,
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
            tempdir: dir,
        }
    }

    /// Write the config file in this temporary configuration directory.
    ///
    /// This will overwrite any existing content.
    ///
    /// # Example
    /// ```rs
    /// let temp_config = TempConfig::init();
    /// let config_content: &str = r#"
    /// [ui]
    /// theme = "default"
    /// "#;
    ///
    /// temp_config.write_config(config_content)?;
    /// ```
    pub fn write_config(&self, contents: &str) -> io::Result<()> {
        fs::write(&self.config_file, contents)
    }

    /// Write a new channel inside this temporary configuration directory.
    ///
    /// This will create a new file with the given name and `.toml` extension.
    ///
    /// # Example
    /// ```rs
    /// let temp_config = TempConfig::init();
    ///
    /// let channel_content: &str = r#"
    /// [metadata]
    /// name = "my-channel"
    /// description = "..."
    /// ...
    /// "#;
    ///
    /// temp_config.write_channel("my-channel", channel_content)?;
    /// ```
    pub fn write_channel(&self, name: &str, contents: &str) -> io::Result<()> {
        fs::write(self.cable_dir.join(name).with_extension("toml"), contents)
    }
}
