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
`Television` is a blazingly fast general purpose fuzzy finder TUI.

It is inspired by the neovim [telescope](https://github.com/nvim-telescope/telescope.nvim) plugin and is designed to be fast, efficient, simple to use and easily extensible. It is built on top of [tokio](https://github.com/tokio-rs/tokio), [ratatui](https://github.com/ratatui/ratatui) and the *nucleo* matcher used by the [helix](https://github.com/helix-editor/helix) editor.


## Installation
<details>  
<summary>MacOS</summary>
    
  ```bash
  brew install alexpasmantier/television/television
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
  curl -LO https://github.com/alexpasmantier/television/releases/download/0.5.2/television_0.5.2-1_amd64.deb
  sudo dpkg -i television_0.5.2-1_amd64.deb
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

## Usage
```bash
tv [channel] #[default: files] [possible values: env, files, git-repos, text, alias]
```
By default, `television` will launch with the `files` channel on.
| <img width="2213" alt="Screenshot 2024-11-10 at 15 04 20" src="https://github.com/user-attachments/assets/a0fd70a9-ea26-452a-b235-cbce8aeed67f"> |
|:--:|
| *`tv`'s `files` channel running on the *curl* codebase* |

#### Matcher behavior
`television` uses a fuzzy matching algorithm to filter the list of entries. The algorithm that is used depends on the
input pattern that you provide.

| Matcher | Pattern |
| --- | :---: |
| Fuzzy | `foo` |
| Substring | `'foo` / `!foo` to negate |
| Prefix | `^foo` / `!^foo` to negate |
| Suffix | `foo$` / `!foo$` to negate |
| Exact | `^foo$` / `!^foo$` to negate |

For more information on the matcher behavior, see the
[nucleo-matcher](https://docs.rs/nucleo-matcher/latest/nucleo_matcher/pattern/enum.AtomKind.html) documentation.

## Keybindings
Default keybindings are as follows:

| Key | Description |
| :---: | ----------- |
| <kbd>↑</kbd> / <kbd>↓</kbd> or <kbd>Ctrl</kbd> + <kbd>n</kbd> / <kbd>p</kbd> | Navigate through the list of entries |
| <kbd>Ctrl</kbd> + <kbd>u</kbd> / <kbd>d</kbd> | Scroll the preview pane up / down |
| <kbd>Enter</kbd> | Select the current entry |
| <kbd>Ctrl</kbd> + <kbd>y</kbd> | Copy the selected entry to the clipboard |
| <kbd>Ctrl</kbd> + <kbd>r</kbd> | Toggle remote control mode |
| <kbd>Ctrl</kbd> + <kbd>s</kbd> | Toggle send to channel mode |
| <kbd>Esc</kbd> | Quit the application |

These keybindings can be customized in the configuration file (see [Customization](#customization)).

## Built-in Channels
The following channels are currently available:
- `Files`: search through files in a directory tree.
- `Text`: search through textual content in a directory tree.
- `GitRepos`: search through git repositories anywhere on the file system.
- `Env`: search through environment variables and their values.
- `Alias`: search through shell aliases and their values.
- `Stdin`: search through lines of text from stdin.


## Design (high-level)
#### Channels
**Television**'s design is primarily based on the concept of **Channels**.
Channels are just structs that implement the `OnAir` trait. 

As such, channels can virtually be anything that can respond to a user query and return a result under the form of a list of entries. This means channels can be anything from conventional data sources you might want to search through (like files, git repositories, remote filesystems, environment variables etc.) to more exotic implementations that might inclue a REPL, a calculator, a web browser, search through your spotify library, your email, etc.



**Television** provides a set of built-in **Channels** that can be used out of the box (see [Built-in Channels](#built-in-channels)). The list of available channels
will grow over time as new channels are implemented to satisfy different use cases. 


#### Transitions
When it makes sense, **Television** allows for transitions between different channels. For example, you might want to
start searching through git repositories, then refine your search to a specific set of files in that shortlist of
repositories and then finally search through the textual content of those files.

This can easily be achieved using transitions.

#### Previewers
Entries returned by different channels can be previewed in a separate pane. This is useful when you want to see the
contents of a file, the value of an environment variable, etc. Because entries returned by different channels may
represent different types of data, **Television** allows for channels to declare the type of previewer that should be
used. Television comes with a set of built-in previewers that can be used out of the box and will grow over time.

## Recipes
Here are some examples of how you can use `television` to make your life easier, more productive and fun. You may want to add some of these examples as aliases to your shell configuration file so that you can easily access them.

**NOTE**: *most of the following examples are meant for macOS. Most of the commands should work on Linux as well, but you may need to adjust them slightly.*

#### CDing into git repo
```bash
cd `tv git-repos`
```
#### Opening file in default editor
```bash
open `tv`
```
##### VSCode:
```bash
code --goto `tv`
```
##### Vim
```bash
vim `tv`
```
at a specific line using the text channel
```bash
tv text | xargs -oI {} sh -c 'vim "$(echo {} | cut -d ":" -f 1)" +$(echo {} | cut -d ":" -f 2)'
```
#### Inspecting the current directory
```bash
ls -1a | tv
```

## Terminal Emulators Compatibility
Here is a list of terminal emulators that have currently been tested with `television` and their compatibility status.

| Terminal Emulator | Tested Platforms | Compatibility |
| --- | :---: | :---: |
| Alacritty | macOS, Linux | ✅ |
| Kitty | macOS, Linux | ✅ |
| iTerm2 | macOS | ✅ |
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




## Customization
You may wish to customize the behavior of `television` by providing your own configuration file. The configuration file
is a simple TOML file that allows you to customize the behavior of `television` in a number of ways.

|Platform|Value|
|--------|:-----:|
|Linux|`$HOME/.config/television/config.toml`|
|macOS|`$HOME/Library/Application Support/com.television/config.toml`|
|Windows|`{FOLDERID_LocalAppData}\television\config`|

**NOTE**: on either platform, `XDG_CONFIG_HOME` will always take precedence over default locations if set, in which case
television will expect the configuration file to be in `$XDG_CONFIG_HOME/television/config.toml`.

You may also override these default paths by setting the `TELEVISION_CONFIG` environment variable to the path of your desired configuration **folder**.

Example:
```bash
export TELEVISION_CONFIG=$HOME/.config/television
touch $TELEVISION_CONFIG/config.toml
```

#### Default Configuration
The default configuration file can be found in the repository's [./.config/config.toml](./.config/config.toml).

## Contributions

Contributions, issues and pull requests are welcome.

See [CONTRIBUTING.md](CONTRIBUTING.md) and [good first issues](https://github.com/alexpasmantier/television/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22) for more information.
