# Channel Specification

This document provides the complete reference for channel TOML configuration files.

## File Location

Channels are stored as `.toml` files in:
- **Linux/macOS**: `~/.config/television/cable/`
- **Windows**: `%LocalAppData%\television\cable\`
- **Custom**: Set via `$TELEVISION_CONFIG/cable/` or `--cable-dir`

## High-Level Structure

```toml
[metadata]
# Channel identification and requirements

[source]
# What to search through

[preview]
# How to preview entries

[ui]
# UI customization

[keybindings]
# Key mappings

[actions.NAME]
# Custom action definitions
```

## [metadata]

Channel identification and documentation.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Unique channel identifier |
| `description` | string | No | Human-readable description |
| `requirements` | string[] | No | Required external tools (checked at runtime) |

**Example:**
```toml
[metadata]
name = "files"
description = "Browse and select files"
requirements = ["fd", "bat"]
```

## [source]

Defines what data the channel searches through.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `command` | string or string[] | Yes | Command(s) that produce entries |
| `ansi` | boolean | No | Parse ANSI escape codes (default: false) |
| `display` | string | No | Template for display (incompatible with `ansi = true`) |
| `output` | string | No | Template for final output |
| `watch` | float | No | Reload interval in seconds |
| `entry_delimiter` | string | No | Custom entry delimiter (default: newline) |
| `no_sort` | boolean | No | Preserve original source order, disabling match-quality sorting and frecency (default: false) |
| `frecency` | boolean | No | Enable frecency-based ranking for this channel (default: true). See [Frecency Sorting](../advanced/02-tips-and-tricks.md#frecency-sorting) |

### Single Source Command

```toml
[source]
command = "fd -t f"
```

### Multiple Source Commands (Cycling)

```toml
[source]
command = ["fd -t f", "fd -t f -H", "fd -t f -H -I"]
# Press Ctrl+S to cycle between commands
```

### With ANSI Colors

```toml
[source]
command = "git log --oneline --color=always"
ansi = true
output = "{strip_ansi|split: :0}"  # Clean output
```

### Display Template

```toml
[source]
command = "docker ps --format '{{.ID}}\\t{{.Names}}\\t{{.Status}}'"
display = "{split:\\t:1} ({split:\\t:2})"  # Show: name (status)
output = "{split:\\t:0}"  # Output: container ID
```

### Watch Mode

```toml
[source]
command = "docker ps"
watch = 2.0  # Reload every 2 seconds
```

### Custom Delimiter

```toml
[source]
command = "find . -print0"
entry_delimiter = "\0"  # Null-byte separated
```

## [preview]

Defines how to preview entries.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `command` | string or string[] | No | Preview command template(s) |
| `env` | table | No | Environment variables for preview |
| `offset` | string | No | Template to extract line offset |
| `header` | string | No | Preview panel header template |
| `footer` | string | No | Preview panel footer template |

### Basic Preview

```toml
[preview]
command = "bat -n --color=always '{}'"
```

### Multiple Preview Commands (Cycling)

```toml
[preview]
command = ["bat -n --color=always '{}'", "cat '{}'", "xxd '{}' | head -100"]
# Press Ctrl+F to cycle between preview commands
```

### With Environment Variables

```toml
[preview]
command = "bat -n --color=always '{}'"
env = { BAT_THEME = "ansi" }
```

### With Line Offset

```toml
# Entry format: "file.txt:42:content"
[preview]
command = "bat -H '{split:\\::1}' --color=always '{split:\\::0}'"
offset = "{split:\\::1}"  # Scroll to line 42
```

### With Header/Footer

```toml
[preview]
command = "bat -n --color=always '{}'"
header = "File: {}"
footer = "Size: $(stat -c%s '{}')"
```

## [ui]

Customize the user interface.

### Top-Level Options

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `ui_scale` | integer (0-100) | 100 | Percentage of terminal to use |
| `layout` | string | "landscape" | "landscape" or "portrait" |
| `input_bar_position` | string | "top" | "top" or "bottom" |
| `input_header` | string | channel name | Input bar header text |
| `input_prompt` | string | ">" | Input prompt string |

```toml
[ui]
ui_scale = 80
layout = "portrait"
input_bar_position = "bottom"
input_header = "Search files:"
input_prompt = ">> "
```

### [ui.preview_panel]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `size` | integer (0-100) | 50 | Preview panel size percentage |
| `header` | string | - | Header template |
| `footer` | string | - | Footer template |
| `scrollbar` | boolean | true | Show scrollbar |
| `border_type` | string | "rounded" | "none", "plain", "rounded", "thick" |
| `padding` | table | all 0 | Panel padding |
| `hidden` | boolean | false | Hide by default |

```toml
[ui.preview_panel]
size = 60
header = "{}"
scrollbar = true
border_type = "rounded"
padding = { left = 1, right = 1 }
hidden = false
```

### [ui.results_panel]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `border_type` | string | "rounded" | Border style |
| `padding` | table | all 0 | Panel padding |

```toml
[ui.results_panel]
border_type = "plain"
padding = { top = 1, bottom = 1 }
```

### [ui.input_bar]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `border_type` | string | "rounded" | Border style |
| `padding` | table | all 0 | Bar padding |

```toml
[ui.input_bar]
border_type = "rounded"
padding = { left = 2, right = 2 }
```

### [ui.status_bar]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `separator_open` | string | "" | Opening separator |
| `separator_close` | string | "" | Closing separator |
| `hidden` | boolean | false | Hide by default |

```toml
[ui.status_bar]
separator_open = "["
separator_close = "]"
hidden = false
```

### [ui.help_panel]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `show_categories` | boolean | true | Group by category |
| `hidden` | boolean | true | Hide by default |
| `disabled` | boolean | false | Completely disable |

```toml
[ui.help_panel]
show_categories = true
hidden = true
disabled = false
```

### [ui.remote_control]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `show_channel_descriptions` | boolean | true | Show descriptions |
| `sort_alphabetically` | boolean | true | Alphabetical sort |
| `disabled` | boolean | false | Disable feature |

```toml
[ui.remote_control]
show_channel_descriptions = true
sort_alphabetically = true
disabled = false
```

## [keybindings]

Custom key mappings for this channel.

| Field | Type | Description |
|-------|------|-------------|
| `shortcut` | string | Global shortcut to switch to this channel |
| `<action>` | string or string[] | Override default keybinding |

```toml
[keybindings]
shortcut = "f1"  # Press F1 to switch to this channel

