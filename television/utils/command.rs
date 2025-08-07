use crate::{
    channels::{
        entry::Entry,
        prototypes::{ActionSpec, ExecutionMode, Template},
    },
    utils::shell::Shell,
};
use anyhow::Result;
use lazy_regex::{Lazy, Regex, regex};
use rustc_hash::FxHashSet;
use std::{
    collections::HashMap,
    os::unix::process::CommandExt,
    process::{Command, ExitStatus, Stdio},
};
use tracing::debug;

static COMPLEX_BRACES_REGEX: &Lazy<Regex> = regex!(r"\{[^}]+\}");

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

    let template_str = template.raw();

    // Check if template has only simple braces (syntactic sugar)
    let has_only_simple_braces = !COMPLEX_BRACES_REGEX.is_match(template_str);
    if has_only_simple_braces {
        // Handle simple braces with predictable multi-value logic
        debug!(
            "Using simple brace syntactic sugar for template: {}",
            template_str
        );

        // Multiple entries: quote each and join with spaces
        let quoted_entries: Vec<String> = entries
            .iter()
            .map(|entry| format!("'{}'", entry.raw.replace('\'', r"\'")))
            .collect();
        let entries_joined = quoted_entries.join(" ");
        let formatted_command = template_str.replace("{}", &entries_joined);
        debug!("Multiple entries command: {:?}", formatted_command);
        Ok(formatted_command)
    } else {
        // Complex braces: use existing template processing
        debug!("Using complex template processing for: {}", template_str);

        // Concatenate entries with separator for template processing
        let entries_str = entries
            .iter()
            .map(|entry| entry.raw.as_str())
            .collect::<Vec<_>>()
            .join(separator);
        debug!("Concatenated entries input: {:?}", entries_str);

        // Process through template system
        let formatted_command = template.format(&entries_str)?;
        debug!("Final command: {:?}", formatted_command);
        Ok(formatted_command)
    }
}

/// Execute an external action with the appropriate execution mode and output handling
///
/// Takes an `ActionSpec` and a set of entries, creates a command using the action's template,
/// and executes the resulting command with the specified execution mode.
///
/// # Arguments
/// * `action_spec` - The `ActionSpec` containing the command template, execution mode, and output mode
/// * `entries` - A reference to a set of Entry items to process
///
/// # Returns
/// * `Result<ExitStatus>` - The exit status of the executed command
///
/// # Behavior
/// - `ExecutionMode::Execute` - make the current process become what the command does
/// - `ExecutionMode::Fork` - spawns the command as a child process
pub fn execute_action(
    action_spec: &ActionSpec,
    entries: &FxHashSet<Entry>,
) -> Result<ExitStatus> {
    debug!("Executing external action with {} entries", entries.len());

    let template: &Template = action_spec.command.get_nth(0);
    let formatted_command =
        format_command(entries, template, &action_spec.separator)?;

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

    #[test]
    fn test_simple_braces_syntactic_sugar() {
        let mut entries = FxHashSet::default();
        entries.insert(Entry::new("file1.txt".to_string()));

        // Simple braces should use syntactic sugar with quotes
        let template = Template::parse("nvim {}").unwrap();
        let result = format_command(&entries, &template, "\n").unwrap();
        assert_eq!(result, "nvim 'file1.txt'");
    }

    #[test]
    fn test_simple_braces_multiple_entries() {
        let mut entries = FxHashSet::default();
        entries.insert(Entry::new("file1.txt".to_string()));
        entries.insert(Entry::new("file2.txt".to_string()));

        // Simple braces with multiple entries should quote each and join with spaces
        let template = Template::parse("nvim {}").unwrap();
        let result = format_command(&entries, &template, "\n").unwrap();

        // Result should contain both files quoted and joined with space
        assert_eq!(result, "nvim 'file1.txt' 'file2.txt'");
    }

    #[test]
    fn test_simple_braces_with_quotes_in_filename() {
        let mut entries = FxHashSet::default();
        entries.insert(Entry::new("file's name.txt".to_string()));

        // Simple braces should escape single quotes in filenames
        let template = Template::parse("nvim {}").unwrap();
        let result = format_command(&entries, &template, "\n").unwrap();
        assert_eq!(result, "nvim 'file\\'s name.txt'");
    }

    #[test]
    fn test_complex_braces_use_template_system() {
        let mut entries = FxHashSet::default();
        entries.insert(Entry::new("file1.txt".to_string()));
        entries.insert(Entry::new("file2.txt".to_string()));

        // Complex braces should use template system
        let template = Template::parse(
            "nvim {split:\\n:..|map:{append:'|prepend:'}|sort|join: }",
        )
        .unwrap();
        let result = format_command(&entries, &template, "\n").unwrap();

        // Result should contain both files quoted and joined with space
        assert_eq!(result, "nvim 'file1.txt' 'file2.txt'");
    }

    #[test]
    fn test_complex_braces_use_template_system_with_quotes_in_filename() {
        let mut entries = FxHashSet::default();
        entries.insert(Entry::new("file1's.txt".to_string()));
        entries.insert(Entry::new("file2.txt".to_string()));

        // Complex braces should use template system
        let template = Template::parse(
            r"nvim {split:\n:..|map:{replace:s/'/\'/g|append:'|prepend:'}|sort|join: }",
        )
        .unwrap();
        let result = format_command(&entries, &template, "\n").unwrap();

        // Result should be escaped with single quotes in filenames
        assert_eq!(result, "nvim 'file1\\'s.txt' 'file2.txt'");
    }
}
