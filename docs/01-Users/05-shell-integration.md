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

## Configuring autocompletion

Shell integration works by setting a dedicated shell keybinding that launches `tv` with the current prompt buffer so that `tv` may guess which channel (builtin or cable) is the most appropriate.

Which channel gets effectively chosen for different commands can be tweaked in the `shell_integration` section of the [configuration file](https://github.com/alexpasmantier/television/wiki/Configuration-file):

```toml
[shell_integration.channel_triggers]
"env" = ["export", "unset"]
"dirs" = ["cd", "ls", "rmdir"]
"files" = ["mv", "cp", "vim"]
```

Each key is a channel name and each value is a set of commands that should trigger that channel.

Example: say you want the following prompts to trigger the following channels when pressing <kbd>CTRL-T</kbd>`:

- `git checkout` should trigger the `git-branches` channel
- `ls` should trigger the `dirs` channel
- `cat` and `nano` should trigger the `files` channel

You would add the following to your configuration file:

```toml
[shell_integration.channel_triggers]
"git-branches" = ["git checkout"]
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

_Note:_ Remember to remove the line added in [Enabling Shell Integration](https://github.com/alexpasmantier/television/wiki/Shell-Autocompletion#enabling-shell-integration) to avoid sourcing the file twice.

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

For all shells you'll have to restart it (or similar) to integrate the changes.

### Recipes

#### Automatically executing selection

Edit the `~/.config/television/shell/integration.zsh` file and add the following:

```zsh
_tv_search() {
    emulate -L zsh
    zle -I

    local current_prompt
    current_prompt=$LBUFFER

    local output

    output=$(tv --autocomplete-prompt "$current_prompt" $*)

    zle reset-prompt

    if [[ -n $output ]]; then
        RBUFFER=""
        LBUFFER=$current_prompt$output

        # uncomment this to automatically accept the line
        # (i.e. run the command without having to press enter twice)
        # zle accept-line
    fi
}


zle -N tv-search _tv_search


bindkey '^T' tv-search
```

Note: Uncommenting `zle accept-line` below will automatically execute the command when accepting a suggestion

#### Open history channel with the most up to date version of the history file

Edit the `~/.config/television/shell/integration.bash` file and replace the `tv_shell_history` function with the following (or rename to keep the default implementation):

```bash
function tv_shell_history() {
  local current_prompt="${READLINE_LINE:0:$READLINE_POINT}"

  local output=$(history -n && history -a && tv bash-history --input "$current_prompt")

  if [[ -n $output ]]; then
    READLINE_LINE=$output
    READLINE_POINT=${#READLINE_LINE}
  fi
}
# history -n  to read the current file, in case other sessions wrote some commands
# history -a  to commit the current one
```

**WARNING:** committing the current history to file could have unintended consequences as a default, for example if the user was planning to run `history -c` to clear the current session (perhaps some commands have sensitive information)
