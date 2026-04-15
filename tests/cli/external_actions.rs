//! Tests for external actions functionality.
//!
//! These tests verify that external actions defined in channel TOML files work correctly,
//! including keybinding integration and command execution.

use std::fs;

use tempfile::TempDir;

use super::super::common::*;

/// Helper to create a custom cable directory with external actions.
fn write_toml_config(
    cable_dir: &std::path::Path,
    filename: &str,
    content: &str,
) {
    let toml_path = cable_dir.join(filename);
    fs::write(&toml_path, content).unwrap();
}

const FILES_TOML_WITH_ACTIONS: &str = r#"
[metadata]
name = "files"
description = "A channel to select files and directories"
requirements = ["fd", "bat"]

[source]
command = ["fd -t f", "fd -t f -H"]

[preview]
command = "bat -n --color=always '{}'"
env = { BAT_THEME = "ansi" }

[keybindings]
shortcut = "f1"
f8 = "actions:thebatman"
f9 = "actions:lsman"

[actions.thebatman]
description = "show file content"
command = "cat '{}'"
mode = "execute"

[actions.lsman]
description = "show stats"
command = "ls '{}'"
mode = "execute"
"#;

/// Tests that external actions execute properly when triggered by keybindings.
#[test]
fn test_external_action_lsman_with_f9() {
    let pt = phantom();

    // Keep the TempDir alive for the lifetime of the test. Dropping it
    // mid-expression removes the directory (then `create_dir_all` recreates
    // the inner path, leaking it).
    let tempdir = TempDir::new().unwrap();
    let cable_dir = tempdir.path().join("custom_cable");
    fs::create_dir_all(&cable_dir).unwrap();
    write_toml_config(&cable_dir, "files.toml", FILES_TOML_WITH_ACTIONS);

    let s = tv_with_args(
        &pt,
        &[
            "--cable-dir",
            cable_dir.to_str().unwrap(),
            "--config-file",
            DEFAULT_CONFIG_FILE,
            "files",
            "--input",
            "LICENSE",
        ],
    )
    .start()
    .unwrap();

    // Wait until `fd -t f` has populated the list and the --input filter
    // has narrowed it to LICENSE. Matching `LICENSE` alone would spuriously
    // succeed on the `> LICENSE` input prompt before fd has even produced
    // any entries, which then makes the F9 below fire against an empty
    // selection and tv just sits there.
    s.wait()
        .text("1 / 1")
        .text("LICENSE")
        .timeout_ms(wait_timeout_ms())
        .until()
        .unwrap();

    // Send F9 to trigger the "lsman" action (mapped to ls command)
    s.send().key("f9").unwrap();

    // The external action runs `ls 'LICENSE'` and exits; its output is on
    // the primary screen after tv leaves alt-screen mode.
    let output = exit_and_output(&s);
    assert!(
        output.contains("LICENSE"),
        "expected ls output to contain 'LICENSE', got:\n{output}"
    );
}

/// Tests that external actions execute properly with F8 keybinding.
#[test]
fn test_external_action_thebatman_with_f8() {
    let pt = phantom();

    let tempdir = TempDir::new().unwrap();
    let cable_dir = tempdir.path().join("custom_cable_f8");
    fs::create_dir_all(&cable_dir).unwrap();
    write_toml_config(&cable_dir, "files.toml", FILES_TOML_WITH_ACTIONS);

    let s = tv_with_args(
        &pt,
        &[
            "--cable-dir",
            cable_dir.to_str().unwrap(),
            "--config-file",
            DEFAULT_CONFIG_FILE,
            "files",
            "--input",
            "LICENSE",
        ],
    )
    .start()
    .unwrap();

    s.wait()
        .text("1 / 1")
        .text("LICENSE")
        .timeout_ms(wait_timeout_ms())
        .until()
        .unwrap();

    // Send F8 to trigger the "thebatman" action (mapped to cat command)
    s.send().key("f8").unwrap();

    // The external action runs `cat LICENSE` and exits; its output is on
    // the primary screen after tv leaves alt-screen mode.
    let output = exit_and_output(&s);
    assert!(
        output.contains("Copyright (c)"),
        "expected cat output to contain 'Copyright (c)', got:\n{output}"
    );
}

