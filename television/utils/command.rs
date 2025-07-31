use crate::{
    channels::{
        entry::Entry,
        prototypes::{ActionSpec, ExecutionMode, OutputMode, Template},
    },
    utils::shell::Shell,
};
use anyhow::Result;
use rustc_hash::FxHashSet;
use std::{
    collections::HashMap,
    process::{Command, ExitStatus, Stdio},
};
use tracing::debug;

#[cfg(not(unix))]
use tracing::warn;

/// Create a shell command configured for the current platform
///
/// Creates a `Command` instance configured with the appropriate shell for the current platform
/// and sets up the command to execute the provided command string.
///
/// # Arguments
/// * `command` - The command string to execute
/// * `interactive` - Whether to run in interactive mode (Unix only)
/// * `envs` - Environment variables to set for the command
///
/// # Returns
/// * `Command` - A configured `Command` ready for execution
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

/// Format a command string from entries using template processing
///
/// Takes a set of entries, concatenates them with the specified separator, and processes them through
/// the provided template to create a formatted command. The template handles escaping, formatting, and any transformations.
///
/// # Arguments
/// * `entries` - A reference to a set of Entry items to process
/// * `template` - The template to process the entries through
/// * `separator` - The separator to use when joining entries
///
/// # Returns
/// * `Result<String>` - The final formatted command ready for execution
///
/// # Example
/// ```no_run
/// # use television::{
///     channels::{entry::Entry, prototypes::Template},
///     utils::command::format_command
/// };
/// # use rustc_hash::FxHashSet;
/// let mut entries = FxHashSet::default();
/// entries.insert(Entry::new("file1.txt".to_string()));
/// entries.insert(Entry::new("file 2.txt".to_string()));
/// let template = Template::parse("nvim {split:\\n:..|map:{append:'|prepend:'}|join: }").unwrap();
/// let result = format_command(&entries, &template, "\n").unwrap();
/// // Should produce something like: nvim 'file1.txt' 'file 2.txt'
/// assert!(result.starts_with("nvim "));
/// assert!(result.contains("'file1.txt'"));
/// assert!(result.contains("'file 2.txt'"));
/// ```
pub fn format_command(
    entries: &FxHashSet<Entry>,
    template: &Template,
    separator: &str,
) -> Result<String> {
    debug!(
        "Formatting command from {} entries using template",
        entries.len()
    );

    // Concatenate entries with separator for template processing
    let entries_str = entries
        .iter()
        .map(|entry| entry.raw.as_str())
        .collect::<Vec<_>>()
        .join(separator);
    debug!("Concatenated entries input: {:?}", entries_str);

    // Process through template system
    debug!("Using template: {}", template);
    let formatted_command = template.format(&entries_str)?;
    debug!("Final command: {:?}", formatted_command);

    Ok(formatted_command)
}

/// Execute an external action with the appropriate execution mode and output handling
///
/// Takes an `ActionSpec` and a set of entries, creates a command using the action's template,
/// and executes the resulting command with the specified execution mode and output handling.
///
/// # Arguments
/// * `action_spec` - The `ActionSpec` containing the command template, execution mode, and output mode
/// * `entries` - A reference to a set of Entry items to process
///
/// # Returns
/// * `Result<ExitStatus>` - The exit status of the executed command
///
/// # Behavior
/// - `ExecutionMode::Execute` - Executes the command and waits for completion (inherits stdio)
/// - `ExecutionMode::Fork` - Spawns the command with stdio configured based on `output_mode` (TODO: implement)
/// - `OutputMode::Capture` - Currently inherits stdio (TODO: implement)
/// - `OutputMode::Discard` - Redirects all stdio to null
pub fn execute_action(
    action_spec: &ActionSpec,
    entries: &FxHashSet<Entry>,
) -> Result<ExitStatus> {
    // For now, preserve existing behavior regardless of the new flags
    // In the future, this will branch based on action_spec.mode and action_spec.output_mode

    debug!("Executing external action with {} entries", entries.len());

    // Create command from entries using template
    let template: &Template = action_spec.command.get_nth(0);
    let formatted_command =
        format_command(entries, template, &action_spec.separator)?;

    let mut cmd = shell_command(
        &formatted_command,
        action_spec.command.interactive,
        &action_spec.command.env,
    );

    // Execute based on execution mode
    match action_spec.mode {
        ExecutionMode::Execute => {
            // For Execute mode, let the new process inherit file descriptors naturally
            // TODO: use execve to replace current process
            let mut child = cmd.spawn()?;
            Ok(child.wait()?)
        }
        ExecutionMode::Fork => {
            // For Fork mode, configure stdio based on output mode
            match action_spec.output_mode {
                OutputMode::Capture => {
                    // TODO: For now, inherit stdio (future: capture output for processing)
                    cmd.stdin(Stdio::inherit())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit());
                }
                OutputMode::Discard => {
                    // Discard output silently
                    cmd.stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null());
                }
            }

            let mut child = cmd.spawn()?;
            Ok(child.wait()?)
        }
    }
}
