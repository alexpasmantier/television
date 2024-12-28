<div align="center">

# 📺  television
**A blazingly fast general purpose fuzzy finder TUI.**

![docs.rs](https://img.shields.io/docsrs/television-channels)
[![Crates.io](https://img.shields.io/crates/v/television.svg)](https://crates.io/crates/television)
![GitHub branch check runs](https://img.shields.io/github/check-runs/alexpasmantier/television/main)
![GitHub License](https://img.shields.io/github/license/alexpasmantier/television)
![Crates.io Total Downloads](https://img.shields.io/crates/d/television)

| ![television.png](https://github.com/user-attachments/assets/cffc3556-c9f3-4704-8303-8bddf661d139) | 
|:--:| 
| *The revolution will (not) be televised.* |

</div>

## About
`Television` is a blazing fast general purpose fuzzy finder TUI.

It lets you search in no time through any kind of data source (files, git repositories, environment variables, docker
images, you name it!) using a fuzzy matching algorithm and is designed to be easily extensible.


It is inspired by the neovim [telescope](https://github.com/nvim-telescope/telescope.nvim) plugin and leverages [tokio](https://github.com/tokio-rs/tokio) and the *nucleo* matcher used by the [helix](https://github.com/helix-editor/helix) editor to achieve high performance.

## Features
- ⚡️ **High Speed**: uses async I/O as well as multithreading to keep the UI highly responsive.

- 🧠 **Fuzzy Matching**: state of the art fuzzy matching library to filter through lists of entries.

- 🔋 **Batteries Included**: comes with a set of builtin channels and previewers that you can start using out of the box.

- 🐚 **Shell Integration**: allows you to easily integrate `television` with your shell to benefit from smart completion anywhere.

- 📺 **Channels**: designed around the concept of channels, which are a set of builtin data sources that you can search through (e.g. files, git repositories, environment variables, etc.).

- 📡 **Cable Channels**: users may add their own custom channels to tv using a simple configuration file.

- 📜 **Previewers**: allows you to preview the contents of an entry in a separate pane.

- 🖼️ **Builtin Syntax Highlighting**: tv comes with builtin asynchronous syntax highlighting for a variety of file types.

- 🎛️ **Keybindings**: tv comes with a set of sensible default keybindings based on vi and other popular terminal shortcuts.

- 🌈 **Themes**: tv comes with a variety of themes that you can choose from, and you can easily craft your own.

- 📦 **Cross-platform**: tv is cross-platform and should work on any platform that supports Rust.

- ✅ **Terminal Emulator Compatibility**: tv has been tested with a variety of terminal emulators and should just work on most.


## Installation
<details>  
<summary>Homebrew</summary>
    
  ```bash
  brew install television
  ```

</details>
<details>
  <summary>
    Arch Linux
  </summary>

  ```bash
  pacman -S television
  ```

</details>
<details>
  <summary>
    Debian-based (Debian, Ubuntu, Pop!_OS, Linux Mint, etc.)
  </summary>
    
  ```bash
  curl -LO https://github.com/alexpasmantier/television/releases/download/0.7.0/television_0.7.0-1_amd64.deb
  sudo dpkg -i television_0.7.0-1_amd64.deb
  ```
    
</details>
<details>
  <summary>
    Conda-forge (cross-platform)
  </summary>
  
  ```bash
  pixi global install television
  ```
</details>
<details>
  <summary>
    Binary
  </summary>
  
  From the [latest release](https://github.com/alexpasmantier/television/releases/latest) page:
  - Download the latest release asset for your platform (e.g. `tv-vX.X.X-linux-x86_64.tar.gz` if you're on a linux x86 machine)
  - Unpack and copy to the relevant location on your system (e.g. `/usr/local/bin` on macos and linux for a global installation)

</details>
<details>
  <summary>
    Cargo
  </summary>

  Setup the latest stable Rust toolchain via rustup:
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup update
  ```
  Install `television`:
  ```bash
  cargo install --locked television
  ```
</details>

### Shell integration

To enable shell integration, run:
```bash
echo 'eval "$(tv init zsh)"' >> ~/.zshrc
```
And then restart your shell. Hitting <kbd>Ctrl-T</kbd> in your shell will now provide you with smart completion powered
by `television`.

*Support for other shells is coming soon.*


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
By default, `television` will launch with the `files` channel on.
| <img width="2213" alt="Screenshot 2024-11-10 at 15 04 20" src="https://github.com/user-attachments/assets/a0fd70a9-ea26-452a-b235-cbce8aeed67f"> |
|:--:|
| *`tv`'s `files` channel running on the *curl* codebase* |

*For more information on the different channels, see the [channels](./channels.md) documentation.*


## Keybindings
Default keybindings are as follows:

| Key | Description |
| :---: | ----------- |
| <kbd>↑</kbd> / <kbd>↓</kbd> | Navigate through the list of entries |
| <kbd>Ctrl</kbd> + <kbd>u</kbd> / <kbd>d</kbd> | Scroll the preview pane up / down |
| <kbd>Enter</kbd> | Select the current entry |
| <kbd>Ctrl</kbd> + <kbd>y</kbd> | Copy the selected entry to the clipboard |
| <kbd>Ctrl</kbd> + <kbd>r</kbd> | Toggle remote control mode |
| <kbd>Ctrl</kbd> + <kbd>s</kbd> | Toggle send to channel mode |
| <kbd>Ctrl</kbd> + <kbd>g</kbd> | Toggle the help panel |
| <kbd>Esc</kbd> | Quit the application |

These keybindings are all configurable (see [Configuration](#configuration)).


## Configuration

**Default configuration: [config.toml](./.config/config.toml)**

Locations where `television` expects the configuration files to be located for each platform:

|Platform|Value|
|--------|:-----:|
|Linux|`$HOME/.config/television/config.toml`|
|macOS|`$HOME/.config/television/config.toml`|
|Windows|`{FOLDERID_LocalAppData}\television\config`|

Or, if you'd rather use the XDG Base Directory Specification, tv will look for the configuration file in
`$XDG_CONFIG_HOME/television/config.toml` if the environment variable is set.

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
`television` uses a fuzzy matching algorithm to filter the list of entries. Its behavior depends on the input pattern you provide.

| Matcher | Pattern |
| --- | :---: |
| Fuzzy | `foo` |
| Substring | `'foo` / `!foo` to negate |
| Prefix | `^foo` / `!^foo` to negate |
| Suffix | `foo$` / `!foo$` to negate |
| Exact | `^foo$` / `!^foo$` to negate |

For more information on the matcher behavior, see the
[nucleo-matcher](https://docs.rs/nucleo-matcher/latest/nucleo_matcher/pattern/enum.AtomKind.html) documentation.

## Terminal Emulators Compatibility
Here is a list of terminal emulators that have currently been tested with `television` and their compatibility status.

| Terminal Emulator | Tested Platforms | Compatibility |
| --- | :---: | :---: |
| Alacritty | macOS, Linux | ✅ |
| Kitty | macOS, Linux | ✅ |
| iTerm2 | macOS | ✅ |
| Ghostty | macOS | ✅ |
| Wezterm | macOS, Linux, Windows | ✅ |
| macOS Terminal | macOS | functional but coloring issues |
| Konsole | Linux | ✅ |
| Terminator | Linux | ✅ |
| Xterm | Linux | ✅ |
| Cmder | Windows | ✖️ |
| Foot | Linux | ✅ |
| Rio | macOS, Linux, Windows | ✅ |
| Warp | macOS | ✅ |
| Hyper | macOS | ✅ |




## Contributions

Contributions, issues and pull requests are welcome.

See [CONTRIBUTING.md](CONTRIBUTING.md) and [good first issues](https://github.com/alexpasmantier/television/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22) for more information.
