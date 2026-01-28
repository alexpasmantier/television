use crate::{
    cli::args::{Cli, Shell as CliShell},
    config::shell_integration::ShellIntegrationConfig,
};
use anyhow::Result;
use clap::CommandFactory;
use std::fmt::Display;
use tracing::{debug, warn};

#[derive(
    Debug, Clone, Copy, PartialEq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    #[serde(rename = "powershell")]
    Psh,
    Cmd,
    Nu,
}

impl Default for Shell {
    #[cfg(unix)]
    fn default() -> Self {
        Shell::Bash
    }

    #[cfg(windows)]
    fn default() -> Self {
        Shell::Psh
    }
}

#[derive(Debug)]
pub enum ShellError {
    UnsupportedShell(String),
}

impl TryFrom<Shell> for clap_complete::Shell {
    type Error = ShellError;

    fn try_from(value: Shell) -> std::result::Result<Self, Self::Error> {
        match value {
            Shell::Bash => Ok(clap_complete::Shell::Bash),
            Shell::Zsh => Ok(clap_complete::Shell::Zsh),
            Shell::Fish => Ok(clap_complete::Shell::Fish),
            Shell::Psh => Ok(clap_complete::Shell::PowerShell),
            Shell::Cmd => Err(ShellError::UnsupportedShell(
                "Cmd shell is not supported for completion scripts"
                    .to_string(),
            )),
            Shell::Nu => Err(ShellError::UnsupportedShell(
                "Nu shell is not supported for completion scripts".to_string(),
            )),
        }
    }
}

impl Display for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Shell::Bash => write!(f, "bash"),
            Shell::Zsh => write!(f, "zsh"),
            Shell::Fish => write!(f, "fish"),
            Shell::Psh => write!(f, "powershell"),
            Shell::Cmd => write!(f, "cmd"),
            Shell::Nu => write!(f, "nu"),
        }
    }
}

const SHELL_ENV_VAR: &str = "SHELL";

impl TryFrom<&str> for Shell {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        if value.contains("bash") {
            Ok(Shell::Bash)
        } else if value.contains("zsh") {
            Ok(Shell::Zsh)
        } else if value.contains("fish") {
            Ok(Shell::Fish)
        } else if value.contains("powershell") {
            Ok(Shell::Psh)
        } else if value.contains("cmd") {
            Ok(Shell::Cmd)
        } else if value.contains("nu") {
            Ok(Shell::Nu)
        } else {
            Err(anyhow::anyhow!("Unsupported shell: {}", value))
        }
    }
}

impl Shell {
    #[allow(clippy::borrow_interior_mutable_const)]
    #[cfg(unix)]
    pub fn from_env() -> Result<Self> {
        if let Ok(shell) = std::env::var(SHELL_ENV_VAR) {
            Shell::try_from(shell.as_str())
        } else {
            debug!("Environment variable {} not set", SHELL_ENV_VAR);
            Ok(Shell::default())
        }
    }

    #[cfg(windows)]
    /// There isn't a straightforward way of doing this on Windows so we poke around
    /// to make an educated guess.
    pub fn from_env() -> Result<Self> {
        // If SHELL is set, try to use it
        if let Ok(shell) = std::env::var(SHELL_ENV_VAR) {
            return Shell::try_from(shell.as_str());
        }
        if std::env::var("POWERSHELL_DISTRIBUTION_CHANNEL").is_ok()
            || std::env::var("PSModulePath").is_ok()
        {
            return Ok(Shell::Psh);
        }
        // Otherwise, fall back to COMSPEC
        if let Ok(shell) = std::env::var("COMSPEC") {
            if shell.to_lowercase().contains("powershell") {
                return Ok(Shell::Psh);
            } else if shell.to_lowercase().contains("cmd") {
                return Ok(Shell::Cmd);
            }
        }
        debug!("Environment variable COMSPEC not set");
        Ok(Shell::default())
    }

    pub fn executable(&self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::Psh => "powershell",
            Shell::Cmd => "cmd",
            Shell::Nu => "nu",
        }
    }
}

impl From<CliShell> for Shell {
    fn from(val: CliShell) -> Self {
        match val {
            CliShell::Bash => Shell::Bash,
            CliShell::Zsh => Shell::Zsh,
            CliShell::Fish => Shell::Fish,
            CliShell::PowerShell => Shell::Psh,
            CliShell::Cmd => Shell::Cmd,
            CliShell::Nu => Shell::Nu,
        }
    }
}

impl From<&CliShell> for Shell {
    fn from(val: &CliShell) -> Self {
        match val {
            CliShell::Bash => Shell::Bash,
            CliShell::Zsh => Shell::Zsh,
            CliShell::Fish => Shell::Fish,
            CliShell::PowerShell => Shell::Psh,
            CliShell::Cmd => Shell::Cmd,
            CliShell::Nu => Shell::Nu,
        }
    }
}

const COMPLETION_ZSH: &str = include_str!("shell/completion.zsh");
const COMPLETION_BASH: &str = include_str!("shell/completion.bash");
const COMPLETION_FISH: &str = include_str!("shell/completion.fish");
const COMPLETION_NU: &str = include_str!("shell/completion.nu");
const COMPLETION_POWERSHELL: &str = include_str!("shell/completion.ps1");

