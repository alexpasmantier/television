use std::{collections::HashMap, process::Command};

#[cfg(not(unix))]
use tracing::warn;

use super::shell::Shell;
use crate::channels::prototypes::{ActionSpec, OutputMode};

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

    // Configure stdio based on output mode (future extension point)
    match action_spec.output_mode {
        OutputMode::Inherit => {
            cmd.stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit());
        }
        OutputMode::Capture => {
            // Future: capture output for processing
            cmd.stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit());
        }
        OutputMode::Discard => {
            // Future: discard output silently
            cmd.stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit());
        }
    }

    // Execute based on become flag (future extension point)
    if action_spec.become {
        // Future: use execve to replace current process
        // For now, use normal execution
        let mut child = cmd.spawn()?;
        child.wait()
    } else {
        // Normal fork execution (current behavior)
        let mut child = cmd.spawn()?;
        child.wait()
    }
}