/// Verifies that execute-mode actions attach their stdout to the
/// controlling tty, not to tv's stdout. We run tv inside a bash script
/// that captures its stdout (`out=$(tv ...)`); with the fix, the action's
/// `TTY_OK` output lands on the terminal (visible in the scrollback),
/// and the captured `$out` stays empty.
#[test]
fn test_execute_action_uses_tty_when_stdout_is_captured() {
    let pt = phantom();

    let tempdir = TempDir::new().unwrap();
    let cable_dir = tempdir.path().join("captured_stdout");
    fs::create_dir_all(&cable_dir).unwrap();

    let files_toml_content = r#"
[metadata]
name = "files"
description = "A channel to select files and directories"
requirements = ["fd", "bat"]

[source]
command = ["fd -t f", "fd -t f -H"]

[preview]
command = "bat -n --color=always '{}'"
env = { BAT_THEME = "ansi" }

[keybindings]
shortcut = "f1"
f12 = "actions:ttycheck"

[actions.ttycheck]
description = "verify execute actions use the terminal tty"
command = "if test -t 1; then printf 'TTY_OK\\n'; else printf 'TTY_BAD\\n'; fi"
shell = "bash"
mode = "execute"
"#;

    write_toml_config(&cable_dir, "files.toml", files_toml_content);

    let script = format!(
        "out=$('{}' --cable-dir '{}' --config-file '{}' files --input LICENSE); printf '\\nSHELL_CAPTURE=[%s]\\n' \"$out\"",
        TV_BIN_PATH,
        cable_dir.display(),
        DEFAULT_CONFIG_FILE,
    );

    let cwd = std::env::current_dir().expect("failed to get cwd");
    let s = pt
        .run("bash")
        .args(&["-lc", &script])
        .size(DEFAULT_COLS, DEFAULT_ROWS)
        .cwd(cwd.to_str().expect("cwd is not valid utf-8"))
        .start()
        .unwrap();

    // Wait until tv's TUI has rendered with LICENSE as the single match.
    // (Same rationale as the sibling external-action tests: matching
    // "LICENSE" alone would spuriously hit the --input prompt before fd
    // has produced any entries.)
    s.wait()
        .text("1 / 1")
        .text("LICENSE")
        .timeout_ms(wait_timeout_ms())
        .until()
        .unwrap();

    // Trigger the ttycheck action.
    s.send().key("f12").unwrap();

    // Wait for bash (and thus the whole script) to exit.
    s.wait()
        .exit_code(0)
        .timeout_ms(wait_timeout_ms())
        .until()
        .unwrap();

    // `TTY_OK` is written by the action via /dev/tty while tv is still
    // running, so by the time the session ends it has scrolled out of
    // the primary screen. Read the full scrollback to find it, and
    // read the post-exit primary screen for the `SHELL_CAPTURE=[]` line
    // bash prints after tv exits.
    let scrollback = s.scrollback(None).unwrap();
    let post_exit = s.output().unwrap();
    let combined = format!("{scrollback}\n{post_exit}");

    assert!(
        combined.contains("TTY_OK"),
        "expected action output to reach the terminal tty, got:\n{combined}"
    );
    assert!(
        combined.contains("SHELL_CAPTURE=[]"),
        "expected shell-captured stdout to stay empty, got:\n{combined}"
    );
    assert!(
        !combined.contains("TTY_BAD"),
        "expected action stdout to be reattached to the tty, got:\n{combined}"
    );
}

/// Same invariant as the execute-mode test, but for fork-mode actions:
/// tv stays alive after the action runs, so we verify the tty attachment
/// out-of-band (via a status file written by the action) and then quit
/// tv with ctrl-c to let the shell script finish.
#[test]
fn test_fork_action_uses_tty_when_stdout_is_captured() {
    let pt = phantom();

    let tempdir = TempDir::new().unwrap();
    let cable_dir = tempdir.path().join("captured_stdout_fork");
    let status_file = cable_dir.join("fork-status");
    fs::create_dir_all(&cable_dir).unwrap();

    let files_toml_content = r#"
[metadata]
name = "files"
description = "A channel to select files and directories"
requirements = ["fd", "bat"]

[source]
command = ["fd -t f", "fd -t f -H"]

[preview]
command = "bat -n --color=always '{}'"
env = { BAT_THEME = "ansi" }

[keybindings]
shortcut = "f1"
f12 = "actions:ttycheck"

[actions.ttycheck]
description = "verify fork actions use the terminal tty"
command = "if test -t 1; then printf 'ok' > '{status_file}'; else printf 'bad' > '{status_file}'; fi; printf 'FORK_STDOUT\\n'"
shell = "bash"
mode = "fork"
"#
    .replace("{status_file}", &status_file.display().to_string());

    write_toml_config(&cable_dir, "files.toml", &files_toml_content);

    let script = format!(
        "out=$('{}' --cable-dir '{}' --config-file '{}' files --input LICENSE); printf '\\nSHELL_CAPTURE=[%s]\\n' \"$out\"",
        TV_BIN_PATH,
        cable_dir.display(),
        DEFAULT_CONFIG_FILE,
    );

    let cwd = std::env::current_dir().expect("failed to get cwd");
    let s = pt
        .run("bash")
        .args(&["-lc", &script])
        .size(DEFAULT_COLS, DEFAULT_ROWS)
        .cwd(cwd.to_str().expect("cwd is not valid utf-8"))
        .start()
        .unwrap();

    s.wait()
        .text("1 / 1")
        .text("LICENSE")
        .timeout_ms(wait_timeout_ms())
        .until()
        .unwrap();

    // Fork mode: tv stays alive after the action fires. Trigger it,
    // wait for the status file to be written, then quit tv with ctrl-c
    // so the outer bash script can finish.
    s.send().key("f12").unwrap();

    // The action writes the status file synchronously before returning.
    // Wait until we see it on disk (up to wait_timeout_ms).
    let deadline = std::time::Instant::now()
        + std::time::Duration::from_millis(wait_timeout_ms());
    while !status_file.exists() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    s.send().key("ctrl-c").unwrap();
    s.wait()
        .exit_code(0)
        .timeout_ms(wait_timeout_ms())
        .until()
        .unwrap();

    let scrollback = s.scrollback(None).unwrap();
    let post_exit = s.output().unwrap();
    let combined = format!("{scrollback}\n{post_exit}");
    let status = fs::read_to_string(&status_file).unwrap_or_default();

    assert_eq!(
        status, "ok",
        "expected fork action stdout to be attached to a tty, got status {status:?}"
    );
    assert!(
        combined.contains("SHELL_CAPTURE=[]"),
        "expected shell-captured stdout to stay empty, got:\n{combined}"
    );
    assert!(
        !combined.contains("FORK_STDOUT"),
        "expected fork action stdout to bypass shell capture, got:\n{combined}"
    );
}
