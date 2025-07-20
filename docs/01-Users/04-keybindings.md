# Keybindings

Television supports two keybinding configuration formats:

1. **Binding syntax** - The modern, recommended format (key => action;)
2. **TOML format** - The legacy format (action = ["key1", "key2"])

Both formats are fully supported and can be used together. The binding syntax is recommended for new configurations due to its enhanced features like action blocks, event bindings, and channel-specific overrides.

## Default Keybindings

Default keybindings are as follows:

|                                                              Key                                                              | Description                                        |
| :---------------------------------------------------------------------------------------------------------------------------: | -------------------------------------------------- |
| <kbd>↑</kbd> / <kbd>↓</kbd> or <kbd>Ctrl</kbd> + <kbd>p</kbd> / <kbd>n</kbd> or <kbd>Ctrl</kbd> + <kbd>k</kbd> / <kbd>j</kbd> | Navigate through the list of entries               |
|                                         <kbd>PageUp</kbd> / <kbd>PageDown</kbd> or <kbd>Mouse Wheel ↑</kbd> / <kbd>Mouse Wheel ↓</kbd> | Scroll the preview pane up / down                  |
|                                                       <kbd>Enter</kbd>                                                        | Select the current entry                           |
|                                              <kbd>Tab</kbd> / <kbd>BackTab</kbd>                                              | Toggle selection and move to next / previous entry |
|                                                <kbd>Ctrl</kbd> + <kbd>y</kbd>                                                 | Copy the selected entry to the clipboard           |
|                                                <kbd>Ctrl</kbd> + <kbd>t</kbd>                                                 | Toggle remote control mode                         |
|                                                <kbd>Ctrl</kbd> + <kbd>h</kbd>                                                 | Toggle the help panel                              |
|                                                <kbd>Ctrl</kbd> + <kbd>o</kbd>                                                 | Toggle the preview panel                           |
|                                                        <kbd>Esc</kbd>                                                         | Quit the application                               |

These keybindings are all configurable via tv's configuration file (see [Configuration](./configuration)).

## Configuration Formats

### TOML Format

The traditional TOML format maps actions to lists of keys:

```toml
[keybindings]
quit = ["esc", "ctrl-c"]
select_next_entry = ["down", "ctrl-n", "ctrl-j"]
select_prev_entry = ["up", "ctrl-p", "ctrl-k"]
toggle_selection_down = "tab"
toggle_selection_up = "backtab"
confirm_selection = "enter"
toggle_remote_control = "ctrl-t"
toggle_preview = "ctrl-o"
toggle_help = "ctrl-h"
toggle_status_bar = "f12"
```

### Binding Syntax

The new binding syntax is an alternative way that uses that follows a key/event to action mapping:

```javascript
bindings {
    // === APPLICATION CONTROL ===
    esc => quit;
    ctrl-c => quit;

    // === NAVIGATION AND SELECTION ===
    down => select_next_entry;
    ctrl-n => select_next_entry;
    ctrl-j => select_next_entry;

    up => select_prev_entry;
    ctrl-p => select_prev_entry;
    ctrl-k => select_prev_entry;

    // === SELECTION ===
    tab => toggle_selection_down;
    backtab => toggle_selection_up;
    enter => confirm_selection;

    // === UI FEATURES ===
    ctrl-t => toggle_remote_control;
    ctrl-o => toggle_preview;
    ctrl-h => toggle_help;
    f12 => toggle_status_bar;
}
```

#### Comments

Full comment support with both single-line and multi-line comments:

```javascript
bindings {
    // Single-line comments
    ctrl-o => toggle_preview;  // inline comments are allowed too

    /* Multi-line comments
       are also supported */
    enter => confirm_selection;
}
```

#### Action Blocks

Execute multiple actions in sequence with a single key press:

```javascript
bindings {
    // Move to next entry and select the next two items
    f5 => {
        select_next_entry;
        toggle_selection_down;
        toggle_selection_down;
    };
}
```

> **Note**: Action blocks execute each action in order, one after another. This guarantees consistent behavior and prevents actions from interfering with each other.

#### Event Bindings

Respond to application events automatically:

```javascript
bindings {
    // Triggered when input stream is complete
    @load => reload_source;

    // Triggered when filtering is complete
    @result => toggle_status_bar;

    // Triggered when multi-selection changes
    @selection-change => copy_entry_to_clipboard;

    // Triggered when there's only one match
    @one => select_and_exit;

    // Triggered when there's no match
    @zero => quit;
}
```

## Key Specifications

### Basic Keys

- **Movement**: `up`, `down`, `left`, `right`
- **Text**: `enter`, `esc`, `tab`, `space`, `backspace`, `delete`
- **Navigation**: `home`, `end`, `pageup`, `pagedown`, `insert`
- **Function**: `f1` through `f12`

