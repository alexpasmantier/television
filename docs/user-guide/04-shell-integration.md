# Shell Integration

Television can integrate with your shell to provide smart autocompletion based on the commands you start typing.

![tv-shell-integration](https://github.com/user-attachments/assets/6292db26-8fcf-4874-ac9d-c9baedc70ff1)

## Keybindings

- <kbd>Ctrl</kbd>-<kbd>R</kbd>: shell history
- <kbd>Ctrl</kbd>-<kbd>T</kbd>: smart autocompletion for the current prompt command

## Enabling shell integration

### Zsh

To enable shell integration for zsh, run:

```bash
echo 'eval "$(tv init zsh)"' >> ~/.zshrc
```

And then restart your shell or run:

```bash
source ~/.zshrc
```

### Bash

To enable shell integration for bash, run:

```bash
echo 'eval "$(tv init bash)"' >> ~/.bashrc
```

And then restart your shell or run:

```bash
source ~/.bashrc
```

### Fish

To enable shell integration for fish, add:

```bash
tv init fish | source
```

to your `is-interactive` block in your `~/.config/fish/config.fish` file and then restart your shell.

### Nushell

To enable shell integration for nu, add this to your `~/.config/nushell/config.nu` file:

```nu
mkdir ($nu.data-dir | path join "vendor/autoload")
tv init nu | save -f ($nu.data-dir | path join "vendor/autoload/tv.nu")
```

### PowerShell

To enable shell integration for PowerShell, run:

```powershell
tv init power-shell | Out-File -FilePath $PROFILE -Append
```

This generates the shell integration script and appends it to your PowerShell profile so that it is loaded at startup.

To find your PowerShell profile location, run:

```powershell
$PROFILE
```

If the profile doesn't exist yet, you can create it with:

```powershell
New-Item -Path $PROFILE -Type File -Force
```

## Configuring autocompletion

Shell integration works by setting a dedicated shell keybinding that launches `tv` with the current prompt buffer so that `tv` may guess which channel (builtin or cable) is the most appropriate.

Which channel gets effectively chosen for different commands can be tweaked in the `shell_integration` section of the [configuration file](./02-configuration.md):

```toml
[shell_integration.channel_triggers]
"env" = ["export", "unset"]
"dirs" = ["cd", "ls", "rmdir"]
"files" = ["mv", "cp", "vim"]
```

Each key is a channel name and each value is a set of commands that should trigger that channel.

Example: say you want the following prompts to trigger the following channels when pressing <kbd>CTRL-T</kbd>:

- `git checkout` should trigger the `git-branch` channel
- `ls` should trigger the `dirs` channel
- `cat` and `nano` should trigger the `files` channel

You would add the following to your configuration file:

```toml
[shell_integration.channel_triggers]
"git-branch" = ["git checkout"]
"dirs" = ["ls"]
"files" = ["cat", "nano"]
```

## Customizing shell integration scripts

### Setting up the files

To customize the default behavior of the shell integration scripts you can save them locally and source that file instead:

Run the following command to make sure the destination directory exists, you can also store them wherever you like

```shell
mkdir -p ~/.config/television/shell
```

_Note:_ Remember to remove the line added in the "Enabling shell integration" section to avoid sourcing the file twice.

#### Zsh

```shell
tv init zsh > ~/.config/television/shell/integration.zsh
echo 'source $HOME/.config/television/shell/integration.zsh' >> ~/.zshrc
```

#### Bash

```shell
tv init bash > ~/.config/television/shell/integration.bash
echo 'source $HOME/.config/television/shell/integration.bash' >> ~/.bashrc
```

#### Fish

```shell
tv init fish > ~/.config/television/shell/integration.fish
```

Then add to your `is-interactive` block in your `~/.config/fish/config.fish` file.

```fish
source $HOME/.config/television/shell/integration.fish
```

#### PowerShell

```powershell
tv init power-shell | Out-File -FilePath ~/.config/television/shell/integration.ps1
```

Then add to your PowerShell profile:

```powershell
. $HOME/.config/television/shell/integration.ps1
```

For all shells you'll have to restart it (or similar) to integrate the changes.

### Recipes

#### Automatically executing selection

Edit the `~/.config/television/shell/integration.zsh` file.

For history search, uncomment the `zle accept-line` line at the end of `_tv_shell_history`:

```zsh
    if [[ -n $output ]]; then
        RBUFFER=""
        LBUFFER=$(echo "$output")
        zle accept-line
    fi
```

For smart autocompletion, add `zle accept-line` after the `__tv_path_completion` call in `_tv_smart_autocomplete`:

```zsh
  __tv_path_completion "$prefix" "$lbuf"
  zle accept-line
```

Note: this runs the command as soon as you accept a suggestion, without pressing enter a second time.

#### Open history channel with the most up to date version of the history file

Edit the `~/.config/television/shell/integration.bash` file and replace the `output=` line in `tv_shell_history` with:

```bash
    output=$(history -n && history -a && tv bash-history --no-status-bar --input "$current_prompt" --inline)
```

`history -n` reads commands written by other sessions and `history -a` commits the current session to the history file before `tv` reads it.

**WARNING:** committing the current history to file could have unintended consequences as a default, for example if the user was planning to run `history -c` to clear the current session (perhaps some commands have sensitive information)
