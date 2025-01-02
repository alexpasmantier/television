<div align="center">

# 📺  television
**Blazing fast general purpose fuzzy finder TUI.**

![docs.rs](https://img.shields.io/docsrs/television-channels)
[![Crates.io](https://img.shields.io/crates/v/television.svg)](https://crates.io/crates/television)
![GitHub branch check runs](https://img.shields.io/github/check-runs/alexpasmantier/television/main)
![GitHub License](https://img.shields.io/github/license/alexpasmantier/television)
![Crates.io Total Downloads](https://img.shields.io/crates/d/television)

![tv on the linux codebase](./assets/tv-linux-gamepad.png "tv running on the linux codebase")

</div>

## About
`Television` is a fast and versatile fuzzy finder TUI.

It lets you quickly search through any kind of data source (files, git repositories, environment variables, docker
images, you name it) using a fuzzy matching algorithm and is designed to be easily extensible.


It is inspired by the neovim [telescope](https://github.com/nvim-telescope/telescope.nvim) plugin and leverages [tokio](https://github.com/tokio-rs/tokio) and the *nucleo* matcher used by the [helix](https://github.com/helix-editor/helix) editor to ensure optimal performance.


## Features
- ⚡️ **High Speed**: asynchronous I/O and multithreading to ensure a smooth and responsive UI.

- 🧠 **Fuzzy Matching**: cutting-edge fuzzy matching library for efficiently filtering through lists of entries.

- 🔋 **Batteries Included**: comes with a set of builtin channels and previewers that you can start using out of the box.

- 🐚 **Shell Integration**: benefit from smart completion anywhere using `television`'s shell integration.

- 📺 **Channels**: designed around the concept of channels, which are a set of builtin data sources that you can search through (e.g. files, git repositories, environment variables, etc).

- 📡 **Cable Channels**: users may add their own custom channels to tv using a simple and centralized configuration file.

- 📜 **Previewers**: allows you to preview the contents of an entry in a separate pane.

- 🖼️ **Builtin Syntax Highlighting**: comes with builtin asynchronous syntax highlighting for a wide variety of file types.

- 🎛️ **Keybindings**: includes a set of intuitive default keybindings inspired by vim and other popular terminal shortcuts.

- 🌈 **Themes**: either use one of the 10 builtin themes or create your own easily.

- 📦 **Cross-platform**: works on Linux, MacOS and Windows.

- ✅ **Terminal Emulator Compatibility**: television works flawlessly on all major terminal emulators.


## Installation
See the [installation docs](https://github.com/alexpasmantier/television/wiki/Installation).


## Usage
```bash
tv [channel] #[default: files] [possible values: env, files, git-repos, text, alias]

# e.g. to search through environment variables
tv env

# piping into tv (e.g. logs)
my_program | tv

# piping into tv with a custom preview command
fd -t f . | tv --preview 'bat -n --color=always {0}'

```
*For more information on the different channels, see the [channels](./docs/channels.md) documentation.*

Television can also integrate with your shell to provide autocompletion based on the commands you start typing. See [Shell Autocompletion](https://github.com/alexpasmantier/television/wiki/Shell-Autocompletion).



https://github.com/user-attachments/assets/395f17f6-14b9-4015-a50a-648259d9f253



## Keybindings

For information about available keybindings, check the [associated page of the wiki](https://github.com/alexpasmantier/television/wiki/Keybindings)


## Configuration

For information about tv's configuration file, check the [associated page of the wiki](https://github.com/alexpasmantier/television/wiki/Configuration-file)

## Themes
Builtin themes are available in the [themes](./themes) directory. Feel free to experiment and maybe even contribute your own!

| ![catppuccin](./assets/catppuccin.png "catppuccin") catppuccin | ![gruvbox](./assets/gruvbox.png "gruvbox") gruvbox-dark |
|:--:|:--:|
| ![solarized-dark](./assets/solarized-dark.png "gruvbox-light") **solarized-dark** | ![nord](./assets/nord.png "nord") **nord** |

You may create your own custom themes by adding them to the `themes` directory in your configuration folder and then referring to them by file name (without the extension) in the configuration file.
```
config_location/
├── themes/
│   └── my_theme.toml
└── config.toml
```

## Search Patterns
For information on how to use search patterns with tv, refer to the [associated page of the wiki](https://github.com/alexpasmantier/television/wiki/Search-patterns)

## Contributions

Contributions, issues and pull requests are welcome.

See [CONTRIBUTING.md](CONTRIBUTING.md) and [good first issues](https://github.com/alexpasmantier/television/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22) for more information.

## Credits
This project was inspired by the **awesome** work done by the [telescope](https://github.com/nvim-telescope/telescope.nvim) neovim plugin.

It also leverages the great [helix](https://github.com/helix-editor/helix) editor's nucleo fuzzy matching library, the [tokio](https://github.com/tokio-rs/tokio) async runtime as well as the **formidable** [ratatui](https://github.com/ratatui/ratatui) library.
