use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Cmd,
}

const COMPLETION_ZSH: &str = include_str!("shell/completion.zsh");
const COMPLETION_BASH: &str = include_str!("shell/completion.bash");
const COMPLETION_FISH: &str = include_str!("shell/completion.fish");

// create the appropriate key binding for each supported shell
pub fn ctrl_keybinding(shell: Shell, character: char) -> Result<String> {
    match shell {
        Shell::Bash => Ok(format!(r"\C-{}", character)),
        Shell::Zsh => Ok(format!(r"^{}", character)),
        Shell::Fish => Ok(format!(r"\c{}", character)),
        _ => anyhow::bail!("This shell is not yet supported: {:?}", shell),
    }
}

pub fn completion_script(shell: Shell) -> Result<&'static str> {
    match shell {
        Shell::Bash => Ok(COMPLETION_BASH),
        Shell::Zsh => Ok(COMPLETION_ZSH),
        Shell::Fish => Ok(COMPLETION_FISH),
        _ => anyhow::bail!("This shell is not yet supported: {:?}", shell),
    }
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
}
