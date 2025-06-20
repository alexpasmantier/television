use std::time::Duration;
use std::{io::Write, thread::sleep};

use portable_pty::{CommandBuilder, PtySize, native_pty_system};

const DEFAULT_CONFIG_FILE: &str = "./.config/config.toml";
#[cfg(unix)]
const DEFAULT_CABLE_DIR: &str = "./cable/unix";
#[cfg(windows)]
const DEFAULT_CABLE_DIR: &str = "./cable/windows";

const DEFAULT_PTY_SIZE: PtySize = PtySize {
    rows: 30,
    cols: 120,
    pixel_width: 0,
    pixel_height: 0,
};

const DEFAULT_DELAY: Duration = Duration::from_millis(200);

struct PtyTester {
    pair: portable_pty::PtyPair,
    cwd: std::path::PathBuf,
    delay: Duration,
    pub reader: Box<dyn std::io::Read + Send>,
    pub writer: Box<dyn std::io::Write + Send + 'static>,
    void_buffer: Vec<u8>,
    parser: vt100::Parser,
}

impl PtyTester {
    fn new() -> Self {
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
    pub fn spawn_command(
        &mut self,
        mut cmd: CommandBuilder,
    ) -> Box<dyn portable_pty::Child + Send + Sync> {
        cmd.cwd(&self.cwd);
        let child = self.pair.slave.spawn_command(cmd).unwrap();
        sleep(self.delay);
        child
    }

    fn read_raw_output(&mut self) -> String {
        self.void_buffer.fill(0);
        let bytes_read = self.reader.read(&mut self.void_buffer).unwrap();
        String::from_utf8_lossy(&self.void_buffer[..bytes_read]).to_string()
    }

    /// Reads the output from the child process's pty.
    /// This method processes the output using a vt100 parser to handle terminal escape
    /// sequences.
    fn read_tui_output(&mut self) -> String {
        self.void_buffer.fill(0);
        let bytes_read = self.reader.read(&mut self.void_buffer).unwrap();
        self.parser.process(&self.void_buffer[..bytes_read]);
        self.parser.screen().contents()
    }

    /// Writes input to the child process's stdin.
    pub fn write_input(&mut self, input: &str) {
        write!(self.writer, "{}", input).unwrap();
        self.writer.flush().unwrap();
        sleep(self.delay);
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
                    sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    panic!("Error waiting for process: {}", e);
                }
            }
        }
        panic!("Process did not exit in time");
    }

    const FRAME_STABILITY_TIMEOUT: Duration = Duration::from_millis(1000);

    pub fn get_tui_frame(&mut self) -> String {
        // wait for the UI to stabilize with a timeout
        let mut frame = String::new();
        let start_time = std::time::Instant::now();
        loop {
            let new_frame = self.read_tui_output();
            if new_frame == frame {
                break;
            }
            frame = new_frame;
            assert!(
                start_time.elapsed() < Self::FRAME_STABILITY_TIMEOUT,
                "UI did not stabilize within {:?}. Last frame:\n{}",
                Self::FRAME_STABILITY_TIMEOUT,
                frame
            );
        }
        frame
    }

    /// Asserts that the output contains the expected string.
    pub fn assert_tui_output_contains(&mut self, expected: &str) {
        let frame = self.get_tui_frame();
        assert!(
            frame.contains(expected),
            "Expected output to contain\n'{}'\nbut got:\n{}",
            expected,
            frame
        );
    }

    pub fn assert_raw_output_contains(&mut self, expected: &str) {
        let output = self.read_raw_output();
        assert!(
            output.contains(expected),
            "Expected output to contain '{}', but got:\n{}",
            expected,
            output
        );
    }
}

fn ctrl(c: char) -> String {
    ((c as u8 & 0x1F) as char).to_string()
}

const ENTER: &str = "\r";

const TV_BIN_PATH: &str = "./target/debug/tv";
const LOCAL_CONFIG_AND_CABLE: &[&str] = &[
    "--cable-dir",
    DEFAULT_CABLE_DIR,
    "--config-file",
    DEFAULT_CONFIG_FILE,
];

fn tv() -> CommandBuilder {
    CommandBuilder::new(TV_BIN_PATH)
}

fn tv_with_args(args: &[&str]) -> CommandBuilder {
    let mut cmd = tv();
    cmd.args(args);
    cmd
}

fn tv_local_config_and_cable_with_args(args: &[&str]) -> CommandBuilder {
    let mut cmd = tv();
    cmd.args(LOCAL_CONFIG_AND_CABLE);
    cmd.args(args);
    cmd
}

mod e2e {
    use super::*;
    mod tv_subcommands {
        use super::*;

        /// Really just a sanity check
        #[test]
        fn tv_version() {
            let mut tester = PtyTester::new();
            let mut child = tester.spawn_command(tv_with_args(&["--version"]));

            tester.assert_raw_output_contains("television");
            PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
        }

        /// Really just a sanity check
        #[test]
        fn tv_help() {
            let mut tester = PtyTester::new();
            let mut child = tester.spawn_command(tv_with_args(&["--help"]));

            tester.assert_raw_output_contains("A cross-platform");
            PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
        }

