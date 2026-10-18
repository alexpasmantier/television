> **语言**：本文档为简体中文翻译，[点此查看英文原版 README.md](README.md)。

<div align="center">

# Television (tv)

**一个非常快速、可移植、易于 Hack 的模糊查找器。**

![GitHub Release](https://img.shields.io/github/v/release/alexpasmantier/television?display_name=tag&color=%23a6a)
![GitHub branch check runs](https://img.shields.io/github/check-runs/alexpasmantier/television/main)
![GitHub License](https://img.shields.io/github/license/alexpasmantier/television)
[![Discord](https://img.shields.io/discord/1366133668535341116?logo=discord)](https://discord.gg/hQBrzsJgUg)

<img width="1909" height="1042" alt="image" src="https://github.com/user-attachments/assets/6255d115-3490-45ee-82cb-b3a84b2e852d" />

<!-- <img width="1473" height="761" alt="image" src="https://github.com/user-attachments/assets/0ff21160-6060-4804-ae0b-2ce9500ed9c5" /> -->

</div>

## 关于

Television 是一个快速、可移植的终端模糊查找器。它让你实时搜索任意类型的数据源，例如文件、文本、Git 仓库、环境变量、Docker 容器等。

**[阅读文档](https://alexpasmantier.github.io/television/)**

## 快速开始

```sh
tv              # Search files (default channel)
tv text         # Search file contents
tv git-repos    # Find git repositories
tv --help       # See all options
```

完整入门介绍请见[快速上手指南](https://alexpasmantier.github.io/television/getting-started/quickstart)。

## 安装

### 快速安装（推荐）

```sh
curl -fsSL https://alexpasmantier.github.io/television/install.sh | bash
```

### 包管理器

| 平台 | 命令 |
| -------------- | ------------------------------------------------------- |
| **Arch Linux** | `pacman -S television`                                  |
| **Homebrew**   | `brew install television`                               |
| **Cargo**      | `cargo install --locked television`                     |
| **Scoop**      | `scoop bucket add extras && scoop install television`   |
| **WinGet**     | `winget install --exact --id alexpasmantier.television` |
| **Nix**        | `nix run nixpkgs#television`                            |

更多安装选项请见[安装](https://alexpasmantier.github.io/television/getting-started/installation)。

## 自定义频道

为任意工作流创建自定义频道。下面是一个 TLDR 频道的示例：

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

如需了解更多，请参阅[创建频道](https://alexpasmantier.github.io/television/getting-started/first-channel)。

## Shell 集成

启用智能自动补全（<kbd>Ctrl</kbd>+<kbd>T</kbd>）与历史搜索（<kbd>Ctrl</kbd>+<kbd>R</kbd>）：

```sh
# Zsh
echo 'eval "$(tv init zsh)"' >> ~/.zshrc

# Bash
echo 'eval "$(tv init bash)"' >> ~/.bashrc
```

其他 shell 的支持情况请见 [Shell Integration](https://alexpasmantier.github.io/television/user-guide/shell-integration)。

## 编辑器集成

- **Neovim**: [tv.nvim](https://github.com/alexpasmantier/tv.nvim)
- **Vim**: [tv.vim](https://github.com/prabirshrestha/tv.vim)
- **VSCode**: [Television extension](https://marketplace.visualstudio.com/items?itemName=alexpasmantier.television)
- **Zed**: [Telescope-style setup](https://zed.dev/blog/hidden-gems-part-2#emulate-vims-telescope-via-television)
- **JetBrains IDEs**: [Television plugin](https://plugins.jetbrains.com/plugin/31448-television)

## 文档

- [Getting Started](https://alexpasmantier.github.io/television/getting-started/quickstart)
- [User Guide](https://alexpasmantier.github.io/television/user-guide/channels)
- [Tips and Tricks](https://alexpasmantier.github.io/television/advanced/tips-and-tricks)
- [Reference](https://alexpasmantier.github.io/television/reference/cli)

## 致谢

- [telescope.nvim](https://github.com/nvim-telescope/telescope.nvim)
- [frizbee](https://github.com/saghen/frizbee)
- [ratatui](https://github.com/ratatui/ratatui)

<a href="https://github.com/alexpasmantier/television/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=alexpasmantier/television" />
</a>