### Character Keys

- **Letters**: `a` through `z`
- **Numbers**: `0` through `9`
- **Symbols**

### Modified Keys

- **Ctrl**: `ctrl-a`, `ctrl-space`, `ctrl-enter`, etc.
- **Alt**: `alt-a`, `alt-space`, `alt-enter`, etc.

### Special Keys

- `backtab` (Shift+Tab)
- `mouse-scroll-up`, `mouse-scroll-down`

## Available Actions

### Navigation

- `select_next_entry` - Move to next entry
- `select_prev_entry` - Move to previous entry
- `select_next_page` - Move to next page
- `select_prev_page` - Move to previous page
- `select_next_history` - Navigate forward in search history
- `select_prev_history` - Navigate backward in search history

### Selection

- `confirm_selection` - Confirm current selection
- `select_and_exit` - Select the entry currently under cursor and exit
- `toggle_selection_down` - Toggle selection and move down
- `toggle_selection_up` - Toggle selection and move up

### Input Navigation

- `go_to_input_start` - Move cursor to start of input
- `go_to_input_end` - Move cursor to end of input
- `go_to_next_char` - Move cursor to next character
- `go_to_prev_char` - Move cursor to previous character

### Input Editing

- `delete_line` - Delete the current line from input buffer
- `delete_next_char` - Delete character after cursor
- `delete_prev_char` - Delete character before cursor
- `delete_prev_word` - Delete previous word from input buffer

### Preview Control

- `scroll_preview_up` - Scroll preview pane up
- `scroll_preview_down` - Scroll preview pane down
- `scroll_preview_half_page_up` - Scroll preview half page up
- `scroll_preview_half_page_down` - Scroll preview half page down

### Application Control

- `quit` - Exit the application
- `suspend` - Suspend the application
- `resume` - Resume the application
- `reload_source` - Reload the current data source

### Feature Toggles

- `toggle_preview` - Toggle the preview pane
- `toggle_help` - Toggle the help panel
- `toggle_status_bar` - Toggle the status bar
- `toggle_remote_control` - Toggle remote control mode

### Data Operations

- `copy_entry_to_clipboard` - Copy selected entry to clipboard
- `cycle_sources` - Cycle through available data sources

### Special Actions

- `nil` - No operation/unbind action

## Event System

Events allow you to automatically trigger actions when certain application events occur.

### Available Events

- `@start` - Triggered once when Television starts up
- `@load` - Triggered when a channel finishes loading data
- `@result` - Triggered when search filtering completes
- `@one` - Triggered when search has exactly one match
- `@zero` - Triggered when search has no matches
- `@selection-change` - Triggered when multi-selection changes

## Migration Guide

### From TOML to New Syntax

**TOML format:**

```toml
[keybindings]
quit = ["esc", "ctrl-c"]
select_next_entry = ["down", "ctrl-j"]
toggle_preview = "ctrl-o"
```

**New syntax equivalent:**

```javascript
bindings {
    esc => quit;
    ctrl-c => quit;
    down => select_next_entry;
    ctrl-j => select_next_entry;
    ctrl-o => toggle_preview;
}
```

### Using Both Formats

You can use both formats together by creating a `bindings.tvb` file alongside your existing `config.toml`:

**Create**: `~/.config/television/bindings.tvb`

```javascript
bindings {
    up => select_next_entry;
    down => select_prev_entry;
    ctrl-c => quit;
    enter => confirm_selection;
}
```

**Keep your existing**: `~/.config/television/config.toml`

```toml
[ui]
theme = "default"

[keybindings]
copy_entry_to_clipboard = "ctrl-y"
```

Television automatically loads and merges both formats. The binding syntax takes precedence over TOML for conflicting bindings.

## Channel-Specific Bindings

The binding syntax supports channel-specific overrides that only apply when using specific channels:

```javascript
bindings {
    // Global bindings
    ctrl-c => quit;

    // Only active when using the "files" channel
    channel "files" {
        f8 => toggle_preview;
        f7 => copy_entry_to_clipboard;
    }

    // Pattern-based channel binding
    for_channels("dirs") {
        f1 => toggle_help;
        f5 => reload_source;
    }
}
```

## Example configuration used as reference

For a complete, up-to-date reference of all available keys, events, and actions, see the default configuration file:

- **In repository**: `.config/bindings.tvb`
- **Documentation header**: Contains comprehensive syntax guide and all available options

This file serves as the authoritative reference and is kept in sync with the latest features.

:::note
**This list is maintained by the community, so feel free to contribute your own ideas too! 😊**
:::
