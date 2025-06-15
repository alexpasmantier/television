use std::thread::sleep;
use std::time::Duration;

use portable_pty::{CommandBuilder, PtySize, native_pty_system};

const DEFAULT_CONFIG_FILE: &str = "./.config/config.toml";
#[cfg(unix)]
const DEFAULT_CABLE_DIR: &str = "./cable/unix";
#[cfg(windows)]
const DEFAULT_CABLE_DIR: &str = "./cable/windows";

const DEFAULT_PTY_SIZE: PtySize = PtySize {
    rows: 20,
    cols: 60,
    pixel_width: 0,
    pixel_height: 0,
};

const DEFAULT_TIMEOUT: Duration = Duration::from_millis(400);

fn assert_exit_ok(
    child: &mut Box<dyn portable_pty::Child + Send + Sync + 'static>,
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

#[test]
fn tv_version() {
    let pty_system = native_pty_system();
    // Create a new pty
    let pair = pty_system.openpty(DEFAULT_PTY_SIZE).unwrap();

    // Spawn a tv process in the pty
    let mut cmd = CommandBuilder::new("./target/debug/tv");
    cmd.cwd(std::env::current_dir().unwrap());
    cmd.args(["--version"]);
    let mut child: Box<dyn portable_pty::Child + Send + Sync + 'static> =
        pair.slave.spawn_command(cmd).unwrap();
    sleep(Duration::from_millis(200));

    // Read the output from the pty
    let mut buf = [0; 512];
    let mut reader = pair.master.try_clone_reader().unwrap();
    let _ = reader.read(&mut buf).unwrap();
    let output = String::from_utf8_lossy(&buf);
    assert!(output.contains("television"));

    assert_exit_ok(&mut child, DEFAULT_TIMEOUT);
}

#[test]
fn tv_help() {
    let pty_system = native_pty_system();
    // Create a new pty
    let pair = pty_system.openpty(DEFAULT_PTY_SIZE).unwrap();

    // Spawn a tv process in the pty
    let mut cmd = CommandBuilder::new("./target/debug/tv");
    cmd.cwd(std::env::current_dir().unwrap());
    cmd.args(["--help"]);
    let mut child = pair.slave.spawn_command(cmd).unwrap();
    sleep(Duration::from_millis(200));

    // Read the output from the pty
    let mut buf = [0; 512];
    let mut reader = pair.master.try_clone_reader().unwrap();
    let _ = reader.read(&mut buf).unwrap();
    let output = String::from_utf8_lossy(&buf);
    assert!(output.contains("A cross-platform"));

    assert_exit_ok(&mut child, DEFAULT_TIMEOUT);
}

#[test]
fn tv_list_channels() {
    let pty_system = native_pty_system();
    // Create a new pty
    let pair = pty_system.openpty(DEFAULT_PTY_SIZE).unwrap();

    // Spawn a tv process in the pty
    let mut cmd = CommandBuilder::new("./target/debug/tv");
    cmd.cwd(std::env::current_dir().unwrap());
    cmd.args([
        "--cable-dir",
        DEFAULT_CABLE_DIR,
        "--config-file",
        DEFAULT_CONFIG_FILE,
        "list-channels",
    ]);
    let mut child = pair.slave.spawn_command(cmd).unwrap();
    sleep(Duration::from_millis(200));

    // Read the output from the pty
    let mut buf = [0; 512];
    let mut reader = pair.master.try_clone_reader().unwrap();
    let _ = reader.read(&mut buf).unwrap();
    let output = String::from_utf8_lossy(&buf);
    assert!(output.contains("files"), "Output: {}", output);
    assert!(output.contains("dirs"), "Output: {}", output);

    assert_exit_ok(&mut child, DEFAULT_TIMEOUT);
}

#[test]
fn tv_init_zsh() {
    let pty_system = native_pty_system();
    // Create a new pty
    let pair = pty_system.openpty(DEFAULT_PTY_SIZE).unwrap();

    // Spawn a tv process in the pty
    let mut cmd = CommandBuilder::new("./target/debug/tv");
    cmd.cwd(std::env::current_dir().unwrap());
    cmd.args([
        "--cable-dir",
        DEFAULT_CABLE_DIR,
        "--config-file",
        DEFAULT_CONFIG_FILE,
        "init",
        "zsh",
    ]);
    let mut child = pair.slave.spawn_command(cmd).unwrap();
    sleep(Duration::from_millis(200));

    assert_exit_ok(&mut child, DEFAULT_TIMEOUT);
}

/// Creates a command to run `tv` with the repo's config and cable directory
/// using the cable directory as cwd.
fn tv_command(channel: &str) -> CommandBuilder {
    let mut cmd = CommandBuilder::new("./target/debug/tv");
    cmd.cwd(std::env::current_dir().unwrap());
    cmd.args([
        "--cable-dir",
        DEFAULT_CABLE_DIR,
        "--config-file",
        DEFAULT_CONFIG_FILE,
        channel,
    ]);
    cmd
}

#[test]
fn tv_ctrl_c() {
    let pty_system = native_pty_system();
    // Create a new pty
    let pair = pty_system.openpty(DEFAULT_PTY_SIZE).unwrap();

    // Spawn a tv process in the pty
    let mut child = pair.slave.spawn_command(tv_command("files")).unwrap();
    sleep(Duration::from_millis(200));

    // Send Ctrl-C to the process
    let mut writer = pair.master.take_writer().unwrap();
    writeln!(writer, "\x03").unwrap(); // Ctrl-C

    // Check if the child process exited with a timeout
    assert_exit_ok(&mut child, DEFAULT_TIMEOUT);
}

#[test]
fn tv_remote_control() {
    let pty_system = native_pty_system();
    // Create a new pty
    let pair = pty_system.openpty(DEFAULT_PTY_SIZE).unwrap();

    // Spawn a tv process in the pty
    let mut child = pair.slave.spawn_command(tv_command("files")).unwrap();
    sleep(Duration::from_millis(200));

    // Send Ctrl-T to the process (open remote control mode)
    let mut writer = pair.master.take_writer().unwrap();
    writeln!(writer, "\x14").unwrap(); // Ctrl-T
    sleep(Duration::from_millis(200));

    let mut buf = [0; 5096]; // Buffer size for reading output
    let mut reader = pair.master.try_clone_reader().unwrap();
    let _ = reader.read(&mut buf).unwrap();

    let s = String::from_utf8_lossy(&buf);

    assert!(s.contains("Remote Control"));

    // Send Ctrl-c to exit remote control mode
    writeln!(writer, "\x03").unwrap();
    sleep(Duration::from_millis(200));
    // resend Ctrl-c to finally exit
    writeln!(writer, "\x03").unwrap();
    sleep(Duration::from_millis(200));

    assert_exit_ok(&mut child, DEFAULT_TIMEOUT);
}

macro_rules! test_channel {
    ($($name:ident: $channel_name:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let pty_system = native_pty_system();
            // Create a new pty
            let pair = pty_system.openpty(DEFAULT_PTY_SIZE).unwrap();
            let cmd = tv_command(&$channel_name);

            let mut child = pair.slave.spawn_command(cmd).unwrap();
            sleep(Duration::from_millis(300));

            // Read the output from the pty
            let mut buf = [0; 5096];
            let mut reader = pair.master.try_clone_reader().unwrap();
            let _ = reader.read(&mut buf).unwrap();
            let output = String::from_utf8_lossy(&buf);

            assert!(
                output.contains(&$channel_name),
                "Unable to find channel name (\"{}\") in output: {}",
                &$channel_name,
                output
            );

            // Send Ctrl-C to the process
            let mut writer = pair.master.take_writer().unwrap();
            writeln!(writer, "\x03").unwrap(); // Ctrl-C
            sleep(Duration::from_millis(200));

            assert_exit_ok(&mut child, DEFAULT_TIMEOUT);
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
