use crate::{
    channels::{
        entry::Entry,
        prototypes::{ActionSpec, ExecutionMode, Template},
    },
    selector::process_entries,
    utils::shell::Shell,
};
use anyhow::Result;
use std::{
    collections::HashMap,
    os::unix::process::CommandExt,
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

/// Format a command string from entries using the unified selector system
///
/// Takes a slice of entries and processes them through the provided template
/// using the unified selector configuration to handle argument distribution,
/// escaping, and formatting.
///
/// # Arguments
/// * `entries` - A reference to a slice of Entry items to process
/// * `template` - The template to process the entries through
///
/// # Returns
/// * `Result<String>` - The final formatted command ready for execution
///
/// # Example
/// ```no_run
/// # use television::{
///     channels::{entry::Entry, prototypes::Template},
///     utils::command::format_command,
/// };
/// let entries = vec![
///     Entry::new("file1.txt".to_string()),
///     Entry::new("file 2.txt".to_string()),
/// ];
/// let template = Template::parse("nvim {}").unwrap();
/// let result = format_command(&entries, &template).unwrap();
/// // Should produce something like: nvim 'file1.txt' 'file 2.txt'
/// assert!(result.starts_with("nvim "));
/// assert!(result.contains("file1.txt"));
/// assert!(result.contains("file 2.txt"));
/// ```
pub fn format_command(
    entries: &[Entry],
    template: &Template,
) -> Result<String> {
    debug!(
        "Formatting command from {} entries using unified selector system",
        entries.len()
    );

    // Use unified selector system to process entries
    let entry_refs: Vec<&Entry> = entries.iter().collect();
    let formatted_command = process_entries(&entry_refs, template)?;

    debug!("Final command: {:?}", formatted_command);
    Ok(formatted_command)
}

/// Execute an external action with the appropriate execution mode and output handling
///
/// Takes an `ActionSpec` and a slice of entries, creates a command using the action's template,
/// and executes the resulting command with the specified execution mode.
///
/// # Arguments
/// * `action_spec` - The `ActionSpec` containing the command template, execution mode, and output mode
/// * `entries` - A reference to a slice of Entry items to process
///
/// # Returns
/// * `Result<ExitStatus>` - The exit status of the executed command
///
/// # Behavior
/// - `ExecutionMode::Execute` - make the current process become what the command does
/// - `ExecutionMode::Fork` - spawns the command as a child process
pub fn execute_action(
    action_spec: &ActionSpec,
    entries: &[Entry],
) -> Result<ExitStatus> {
    debug!("Executing external action with {} entries", entries.len());

    let template: &Template = action_spec.command.get_nth(0);
    let formatted_command = format_command(entries, template)?;

    let mut cmd = shell_command(
        &formatted_command,
        action_spec.command.interactive,
        &action_spec.command.env,
    );

    match action_spec.mode {
        ExecutionMode::Execute => {
            let err = cmd.exec();
            eprintln!("Failed to execute command: {}", err);
            Err(err.into())
        }
        ExecutionMode::Fork => {
            cmd.stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());

            let mut child = cmd.spawn()?;
            Ok(child.wait()?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channels::entry::Entry;
    use crate::channels::prototypes::Template;
    use crate::selector::SelectorMode;

    #[test]
    fn test_simple_braces_syntactic_sugar() {
        let entries = vec![Entry::new("file1.txt".to_string())];

        // Simple braces should use syntactic sugar with quotes
        let template = Template::parse("nvim {}").unwrap();
        let result = format_command(&entries, &template).unwrap();
        assert_eq!(result, "nvim file1.txt");
    }

    #[test]
    fn test_simple_braces_multiple_entries() {
        let entries = vec![
            Entry::new("file1.txt".to_string()),
            Entry::new("file 2.txt".to_string()),
        ];

        // Simple braces with multiple entries should quote when needed and join with spaces
        let mut template = Template::parse("nvim {}").unwrap();
        template.mode = SelectorMode::Concatenate;
        template.separator = " ".to_string();
        template.shell_escaping = true;
        let result = format_command(&entries, &template).unwrap();

        // Result should contain both files quoted when needed and joined with space
        assert_eq!(result, "nvim file1.txt 'file 2.txt'");
    }

    #[test]
    fn test_simple_braces_with_quotes_in_filename() {
        let entries = vec![Entry::new("file's name.txt".to_string())];

        // Simple braces should escape single quotes in filenames
        let mut template = Template::parse("nvim {}").unwrap();
        template.mode = SelectorMode::Concatenate;
        template.separator = " ".to_string();
        template.shell_escaping = true;
        let result = format_command(&entries, &template).unwrap();
        assert_eq!(result, "nvim \"file's name.txt\"");
    }

    #[test]
    fn test_complex_braces_use_template_system() {
        let entries = vec![
            Entry::new("file1.txt".to_string()),
            Entry::new("file2.txt".to_string()),
        ];

        // Complex braces should use template system
        let mut template = Template::parse(
            "nvim {split:\\n:..|map:{append:'|prepend:'}|sort|join: }",
        )
        .unwrap();
        template.mode = SelectorMode::Concatenate;
        template.separator = " ".to_string();
        template.shell_escaping = true;
        let result = format_command(&entries, &template).unwrap();

        // Result should contain both files quoted and joined with space
        assert_eq!(result, "nvim 'file1.txt' 'file2.txt'");
    }

    #[test]
    fn test_complex_braces_use_template_system_with_quotes_in_filename() {
        let entries = vec![
            Entry::new("file1's.txt".to_string()),
            Entry::new("file2.txt".to_string()),
        ];

        // Complex braces should use template system
        let mut template = Template::parse(
            r"nvim {split:\n:..|map:{replace:s/'/\'/g|append:'|prepend:'}|sort|join: }",
        ).unwrap();
        template.mode = SelectorMode::Concatenate;
        template.separator = " ".to_string();
        template.shell_escaping = true;
        let result = format_command(&entries, &template).unwrap();

        // Result should contain both files quoted correctly
        // Note: Order may vary due to hash set ordering, and quoting mechanism uses double quotes
        assert!(result.contains("nvim"));
        assert!(result.contains("\"file1\\'s.txt\""));
        assert!(result.contains("file2.txt"));
    }
}
