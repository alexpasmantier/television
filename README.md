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

Television is a fast, portable fuzzy finder for the terminal. It lets you search in real-time through any kind of data source such as files, text, git repositories, environment variables, docker containers, and more.

**[Read the documentation](https://alexpasmantier.github.io/television/)**

## Quick Start

```sh
tv              # Search files (default channel)
tv text         # Search file contents
tv git-repos    # Find git repositories
tv --help       # See all options
```

For a complete introduction, see the [Quickstart Guide](https://alexpasmantier.github.io/television/docs/getting-started/quickstart).

## Installation

### Quick Install (Recommended)

```sh
curl -fsSL https://alexpasmantier.github.io/television/install.sh | bash
```

### Package Managers

| Platform | Command |
|----------|---------|
| **Arch Linux** | `pacman -S television` |
| **Homebrew** | `brew install television` |
| **Cargo** | `cargo install television` |
| **Scoop** | `scoop bucket add extras && scoop install television` |
| **WinGet** | `winget install --exact --id alexpasmantier.television` |
| **Nix** | `nix run nixpkgs#television` |

For more installation options, see [Installation](https://alexpasmantier.github.io/television/docs/getting-started/installation).

## Custom Channels

Create custom channels for any workflow. Here's an example TLDR channel:

```toml
# ~/.config/television/cable/tldr.toml
[metadata]
name = "tldr"
description = "Browse TLDR pages"

[source]
command = "tldr --list"

[preview]
command = "tldr '{}'"

[keybindings]
ctrl-e = "actions:open"

[actions.open]
command = "tldr '{}'"
mode = "execute"
```

Learn more about [creating channels](https://alexpasmantier.github.io/television/docs/getting-started/first-channel).

## Shell Integration

Enable smart autocomplete (<kbd>Ctrl</kbd>+<kbd>T</kbd>) and history search (<kbd>Ctrl</kbd>+<kbd>R</kbd>):

```sh
# Zsh
echo 'eval "$(tv init zsh)"' >> ~/.zshrc

# Bash
echo 'eval "$(tv init bash)"' >> ~/.bashrc
```

See [Shell Integration](https://alexpasmantier.github.io/television/docs/user-guide/shell-integration) for more shells.

## Editor Integration

- **Neovim**: [tv.nvim](https://github.com/alexpasmantier/tv.nvim)
- **Vim**: [tv.vim](https://github.com/prabirshrestha/tv.vim)
- **VSCode**: [Television extension](https://marketplace.visualstudio.com/items?itemName=alexpasmantier.television)
- **Zed**: [Telescope-style setup](https://zed.dev/blog/hidden-gems-part-2#emulate-vims-telescope-via-television)

## Documentation

- [Getting Started](https://alexpasmantier.github.io/television/docs/getting-started/quickstart)
- [User Guide](https://alexpasmantier.github.io/television/docs/user-guide/channels)
- [Tips and Tricks](https://alexpasmantier.github.io/television/docs/advanced/tips-and-tricks)
- [Reference](https://alexpasmantier.github.io/television/docs/reference/cli)

## Credits

Inspired by [telescope.nvim](https://github.com/nvim-telescope/telescope.nvim). Built with [nucleo](https://github.com/helix-editor/helix) (fuzzy matching), [tokio](https://github.com/tokio-rs/tokio) (async runtime), and [ratatui](https://github.com/ratatui/ratatui) (TUI framework).

Thanks to all contributors:

<a href="https://github.com/alexpasmantier/television/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=alexpasmantier/television" />
</a>
