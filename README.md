<div align="center">

[<img src="./assets/television-title.png">](https://alexpasmantier.github.io/television/)  
**A fast and hackable fuzzy finder for the terminal.**

![GitHub Release](https://img.shields.io/github/v/release/alexpasmantier/television?display_name=tag&color=%23a6a)
![docs.rs](https://img.shields.io/docsrs/television-channels)
![GitHub branch check runs](https://img.shields.io/github/check-runs/alexpasmantier/television/main)
![GitHub License](https://img.shields.io/github/license/alexpasmantier/television)
[![Discord](https://img.shields.io/discord/1366133668535341116?logo=discord)](https://discord.gg/hQBrzsJgUg)

![tv's files channel](./assets/tv-transparent.png)

</div>

## About

`Television` is a fast and hackable fuzzy finder for the terminal.

It lets you search in real time through any kind of data source (called "channels") such as:

- files and directories
- code
- notes
- processes
- git repositories
- environment variables
- docker containers
- ...and much more ([creating your own channels](https://alexpasmantier.github.io/television/docs/Users/channels/#creating-your-own-channels))

## TL;DR

Create a new channel: _~/.config/television/cable/files.toml_

```toml
[metadata]
name = "files"
description = "A channel to search through files and directories"
requirements = ["fd", "bat"]

[source]
command = "fd -t f"

[preview]
command = "bat -n --color=always '{}'"
env = { BAT_THEME = "Catppuccin Mocha" }
```

Start searching:

```sh
tv files
```

![tv files](./assets/tv-transparent.png)

Switch channels using the remote control and pick from a list of [community-maintained channels](https://alexpasmantier.github.io/television/docs/Users/community-channels-unix) or [create your own!](https://alexpasmantier.github.io/television/docs/Users/channels/#creating-your-own-channels).

![tv remote](./assets/tv-files-remote.png)

See the [channels docs](https://alexpasmantier.github.io/television/docs/Users/channels) for more info on how to set these up.

## Installation

### Automatic installation script:

```sh
curl -fsSL https://alexpasmantier.github.io/television/install.sh | bash
```

### Package managers:

[![Packaging status](https://repology.org/badge/vertical-allrepos/television.svg)](https://repology.org/project/television/versions)

#### MacOS:

- [Homebrew](https://brew.sh/):

```sh
brew install television
```

#### Linux:

- [Arch Linux](https://archlinux.org/):

```sh
pacman -S television
```

- Debian/Ubuntu:

```sh
VER=`curl -s "https://api.github.com/repos/alexpasmantier/television/releases/latest" | grep '"tag_name":' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/'`
curl -LO https://github.com/alexpasmantier/television/releases/download/$VER/tv-$VER-x86_64-unknown-linux-musl.deb
echo $VER
sudo dpkg -i tv-$VER-x86_64-unknown-linux-musl.deb
```

- Chimera Linux:

```sh
apk add chimera-repo-user
apk add television
```

- Nix:

```sh
nix run nixpkgs#television
```

#### Windows:

- [Scoop](https://scoop.sh/):

```sh
scoop bucket add extras
scoop install television
```

- [Winget](https://github.com/microsoft/winget-cli):

```sh
winget install --exact --id alexpasmantier.television
```

#### NetBSD:

- [pkgsrc](https://pkgsrc.se/textproc/television):

```sh
pkgin install television
```

#### Cross-platform:

- [Cargo](https://doc.rust-lang.org/cargo/):

```sh
cargo install television
```

- [Conda-forge](https://anaconda.org/conda-forge/television):

```sh
pixi global install television
```

### Precompiled binaries:

Download the latest release from the [releases page](https://www.github.com/alexpasmantier/television/releases).

## Usage

```bash
tv  # default channel

tv [channel]  # e.g. `tv files`, `tv env`, `tv git-repos`, `tv my-awesome-channel` etc.

# pipe the output of your program into tv
my_program | tv

fd -t f . | tv --preview 'bat -n --color=always {}'

# or build your own channel on the fly
tv --source-command 'fd -t f .' --preview-command 'bat -n --color=always {}' --preview-size 70
```

> [!TIP]
> 🐚 _Television has **builtin shell integration**. More info [here](https://alexpasmantier.github.io/television/docs/Users/shell-integration)._

For more information, check out the [docs](https://alexpasmantier.github.io/television/).

## Using tv inside your favorite editor

- **Neovim**: [tv.nvim](https://github.com/alexpasmantier/tv.nvim) (lua)
- **Vim**: [tv.vim](https://github.com/prabirshrestha/tv.vim) (vimscript)
- **Zed**: [Easy Telescope-style file finder in Zed using television](https://github.com/zed-industries/zed/discussions/22581)
- **VSCode**: [using television as a file picker inside vscode](https://marketplace.visualstudio.com/items?itemName=alexpasmantier.television)

## Credits

This project was inspired by the **awesome** work done by the [telescope](https://github.com/nvim-telescope/telescope.nvim) neovim plugin.

It also leverages the great [helix](https://github.com/helix-editor/helix) editor's nucleo fuzzy matching library, the [tokio](https://github.com/tokio-rs/tokio) async runtime as well as the **formidable** [ratatui](https://github.com/ratatui/ratatui) library.

A special thanks to tv's contributors for their help and support:

<a href="https://github.com/alexpasmantier/television/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=alexpasmantier/television" />
</a>
