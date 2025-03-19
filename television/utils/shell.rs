use crate::cli::args::Shell as CliShell;
use crate::config::shell_integration::ShellIntegrationConfig;
use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Cmd,
    Nu,
}

impl From<CliShell> for Shell {
    fn from(val: CliShell) -> Self {
        match val {
            CliShell::Bash => Shell::Bash,
            CliShell::Zsh => Shell::Zsh,
            CliShell::Fish => Shell::Fish,
            CliShell::PowerShell => Shell::PowerShell,
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
            CliShell::PowerShell => Shell::PowerShell,
            CliShell::Cmd => Shell::Cmd,
            CliShell::Nu => Shell::Nu,
        }
    }
}

const COMPLETION_ZSH: &str = include_str!("shell/completion.zsh");
const COMPLETION_BASH: &str = include_str!("shell/completion.bash");
const COMPLETION_FISH: &str = include_str!("shell/completion.fish");
const COMPLETION_NU: &str = include_str!("shell/completion.nu");

// create the appropriate key binding for each supported shell
pub fn ctrl_keybinding(shell: Shell, character: char) -> Result<String> {
    match shell {
        Shell::Bash => Ok(format!(r"\C-{character}")),
        Shell::Zsh => Ok(format!(r"^{character}")),
        Shell::Fish => Ok(format!(r"\c{character}")),
        Shell::Nu => Ok(format!(r"Ctrl-{character}")),
        _ => anyhow::bail!("This shell is not yet supported: {:?}", shell),
    }
}

pub fn completion_script(shell: Shell) -> Result<&'static str> {
    match shell {
        Shell::Bash => Ok(COMPLETION_BASH),
        Shell::Zsh => Ok(COMPLETION_ZSH),
        Shell::Fish => Ok(COMPLETION_FISH),
        Shell::Nu => Ok(COMPLETION_NU),
        _ => anyhow::bail!("This shell is not yet supported: {:?}", shell),
    }
}

pub fn render_autocomplete_script_template(
    shell: Shell,
    template: &str,
    config: &ShellIntegrationConfig,
) -> Result<String> {
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
    Ok(script)
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
        assert_eq!(result.unwrap(), "\\cs");
    }

    #[test]
    fn test_powershell_ctrl_keybinding() {
        let character = 's';
        let shell = Shell::PowerShell;
        let result = ctrl_keybinding(shell, character);
        assert!(result.is_err());
    }

    #[test]
    fn test_nushell_ctrl_keybinding() {
        let character = 's';
        let shell = Shell::Nu;
        let result = ctrl_keybinding(shell, character);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Ctrl-s");
    }
}
