# Actions Reference

This document provides a complete reference of all actions available in Television.

## What are Actions?

Actions are the operations tv can perform in response to key presses or other events. They're used in:
- Keybinding configuration
- Custom channel actions
- Multi-action key bindings

## Action Syntax

In configuration files, actions use `snake_case`:

```toml
[keybindings]
ctrl-j = "select_next_entry"
ctrl-k = "select_prev_entry"
```

## Navigation Actions

| Action | Description | Default Key |
|--------|-------------|-------------|
| `select_next_entry` | Move selection down | <kbd>↓</kbd>, <kbd>Ctrl</kbd>+<kbd>n</kbd>, <kbd>Ctrl</kbd>+<kbd>j</kbd> |
| `select_prev_entry` | Move selection up | <kbd>↑</kbd>, <kbd>Ctrl</kbd>+<kbd>p</kbd>, <kbd>Ctrl</kbd>+<kbd>k</kbd> |
| `select_next_page` | Move down one page | - |
| `select_prev_page` | Move up one page | - |

## Selection Actions

| Action | Description | Default Key |
|--------|-------------|-------------|
| `confirm_selection` | Select current entry and exit | <kbd>Enter</kbd> |
| `toggle_selection_down` | Toggle selection, move down | <kbd>Tab</kbd> |
| `toggle_selection_up` | Toggle selection, move up | <kbd>Shift</kbd>+<kbd>Tab</kbd> |
| `copy_entry_to_clipboard` | Copy entry to clipboard | <kbd>Ctrl</kbd>+<kbd>y</kbd> |

## Input Editing Actions

| Action | Description | Default Key |
|--------|-------------|-------------|
| `delete_prev_char` | Delete character before cursor | <kbd>Backspace</kbd> |
| `delete_next_char` | Delete character after cursor | <kbd>Delete</kbd> |
| `delete_prev_word` | Delete previous word | <kbd>Ctrl</kbd>+<kbd>w</kbd> |
| `delete_line` | Clear entire input | <kbd>Ctrl</kbd>+<kbd>u</kbd> |
| `go_to_prev_char` | Move cursor left | <kbd>←</kbd> |
| `go_to_next_char` | Move cursor right | <kbd>→</kbd> |
| `go_to_input_start` | Move cursor to start | <kbd>Home</kbd>, <kbd>Ctrl</kbd>+<kbd>a</kbd> |
| `go_to_input_end` | Move cursor to end | <kbd>End</kbd>, <kbd>Ctrl</kbd>+<kbd>e</kbd> |

## Preview Actions

| Action | Description | Default Key |
|--------|-------------|-------------|
| `scroll_preview_up` | Scroll preview up one line | - |
| `scroll_preview_down` | Scroll preview down one line | - |
| `scroll_preview_half_page_up` | Scroll preview up half page | <kbd>PageUp</kbd> |
| `scroll_preview_half_page_down` | Scroll preview down half page | <kbd>PageDown</kbd> |
| `cycle_previews` | Cycle through preview commands | <kbd>Ctrl</kbd>+<kbd>F</kbd> |

## UI Toggle Actions

| Action | Description | Default Key |
|--------|-------------|-------------|
| `toggle_preview` | Show/hide preview panel | <kbd>Ctrl</kbd>+<kbd>o</kbd> |
| `toggle_remote_control` | Show/hide channel picker | <kbd>Ctrl</kbd>+<kbd>t</kbd> |
| `toggle_help` | Show/hide help panel | <kbd>Ctrl</kbd>+<kbd>h</kbd> |
| `toggle_status_bar` | Show/hide status bar | <kbd>F12</kbd> |
| `toggle_layout` | Switch portrait/landscape | <kbd>Ctrl</kbd>+<kbd>l</kbd> |
| `toggle_action_picker` | Show available actions | <kbd>Ctrl</kbd>+<kbd>x</kbd> |

## Channel Actions

| Action | Description | Default Key |
|--------|-------------|-------------|
| `cycle_sources` | Cycle through source commands | <kbd>Ctrl</kbd>+<kbd>s</kbd> |
| `reload_source` | Reload current source | <kbd>Ctrl</kbd>+<kbd>r</kbd> |

## History Actions

| Action | Description | Default Key |
|--------|-------------|-------------|
| `select_prev_history` | Previous history entry | <kbd>Ctrl</kbd>+<kbd>↑</kbd> |
| `select_next_history` | Next history entry | <kbd>Ctrl</kbd>+<kbd>↓</kbd> |

## Application Actions

| Action | Description | Default Key |
|--------|-------------|-------------|
| `quit` | Exit tv | <kbd>Esc</kbd>, <kbd>Ctrl</kbd>+<kbd>c</kbd> |

## Custom Actions

Define custom actions in channels using the `actions:` prefix:

```toml
[keybindings]
ctrl-e = "actions:edit"
ctrl-o = "actions:open"

[actions.edit]
description = "Edit file"
command = "nvim '{}'"
mode = "execute"

[actions.open]
description = "Open in default app"
command = "xdg-open '{}'"
mode = "fork"
```

### Action Modes

| Mode | Behavior |
|------|----------|
| `fork` | Run command, return to tv when done |
| `execute` | Replace tv with the command |

## Multiple Actions

Bind multiple actions to a single key:

```toml
[keybindings]
ctrl-r = ["reload_source", "go_to_input_start"]
```

Actions execute in sequence.

## Reserved Actions (Internal)

These actions are used internally and cannot be bound:

| Action | Purpose |
|--------|---------|
| `render` | Trigger UI render |
| `resize` | Handle terminal resize |
| `clear_screen` | Clear terminal |
| `tick` | Application tick |
| `suspend` | Suspend application |
| `resume` | Resume application |
| `error` | Display error |
| `no_op` | No operation |

## Example Configurations

### Vim-style Navigation

```toml
[keybindings]
ctrl-j = "select_next_entry"
ctrl-k = "select_prev_entry"
ctrl-d = "select_next_page"
ctrl-u = "select_prev_page"
```

> **Note:** Single character keys like `j` and `k` cannot be used for navigation as they are captured as search input. Use modifier keys (Ctrl, Alt) for navigation bindings.

### Emacs-like Input

These are already the defaults:

```toml
[keybindings]
ctrl-a = "go_to_input_start"
ctrl-e = "go_to_input_end"
ctrl-u = "delete_line"
ctrl-w = "delete_prev_word"
```

### Quick Actions

```toml
[keybindings]
ctrl-y = "copy_entry_to_clipboard"
ctrl-r = "reload_source"
f5 = "reload_source"
```

## See Also

- [Keybindings configuration](../user-guide/03-keybindings.md)
- [Channel specification](./03-channel-spec.md)
