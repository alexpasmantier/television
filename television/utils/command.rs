use std::process::Command;

#[cfg(not(windows))]
pub fn shell_command() -> Command {
    let mut cmd = Command::new("sh");

    cmd.arg("-c");

    cmd
}

#[cfg(windows)]
pub fn shell_command() -> Command {
    let mut cmd = Command::new("cmd");

    cmd.arg("/c");

    cmd
}
