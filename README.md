![docs.rs](https://img.shields.io/docsrs/television-channels)
[![Crates.io](https://img.shields.io/crates/v/television.svg)](https://crates.io/crates/television)
![GitHub branch check runs](https://img.shields.io/github/check-runs/alexpasmantier/television/main)
![GitHub License](https://img.shields.io/github/license/alexpasmantier/television)
![Crates.io Total Downloads](https://img.shields.io/crates/d/television)


# 📺  television
| ![television.png](https://github.com/user-attachments/assets/cffc3556-c9f3-4704-8303-8bddf661d139) | 
|:--:| 
| *The revolution will (not) be televised.* |

## About
`Television` is a very fast general purpose fuzzy finder TUI written in Rust. 

It is inspired by the neovim [telescope](https://github.com/nvim-telescope/telescope.nvim) plugin and is designed to be fast, efficient, simple to use and easily extensible. It is built on top of [tokio](https://github.com/tokio-rs/tokio), [ratatui](https://github.com/ratatui/ratatui) and the *nucleo* matcher used by the [helix](https://github.com/helix-editor/helix) editor.


## Installation
```bash
cargo install television
```

## Usage
```bash
tv [channel] #[default: files] [possible values: env, files, git-repos, text, alias]
```
By default, `television` will launch with the `files` channel on.

## Built-in Channels
The following channels are currently available:
- `Files`: search through files in a directory tree.
- `Text`: search through textual content in a directory tree.
- `GitRepos`: search through git repositories anywhere on the file system.
- `Env`: search through environment variables and their values.
- `Alias`: search through shell aliases and their values.
- `Stdin`: search through lines of text from stdin.


## Design
#### Channels
**Television**'s design is primarily based on the concept of **Channels**.

A **Channel** is a source of data that can be used for fuzzy finding. It can be anything from a file system directory, a git repository, a list of strings, a list of numbers, etc. 

**Television** provides a set of built-in **Channels** that can be used out of the box (see [Built-in Channels](#📺-built-in-channels)). The list of available channels
will grow over time as new channels are implemented to satisfy different use cases. 

Because a **Channel** is nothing more than a source of data that can respond to a user query, channels can virtually search through anything ranging from a local file system to a remote database, a list of environment variables, something passed through stdin, etc.

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



## Customization
You may wish to customize the behavior of `television` by providing your own configuration file. The configuration file
is a simple TOML file that allows you to customize the behavior of `television` in a number of ways.

|Platform|Value|
|--------|-----|
|Linux|`$XDG_CONFIG_HOME/television/config.toml` or `$HOME/.config/television/config.toml`|
|macOS|`$HOME/Library/Application Support/television/config.toml`|
|Windows|`{FOLDERID_LocalAppData}\television\config`|

Any of these paths may be overriden by setting the `TELEVISION_CONFIG` environment variable to the path of your desired configuration folder.

#### Default Configuration
```toml
# Ui settings
# ----------------------------------------------------------------------------
[ui]
# Whether to use nerd font icons in the UI
# This option requires a font patched with Nerd Font in order to properly
# display glyphs (see https://www.nerdfonts.com/ for more information)
use_nerd_font_icons = false
# How much space to allocate for the UI (in percentage of the screen)
# ┌───────────────────────────────────────┐
# │                                       │
# │            Terminal screen            │
# │    ┌─────────────────────────────┐    │
# │    │                             │    │
# │    │                             │    │
# │    │                             │    │
# │    │       Television UI         │    │
# │    │                             │    │
# │    │                             │    │
# │    │                             │    │
# │    │                             │    │
# │    └─────────────────────────────┘    │
# │                                       │
# │                                       │
# └───────────────────────────────────────┘
ui_scale = 80

# Previewers settings
# ----------------------------------------------------------------------------
[previewers.file]
# The theme to use for syntax highlighting
# A list of available themes can be found in the https://github.com/sharkdp/bat
# repository which uses the same syntax highlighting engine as television
theme = "Visual Studio Dark+"

# Keybindings
# ----------------------------------------------------------------------------
# Channel mode keybindings
[keybindings.Channel]
# Quit the application
esc = "Quit"
# Scrolling through entries
down = "SelectNextEntry"
ctrl-n = "SelectNextEntry"
up = "SelectPrevEntry"
ctrl-p = "SelectPrevEntry"
# Scrolling the preview pane
ctrl-d = "ScrollPreviewHalfPageDown"
ctrl-u = "ScrollPreviewHalfPageUp"
# Select an entry
enter = "SelectEntry"
# Copy the selected entry to the clipboard
ctrl-y = "CopyEntryToClipboard"
# Toggle the remote control mode
ctrl-r = "ToggleRemoteControl"
# Toggle the send to channel mode
ctrl-s = "ToggleSendToChannel"

# Remote control mode keybindings
[keybindings.RemoteControl]
# Quit the application
esc = "Quit"
# Scrolling through entries
down = "SelectNextEntry"
up = "SelectPrevEntry"
ctrl-n = "SelectNextEntry"
ctrl-p = "SelectPrevEntry"
# Select an entry
enter = "SelectEntry"
# Toggle the remote control mode
ctrl-r = "ToggleRemoteControl"

# Send to channel mode keybindings
[keybindings.SendToChannel]
# Quit the application
esc = "Quit"
# Scrolling through entries
down = "SelectNextEntry"
up = "SelectPrevEntry"
ctrl-n = "SelectNextEntry"
ctrl-p = "SelectPrevEntry"
# Select an entry
enter = "SelectEntry"
# Toggle the send to channel mode
ctrl-s = "ToggleSendToChannel"
```
