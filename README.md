

![GitHub branch check runs](https://img.shields.io/github/check-runs/alexpasmantier/television/main)
![GitHub License](https://img.shields.io/github/license/alexpasmantier/television)


# 📺 television
| ![television.png](https://github.com/user-attachments/assets/cffc3556-c9f3-4704-8303-8bddf661d139) | 
|:--:| 
| *The revolution will (not) be televised.* |

## 📺 About
`Television` is a very fast general purpose fuzzy finder TUI written in Rust. 

It is inspired by the neovim [telescope](https://github.com/nvim-telescope/telescope.nvim) plugin and is designed to be fast, efficient, simple to use and easily extensible. It is built on top of [tokio](https://github.com/tokio-rs/tokio), [ratatui](https://github.com/ratatui/ratatui) and the *nucleo* matcher used by the [helix](https://github.com/helix-editor/helix) editor.


## 📺 Installation
```bash
cargo install television
```

## 📺 Usage
```bash
tv [channel] #[default: files] [possible values: env, files, git-repos, text, alias]
```
By default, `television` will search through files in the current directory.

## 📺 Built-in Channels
The following channels are currently available:
- `Files`: search through files in a directory tree.
- `Text`: search through textual content in a directory tree.
- `GitRepos`: search through git repositories anywhere on the file system.
- `Env`: search through environment variables and their values.
- `Alias`: search through shell aliases and their values.
- `Stdin`: search through lines of text from stdin.


## 📺 Design
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
