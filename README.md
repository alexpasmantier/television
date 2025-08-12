<div align="center">

[<img src="./assets/television-title.png">](https://alexpasmantier.github.io/television/)  
**A cross-platform, fast and extensible general purpose fuzzy finder for the terminal.**

![GitHub Release](https://img.shields.io/github/v/release/alexpasmantier/television?display_name=tag&color=%23a6a)
![docs.rs](https://img.shields.io/docsrs/television-channels)
![GitHub branch check runs](https://img.shields.io/github/check-runs/alexpasmantier/television/main)
![GitHub License](https://img.shields.io/github/license/alexpasmantier/television)
[![Discord](https://img.shields.io/discord/1366133668535341116?logo=discord)](https://discord.gg/hQBrzsJgUg)

![tv's files channel](./assets/tv-transparent.png)

</div>

## About

`Television` is a cross-platform, fast and extensible fuzzy finder for the terminal.

It integrates with your shell and lets you quickly search through any kind of data source (files, git repositories, environment variables, docker
images, you name it) using a fuzzy matching algorithm and is designed to be extensible.

It is inspired by the neovim [telescope](https://github.com/nvim-telescope/telescope.nvim) plugin and leverages [tokio](https://github.com/tokio-rs/tokio) and the [nucleo](https://github.com/helix-editor/nucleo) matcher used by the [helix](https://github.com/helix-editor/helix) editor to ensure optimal performance.

## Installation

See [installation docs](https://alexpasmantier.github.io/television/docs/Users/installation).

## TL;DR

Create a channel: _~/.config/television/cable/files.toml_

```toml
[metadata]
name = "files"
description = "A channel to select files and directories"
requirements = ["fd", "bat"]

[source]
command = "fd -t f"

[preview]
command = "bat -n --color=always '{}'"
env = { BAT_THEME = "Catppuccin Mocha" }

[ui]
preview_panel = { "size" = 70, "scrollbar" = true }

[keybindings]
shortcut = "f1"
f12 = "actions:edit"
f11 = ["actions:rm", "reload_source"]

[actions.edit]
description = "Opens the selected entries with the default editor (falls back to vim)"
command = "${EDITOR:-vim} {}"
mode = "execute"

[actions.rm]
description = "Removes the selected entries"
command = "rm {}"
```

Start searching:

```sh
tv files
```

![tv files](./assets/tv-transparent.png)

Switch channels using the remote control and pick from a list of [community-maintained channels](https://alexpasmantier.github.io/television/docs/Users/community-channels-unix) which
you can install with `tv update-channels`:

![tv remote](./assets/tv-files-remote.png)

See the [channels docs](https://alexpasmantier.github.io/television/docs/Users/channels) for more info on how to set these up.

## Usage

```bash
tv  # default channel

tv [channel]  # e.g. `tv files`, `tv env`, `tv git-repos`, etc.

# pipe the output of your program into tv
my_program | tv

fd -t f . | tv --preview 'bat -n --color=always {}'

# or build your own channel on the fly
tv --source-command 'fd -t f .' --preview-command 'bat -n --color=always {}' --preview-size 70
```

> [!TIP]
> 🐚 _Television has **builtin shell integration**. More info [here](https://alexpasmantier.github.io/television/docs/Users/shell-integration)._

## Credits

This project was inspired by the **awesome** work done by the [telescope](https://github.com/nvim-telescope/telescope.nvim) neovim plugin.

It also leverages the great [helix](https://github.com/helix-editor/helix) editor's nucleo fuzzy matching library, the [tokio](https://github.com/tokio-rs/tokio) async runtime as well as the **formidable** [ratatui](https://github.com/ratatui/ratatui) library.
