<div align="center">

# Television (tv)

**A very fast, portable and hackable fuzzy finder for the terminal.**

![GitHub Release](https://img.shields.io/github/v/release/alexpasmantier/television?display_name=tag&color=%23a6a)
![docs.rs](https://img.shields.io/docsrs/television-channels)
![GitHub branch check runs](https://img.shields.io/github/check-runs/alexpasmantier/television/main)
![GitHub License](https://img.shields.io/github/license/alexpasmantier/television)
[![Discord](https://img.shields.io/discord/1366133668535341116?logo=discord)](https://discord.gg/hQBrzsJgUg)

<img width="1694" height="1005" alt="2025-12-18-013816_hyprshot" src="https://github.com/user-attachments/assets/d512a358-5d36-48f7-b40e-695c644c75b7" />


</div>

## About

`Television` is a very fast, portable and hackable fuzzy finder for the terminal.

It lets you search in real time through any kind of data source (called "channels") such as:

- files and directories
- code
- notes
- processes
- git repositories
- environment variables
- docker containers
- ...and much more ([creating your own channels](https://alexpasmantier.github.io/television/docs/Users/channels/#creating-your-own-channels))

with support for previewing results, customizable actions and keybindings, and integration with your favorite
shell and editor.

## Getting started with tv

tv uses `channels` to define different sources of data to browse and preview. It comes with several built-in channels for common tasks like browsing files, searching text, and viewing git repositories.

```sh
tv            # uses the default channel (usually "files")
tv files      # browse files and directories
tv text       # ripgrep-powered text search
tv git-repos  # browse git repositories
```

To get a list of available channels, run:

```sh
tv list-channels
```

To pull in the latest community channels from the github repo, run:

```sh
tv update-channels
```

You can also pipe output into tv to search through command results, logs, or any
stream of text:

```sh
rg "ERROR" /var/log/syslog | tv
git log --oneline | tv
my_program_that_generates_logs | tv
```

And if you need a one-off channel for a specific task, tv's command line options let you create temporary channels on the fly:

```sh
tv --source-command "rg --line-number --no-heading TODO ."
tv --source-command "fd -t f" --preview-command "bat -n --color=always '{}'" --preview-size 70
```

## Custom channels

You can create custom channels for any specific task you want to do regularly. Channels are defined using TOML files that specify how to get the data, how to preview it, and any keybindings or actions you want to add.

### Example: TLDR pages channel
Create a channel: _~/.config/television/cable/tldr.toml_

```toml
[metadata]
name = "tldr"
description = "Browse and preview TLDR help pages for command-line tools"
requirements = ["tldr"]

[source]
command = "tldr --list"

[preview]
command = "tldr '{0}'"

[keybindings]
ctrl-e = "actions:open"

[actions.open]
description = "Open the selected TLDR page"
command = "tldr '{0}'"
mode = "execute"
```

Start searching:

```sh
tv tldr
```

Switch channels using the remote control and pick from a large choice of [community-maintained channels](https://alexpasmantier.github.io/television/docs/Users/community-channels-unix):

![tv remote](./assets/tv-files-remote.png)

See the [channels docs](https://alexpasmantier.github.io/television/docs/Users/channels/) for more info on how to set these up.

## Installation

1. [Automatically select the best installation method](#automatically-select-the-best-installation-method)
2. [Linux](#linux)
3. [MacOS](#macos)
4. [Windows](#windows)
5. [NetBSD](#netbsd)
6. [Cross-platform](#cross-platform)
7. [Precompiled binaries](#precompiled-binaries)

### Automatically select the best installation method
Running the following command will detect your OS and install `television` using the best available method:

```sh
curl -fsSL https://alexpasmantier.github.io/television/install.sh | bash
```

### Linux

- [Arch Linux](https://archlinux.org/), [Manjaro](https://manjaro.org/), [EndeavourOS](https://endeavouros.com/), etc.:

```sh
pacman -S television
```

- [Debian](https://www.debian.org/), [Ubuntu](https://ubuntu.com/), [Linux Mint](https://linuxmint.com/),
[Pop!_OS](https://pop.system76.com/), etc.:

```sh
VER=`curl -s "https://api.github.com/repos/alexpasmantier/television/releases/latest" | grep '"tag_name":' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/'`
curl -LO https://github.com/alexpasmantier/television/releases/download/$VER/tv-$VER-x86_64-unknown-linux-musl.deb
echo $VER
sudo dpkg -i tv-$VER-x86_64-unknown-linux-musl.deb
```

- [ChimeraOS](https://chimeraos.org/), [Alpine Linux](https://alpinelinux.org/):

```sh
apk add chimera-repo-user
apk add television
```

- [NixOS](https://nixos.org/) and other systems with [Nix package manager](https://nixos.org/download.html):

```sh
nix run nixpkgs#television
```

### MacOS
- [Homebrew](https://brew.sh/):

```sh
brew install television
```

### Windows
- [Scoop](https://scoop.sh/):

```sh
scoop bucket add extras
scoop install television
```

- [Winget](https://github.com/microsoft/winget-cli):

```sh
winget install --exact --id alexpasmantier.television
```

### NetBSD

- [pkgsrc](https://pkgsrc.se/textproc/television):

```sh
pkgin install television
```


### Cross-platform

- [Cargo](https://doc.rust-lang.org/cargo/):

```sh
cargo install television
```

- [Conda-forge](https://anaconda.org/conda-forge/television):

```sh
pixi global install television
```

### Precompiled binaries

Download the latest release from the [releases page](https://www.github.com/alexpasmantier/television/releases).

## Usage

```bash
tv  # default channel

tv [channel]  # e.g. `tv files`, `tv env`, `tv git-repos`, `tv my-awesome-channel` etc.

# pipe the output of your program into tv
my_program | tv

fd -t f . | tv --preview-command 'bat -n --color=always {}'

# or build your own channel on the fly
tv --source-command 'fd -t f .' --preview-command 'bat -n --color=always {}' --preview-size 70
```

> [!TIP]
> 🐚 _Television has **builtin shell integration**. More info [here](https://alexpasmantier.github.io/television/docs/Users/shell-integration)._

For more information, check out the [docs](https://alexpasmantier.github.io/television/).

## Using tv inside your favorite editor

- **Neovim**: [tv.nvim](https://github.com/alexpasmantier/tv.nvim) (lua)
- **Vim**: [tv.vim](https://github.com/prabirshrestha/tv.vim) (vimscript)
- **Zed**: [Easy Telescope-style file finder in Zed using television](https://zed.dev/blog/hidden-gems-part-2#emulate-vims-telescope-via-television)
- **VSCode**: [using television as a file picker inside vscode](https://marketplace.visualstudio.com/items?itemName=alexpasmantier.television)

## Credits

This project was inspired by the **awesome** work done by the [telescope](https://github.com/nvim-telescope/telescope.nvim) neovim plugin.

It also leverages the great [helix](https://github.com/helix-editor/helix) editor's nucleo fuzzy matching library, the [tokio](https://github.com/tokio-rs/tokio) async runtime as well as the **formidable** [ratatui](https://github.com/ratatui/ratatui) library.

A special thanks to tv's contributors for their help and support:

<a href="https://github.com/alexpasmantier/television/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=alexpasmantier/television" />
</a>