        /// Tests the `tv list-channels` command.
        ///
        /// We expect this to list all available channels in the cable directory.
        #[test]
        fn tv_list_channels() {
            let mut tester = PtyTester::new();
            let mut child =
                tester.spawn_command(tv_local_config_and_cable_with_args(&[
                    "list-channels",
                ]));

            // Check what's in the cable directory
            let cable_dir_filenames = std::fs::read_dir(DEFAULT_CABLE_DIR)
                .expect("Failed to read cable directory")
                .filter_map(Result::ok)
                .filter_map(|entry| {
                    // this is pretty lazy and can be improved later on
                    entry.path().extension().and_then(|ext| {
                        if ext == "toml" {
                            entry.path().file_stem().and_then(|stem| {
                                stem.to_str().map(String::from)
                            })
                        } else {
                            None
                        }
                    })
                })
                .collect::<Vec<_>>();

            // Check if the output contains all channel names
            let output = tester.read_raw_output();
            for channel in cable_dir_filenames {
                assert!(
                    output.contains(&channel),
                    "Channel '{}' not found in output: {}",
                    channel,
                    output
                );
            }

            PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
        }

        #[test]
        /// This simply tests that the command exits successfully.
        fn tv_init_zsh() {
            let mut tester = PtyTester::new();
            let mut child =
                tester.spawn_command(tv_with_args(&["init", "zsh"]));

            PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
        }
    }

    mod general_ui {
        use super::*;

        #[test]
        fn toggle_help() {
            let mut tester = PtyTester::new();
            let mut child =
                tester.spawn_command(tv_local_config_and_cable_with_args(&[]));

            tester.write_input(&ctrl('g'));

            tester.assert_tui_output_contains("current mode:");

            // Exit the application
            tester.write_input(&ctrl('c'));
            PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
        }

        #[test]
        // FIXME: was lazy, this should be more robust
        fn toggle_preview() {
            let mut tester = PtyTester::new();
            let mut child =
                tester.spawn_command(tv_local_config_and_cable_with_args(&[]));

            let with_preview =
                "╭───────────────────────── files ──────────────────────────╮";
            tester.assert_tui_output_contains(with_preview);

            // Toggle preview
            tester.write_input(&ctrl('o'));

            let without_preview = "╭─────────────────────────────────────────────────────── files ────────────────────────────────────────────────────────╮";
            tester.assert_tui_output_contains(without_preview);

            // Toggle preview
            tester.write_input(&ctrl('o'));

            tester.assert_tui_output_contains(with_preview);

            // Exit the application
            tester.write_input(&ctrl('c'));
            PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
        }
    }

    mod channels {
        use super::*;

        #[test]
        fn tv_ctrl_c() {
            let mut tester = PtyTester::new();
            let mut child =
                tester.spawn_command(tv_local_config_and_cable_with_args(&[
                    "files",
                ]));

            tester.write_input(&ctrl('c'));

            // Check if the child process exited with a timeout
            PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
        }

        /// Test that the various channels open correctly, spawn a UI that contains the
        /// expected channel name, and exit cleanly when Ctrl-C is pressed.
        macro_rules! test_channel {
    ($($name:ident: $channel_name:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let mut tester = PtyTester::new();
            let mut child = tester.spawn_command(tv_local_config_and_cable_with_args(&[
                $channel_name,
            ]));

            tester.assert_tui_output_contains(&format!(
                "── {} ──",
                $channel_name
            ));

            tester.write_input(&ctrl('c'));
            PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
        }
    )*
    }
}

        test_channel! {
            test_channel_files: "files",
            test_channel_dirs: "dirs",
            test_channel_env: "env",
            test_channel_git_log: "git-log",
            test_channel_git_reflog: "git-reflog",
            test_channel_git_branch: "git-branch",
            test_channel_text: "text",
            test_channel_diff: "git-diff",
        }
    }

    mod remote_control {
        use super::*;

        #[test]
        fn tv_remote_control_shows() {
            let mut tester = PtyTester::new();
            let mut child = tester
                .spawn_command(tv_local_config_and_cable_with_args(&["dirs"]));

            // open remote control mode
            tester.write_input(&ctrl('t'));

            tester.assert_tui_output_contains("──Remote Control──");

            // exit remote then app
            tester.write_input(&ctrl('c'));
            tester.write_input(&ctrl('c'));

            PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
        }

        #[test]
        fn tv_remote_control_zaps() {
            let mut tester = PtyTester::new();
            let mut child = tester
                .spawn_command(tv_local_config_and_cable_with_args(&["dirs"]));

            // open remote control mode
            tester.write_input(&ctrl('t'));
            tester.write_input("files");
            tester.write_input(ENTER);

            tester.assert_tui_output_contains("── files ──");

            // exit remote then app
            tester.write_input(&ctrl('c'));
            tester.write_input(&ctrl('c'));

            PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
        }
    }

    mod cli {
        use super::*;

        #[test]
        fn tv_custom_input_header_and_preview_size() {
            let mut tester = PtyTester::new();
            let mut cmd = tv_local_config_and_cable_with_args(&["files"]);
            cmd.args(["--input-header", "toasted bagels"]);
            let mut child = tester.spawn_command(cmd);

            tester.assert_tui_output_contains("── toasted bagels ──");

            tester.write_input(&ctrl('c'));

            PtyTester::assert_exit_ok(&mut child, DEFAULT_DELAY);
        }
    }
}