# Override defaults
quit = ["esc", "ctrl-c"]
select_next_entry = ["down", "ctrl-j"]

# Trigger custom actions
ctrl-e = "actions:edit"
ctrl-o = "actions:open"
```

## [actions.NAME]

Define custom actions that can be triggered by keybindings.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `description` | string | No | Action description |
| `command` | string | Yes | Command template |
| `mode` | string | No | "fork" (default) or "execute" |
| `separator` | string | No | Multi-select join character (default: " ") |

### Fork Mode (Return to tv)

```toml
[actions.view]
description = "View file in less"
command = "less '{}'"
mode = "fork"
```

### Execute Mode (Replace tv)

```toml
[actions.edit]
description = "Edit in nvim"
command = "nvim '{}'"
mode = "execute"
```

### Multi-Select with Custom Separator

```toml
[actions.delete]
description = "Delete selected files"
command = "rm {}"
mode = "fork"
separator = " "  # Files joined with spaces
```

## Complete Example

```toml
[metadata]
name = "docker-containers"
description = "Manage Docker containers"
requirements = ["docker"]

[source]
command = ["docker ps --format '{{.ID}}\\t{{.Names}}\\t{{.Status}}'",
           "docker ps -a --format '{{.ID}}\\t{{.Names}}\\t{{.Status}}'"]
display = "{split:\\t:1} | {split:\\t:2}"
output = "{split:\\t:0}"

[preview]
command = "docker inspect '{split:\\t:0}' | jq ."

[ui]
layout = "landscape"
[ui.preview_panel]
size = 55
header = "Container: {split:\\t:1}"

[keybindings]
shortcut = "f5"
ctrl-l = "actions:logs"
ctrl-x = "actions:stop"
ctrl-a = "actions:attach"

[actions.logs]
description = "View container logs"
command = "docker logs -f '{split:\\t:0}'"
mode = "fork"

[actions.stop]
description = "Stop container"
command = "docker stop '{split:\\t:0}'"
mode = "fork"

[actions.attach]
description = "Attach to container"
command = "docker exec -it '{split:\\t:0}' /bin/sh"
mode = "execute"
```

## Template Syntax

Templates use the [string-pipeline](https://docs.rs/string_pipeline) syntax. Common patterns:

| Pattern | Description |
|---------|-------------|
| `{}` | Entire entry |
| `{0}`, `{1}` | Positional fields (delimiter: `:`) |
| `{split:DELIM:INDEX}` | Split on custom delimiter |
| `{strip_ansi}` | Remove ANSI codes |
| `{trim}` | Remove whitespace |
| `{upper}`, `{lower}` | Case conversion |

For complete template documentation, see [Template System](../advanced/01-template-system.md).

## See Also

- [Creating your first channel](../getting-started/03-first-channel.md)
- [Template system](../advanced/01-template-system.md)
- [Actions reference](./02-actions.md)