// create the appropriate key binding for each supported shell
pub fn ctrl_keybinding(shell: Shell, character: char) -> Result<String> {
    match shell {
        Shell::Bash => Ok(format!(r"\C-{character}")),
        Shell::Zsh => Ok(format!(r"^{character}")),
        Shell::Fish => {
            if character == ' ' {
                Ok(r"ctrl-space".to_string())
            } else {
                let lower_char = character.to_ascii_lowercase();
                Ok(format!(r"ctrl-{lower_char}"))
            }
        }
        Shell::Nu => Ok(format!(r"Ctrl-{character}")),
        Shell::Psh => Ok(format!(r"Ctrl+{}", character.to_ascii_lowercase())),
        Shell::Cmd => {
            anyhow::bail!("This shell is not yet supported: {:?}", shell)
        }
    }
}

pub fn completion_script(shell: Shell) -> Result<&'static str> {
    match shell {
        Shell::Bash => Ok(COMPLETION_BASH),
        Shell::Zsh => Ok(COMPLETION_ZSH),
        Shell::Fish => Ok(COMPLETION_FISH),
        Shell::Nu => Ok(COMPLETION_NU),
        Shell::Psh => Ok(COMPLETION_POWERSHELL),
        Shell::Cmd => {
            anyhow::bail!("This shell is not yet supported: {:?}", shell)
        }
    }
}

pub fn render_autocomplete_script_template(
    shell: Shell,
    template: &str,
    config: &ShellIntegrationConfig,
) -> Result<String> {
    // Custom autocomplete
    let script = template
        .replace(
            "{tv_smart_autocomplete_keybinding}",
            &ctrl_keybinding(
                shell,
                config.get_shell_autocomplete_keybinding_character(),
            )?,
        )
        .replace(
            "{tv_shell_history_keybinding}",
            &ctrl_keybinding(
                shell,
                config.get_command_history_keybinding_character(),
            )?,
        );

    let clap_autocomplete =
        render_clap_autocomplete(shell).unwrap_or_default();

    Ok(clap_autocomplete + &script)
}

fn render_clap_autocomplete(shell: Shell) -> Option<String> {
    // Clap autocomplete
    let mut clap_autocomplete = vec![];
    let mut cmd = Cli::command();
    let clap_shell: clap_complete::Shell = match shell.try_into() {
        Ok(shell) => shell,
        Err(err) => {
            warn!("Failed to convert shell {:?}: {:?}", shell, err);
            return None;
        }
    };

    clap_complete::aot::generate(
        clap_shell,
        &mut cmd,
        "tv", // the command defines the name as "television"
        &mut clap_autocomplete,
    );

    String::from_utf8(clap_autocomplete).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bash_ctrl_keybinding() {
        let character = 's';
        let shell = Shell::Bash;
        let result = ctrl_keybinding(shell, character);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "\\C-s");
    }

    #[test]
    fn test_zsh_ctrl_keybinding() {
        let character = 's';
        let shell = Shell::Zsh;
        let result = ctrl_keybinding(shell, character);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "^s");
    }

    #[test]
    fn test_fish_ctrl_keybinding() {
        let character = 's';
        let shell = Shell::Fish;
        let result = ctrl_keybinding(shell, character);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "ctrl-s");
    }

    #[test]
    fn test_powershell_ctrl_keybinding() {
        let character = 's';
        let shell = Shell::Psh;
        let result = ctrl_keybinding(shell, character);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Ctrl+s");
    }

    #[test]
    fn test_powershell_ctrl_keybinding_uppercase_input() {
        // PowerShell keybindings should be lowercase even if input is uppercase
        let character = 'S';
        let shell = Shell::Psh;
        let result = ctrl_keybinding(shell, character);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Ctrl+s");
    }

    #[test]
    fn test_nushell_ctrl_keybinding() {
        let character = 's';
        let shell = Shell::Nu;
        let result = ctrl_keybinding(shell, character);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Ctrl-s");
    }

    #[test]
    fn test_zsh_clap_completion() {
        let shell = Shell::Zsh;
        let result = render_autocomplete_script_template(
            shell,
            "",
            &ShellIntegrationConfig::default(),
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.contains("compdef _tv tv"));
    }

    #[test]
    fn test_unsupported_clap_completion() {
        let shell = Shell::Nu;
        let result = render_autocomplete_script_template(
            shell,
            "",
            &ShellIntegrationConfig::default(),
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_powershell_clap_completion() {
        let shell = Shell::Psh;
        let result = render_autocomplete_script_template(
            shell,
            "",
            &ShellIntegrationConfig::default(),
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        // PowerShell completion should contain the completion function
        assert!(result.contains("Register-ArgumentCompleter"));
    }

    #[test]
    fn test_powershell_completion_script() {
        let shell = Shell::Psh;
        let result = completion_script(shell);
        assert!(result.is_ok());
        let script = result.unwrap();
        assert!(script.contains("Invoke-TvSmartAutocomplete"));
        assert!(script.contains("Invoke-TvShellHistory"));
        assert!(script.contains("Set-PSReadLineKeyHandler"));
    }
}
