![GitHub branch check runs](https://img.shields.io/github/check-runs/alexpasmantier/television/main)
![GitHub License](https://img.shields.io/github/license/alexpasmantier/television)


# 📺 television

```
           _______________
          |,----------.  |\
          ||           |=| |
          ||           | | |
          ||           |o| |
          |`-----------' |/
          `--------------'
  __      __         _     _
 / /____ / /__ _  __(_)__ (_)__  ___
/ __/ -_) / -_) |/ / (_-</ / _ \/ _ \
\__/\__/_/\__/|___/_/___/_/\___/_//_/

```

 The revolution will be televised.


## 📺 About
`Television` is a blazingly fast general purpose fuzzy finder TUI written in Rust. It is inspired by the neovim `Telescope` plugin and the `fzf` command line tool. It is designed to be fast, efficient, easy to use and easily extensible. It is built on top of `tokio`, `ratatui` and the `nucleo` matcher which is used in the `helix` editor.

## 📺 Design
`Television`'s design is based on the concept of `Channels`. A `Channel` is a source of data that can be used for fuzzy
finding. A `Channel` can be anything from a file system directory, a git repository, a list of strings, a list of
numbers, etc. `Television` provides a set of built-in `Channels` that can be used out of the box. However, `Television`
is designed to be easily extensible and allows users to define their own custom `Channels` by implementing a simple
trait.

## Built-in Channels
- `Files`: search through files in a directory tree.
- `Text`: search through textual content in a directory tree.
- `GitRepos`: search through git repositories anywhere on the file system.
- `Env`: search through environment variables and their values.
- `Alias`: search through shell aliases and their values.
- `Stdin`: search through lines of text from stdin.

