use crate::{
    channels::prototypes::{ActionSpec, ExecutionMode, OutputMode},
    utils::shell::Shell,
};
use std::{collections::HashMap, process::Command};

#[cfg(not(unix))]
use tracing::warn;

pub fn shell_command<S>(
    command: &str,
    interactive: bool,
    envs: &HashMap<String, String, S>,
) -> Command {
    let shell = Shell::from_env().unwrap_or_default();
    let mut cmd = Command::new(shell.executable());

    cmd.arg(match shell {
        Shell::PowerShell => "-Command",
        Shell::Cmd => "/C",
        _ => "-c",
    });

    #[cfg(unix)]
    if interactive {
        cmd.arg("-i");
    }

    #[cfg(not(unix))]
    if interactive {
        warn!("Interactive mode is not supported on Windows.");
    }

    cmd.envs(envs).arg(command);
    cmd
}

/// Execute an external action with the appropriate execution mode and output handling
///
/// Currently implements the existing behavior but designed to be extended with:
/// - `become` flag for execve behavior
/// - `output_mode` for different output handling modes
pub fn execute_action(
    action_spec: &ActionSpec,
    formatted_command: &str,
) -> std::io::Result<std::process::ExitStatus> {
    // For now, preserve existing behavior regardless of the new flags
    // In the future, this will branch based on action_spec.become and action_spec.output_mode

    let mut cmd = shell_command(
        formatted_command,
        action_spec.command.interactive,
        &action_spec.command.env,
    );

    // Execute based on execution mode
    match action_spec.mode {
        ExecutionMode::Execute => {
            // For Execute mode, let the new process inherit file descriptors naturally
            // TODO: use execve to replace current process
            let mut child = cmd.spawn()?;
            child.wait()
        }
        ExecutionMode::Fork => {
            // For Fork mode, configure stdio based on output mode
            match action_spec.output_mode {
                OutputMode::Capture => {
                    // TODO: For now, inherit stdio (future: capture output for processing)
                    cmd.stdin(std::process::Stdio::inherit())
                        .stdout(std::process::Stdio::inherit())
                        .stderr(std::process::Stdio::inherit());
                }
                OutputMode::Discard => {
                    // Discard output silently
                    cmd.stdin(std::process::Stdio::null())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null());
                }
            }

            let mut child = cmd.spawn()?;
            child.wait()
        }
    }
}
