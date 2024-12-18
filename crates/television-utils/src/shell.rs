use color_eyre::Result;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Cmd,
}

const COMPLETION_ZSH: &str = include_str!("../../../shell/completion.zsh");

pub fn completion_script(shell: Shell) -> Result<&'static str> {
    match shell {
        Shell::Zsh => Ok(COMPLETION_ZSH),
        _ => color_eyre::eyre::bail!(
            "This shell is not yet supported: {:?}",
            shell
        ),
    }
}
