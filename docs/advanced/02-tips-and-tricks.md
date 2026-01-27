# Tips and Tricks

This guide covers advanced features and lesser-known capabilities.

## Source Cycling

Switch between multiple source commands within a single channel.

### Configuration

```toml
[source]
command = ["fd -t f", "fd -t f -H", "fd -t f -H -I"]
# 1. Normal files
# 2. Include hidden files
# 3. Include hidden + ignored files
```

### Usage

Press <kbd>Ctrl</kbd>+<kbd>S</kbd> (default) to cycle through source commands.

### Use Cases

- Toggle hidden files on/off
- Switch between different search depths
- Alternate sorting methods

## Preview Cycling

Similar to source cycling, but for preview commands.

### Configuration

```toml
[preview]
command = ["bat -n --color=always '{}'", "cat '{}'", "head -50 '{}'"]
```

### Usage

Press <kbd>Ctrl</kbd>+<kbd>F</kbd> (default) to cycle through preview commands.

### Use Cases

- Switch between syntax highlighting and raw view
- Toggle between full file and head/tail
- Different preview tools for different file types

## Watch Mode

Automatically reload the source at regular intervals.

### CLI Usage

```sh
tv files --watch 5.0  # Reload every 5 seconds
```

### Channel Configuration

```toml
[source]
command = "docker ps"
watch = 2.0  # Reload every 2 seconds
```

### Use Cases

- Monitor running processes
- Watch file changes
- Track container status
- Live log monitoring

## Inline Mode

Display tv as an inline element rather than fullscreen.

### Basic Inline

```sh
tv --inline
```

Uses available space at the bottom of the terminal.

### Fixed Height

```sh
tv --height 15  # 15 lines tall
```

### Fixed Width

```sh
tv --height 15 --width 80  # 15 lines, 80 columns
```

### Use Cases

- Embed in scripts without taking over the terminal
- Quick selections without fullscreen disruption
- Integration with tmux panes

## UI Scaling

Control how much screen space tv uses.

### CLI Usage

```sh
tv --ui-scale 80  # Use 80% of available space
```

### Configuration

```toml
[ui]
ui_scale = 70  # 70% of terminal
```

The UI is centered within the scaled area.

## Frecency Sorting

Entries are ranked by a combination of **frequency** and **recency** of use. Items you select often and recently appear higher in results.

Frecency is enabled by default and works automatically. The more you use tv, the smarter it gets at predicting what you want.

## Action Picker

Browse and execute available actions for the current entry.

### Usage

Press <kbd>Ctrl</kbd>+<kbd>X</kbd> to open the action picker, which shows all available actions for the current channel.

## Expect Keys

Define additional keys that can confirm selection, with the key name output first.

### CLI Usage

```sh
tv files --expect "ctrl-e,ctrl-v,ctrl-x"
```

If you press <kbd>Ctrl</kbd>+<kbd>E</kbd>, output is:
```
ctrl-e
selected_file.txt
```

### Use Cases

Useful for shell scripts that need to know *how* an item was selected:

```sh
output=$(tv files --expect "ctrl-e,ctrl-v")
key=$(echo "$output" | head -1)
file=$(echo "$output" | tail -1)

case "$key" in
  ctrl-e) nvim "$file" ;;
  ctrl-v) code "$file" ;;
  "") cat "$file" ;;  # Regular enter
esac
```

## Entry Selection Strategies

Control how tv handles single-result scenarios.

### Select If Only One (`--select-1`)

Auto-select and exit if there's exactly one match:

```sh
tv files --input "unique_file" --select-1
```

### Take First (`--take-1`)

Wait for loading, then auto-select the first entry:

```sh
tv files --take-1  # Always returns first result
```

### Take First Fast (`--take-1-fast`)

Return the first entry as soon as it appears (no waiting):

```sh
tv files --take-1-fast  # Fastest, returns immediately
```

### Use Cases

- Script automation
- Quick file selection
- Default value fallback

## Multiple Actions Per Key

Bind multiple actions to a single key:

```toml
[keybindings]
ctrl-r = ["reload_source", "copy_entry_to_clipboard"]
```

Actions execute in sequence.

## Exact/Substring Matching

Disable fuzzy matching for exact substring search:

```sh
tv files --exact
```

### When to Use

- Better performance on very large datasets
- When you know the exact string you're looking for
- Log searching where fuzzy matching creates noise

## Global vs Channel History

### Channel-Specific (Default)

History is scoped to each channel:

```sh
tv files  # Shows files history
tv text   # Shows text history
```

### Global History

Share history across all channels:

```sh
tv files --global-history
```

Or in config:

```toml
global_history = true
```

## Custom Input Prefill

Start with pre-filled search text:

```sh
tv files --input "src/"
```

### Use Cases

- Shell integration context
- Scripted searches
- Default filter patterns

## Panel Toggles

Control panel visibility at startup:

```sh
# Hide on startup (can toggle later)
tv --hide-preview
tv --hide-status-bar
tv --hide-remote

# Show on startup (overrides channel config)
tv --show-preview
tv --show-help-panel

# Disable entirely (can't toggle)
tv --no-preview
tv --no-remote
tv --no-help-panel
```

## Layout Control

Switch orientation at startup or runtime:

```sh
tv --layout portrait  # Preview below results
tv --layout landscape # Preview beside results (default)
```

Toggle with <kbd>Ctrl</kbd>+<kbd>L</kbd> at runtime.

## Custom Borders and Padding

Fine-tune panel appearance:

```sh
tv --preview-border thick --results-border none
tv --preview-padding "top=1;left=2;bottom=1;right=2"
```

Border options: `none`, `plain`, `rounded`, `thick`

## Source Entry Delimiter

Use custom delimiters for source output:

```sh
tv --source-command "find . -print0" --source-entry-delimiter "\0"
```

Useful for handling filenames with newlines or special characters.

## Preview Word Wrap

Enable word wrapping in the preview panel:

```sh
tv --preview-word-wrap
```

## Configuration File Override

Use a different config file:

```sh
tv --config-file ~/.config/television/minimal.toml
```

## Custom Cable Directory

Load channels from a different location:

```sh
tv --cable-dir ~/my-channels/
```

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `TELEVISION_CONFIG` | Override config directory |
| `TELEVISION_DATA` | Override data directory |
| `XDG_CONFIG_HOME` | XDG config base |
| `XDG_DATA_HOME` | XDG data base |

## Combining Features

These features compose well together:

```sh
# Watch docker containers, inline, with fast selection
tv docker-ps --watch 2.0 --inline --height 10

# File picker with preview, expect keys for different actions
tv files --preview-size 70 --expect "ctrl-e,ctrl-o" --ui-scale 80

# Quick exact search with auto-selection
tv files --input "config" --exact --select-1
```

## Performance Tips

1. **Use `--exact`** for large datasets when fuzzy isn't needed
2. **`--take-1-fast`** for fastest scripted selections
3. **`--no-preview`** when you don't need preview
4. **Limit source output** with flags like `fd --max-results 10000`

## What's Next?

- [Template system for complex formatting](./01-template-system.md)
- [Troubleshooting common issues](./03-troubleshooting.md)
