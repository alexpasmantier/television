# UI Features documentation

**Table of Contents**

- [Overview](#overview)
- [Architecture](#architecture)
- [Feature Components](#feature-components)
- [State Management](#state-management)
- [Configuration System](#configuration-system)
- [Examples](#examples)

## Overview

The UI Features System allows control over UI components using two properties:

- **Enabled/Disabled**: Whether the feature's functionality is available
- **Visible/Hidden**: Whether the feature is displayed in the interface

This design pattern allows for UI management where features can exist in three meaningful states: **Active** (enabled and visible), **Hidden** (enabled but not visible), and **Disabled** (completely inactive).

## Architecture

UI Features sit at the intersection of several core Television modules, acting as a central coordination point for UI state management.

### Context Diagram

```text
┌────────────┐    ┌────────────────────┐    ┌───────────────┐
│ CLI Module │───►│ UI Features System │◄───│ Config Module │
└────────────┘    └────────────────────┘    └───────────────┘
                            │
                            ▼
                    ┌───────────────┐
                    │ Screen/Layout │
                    │    Module     │
                    └───────────────┘
                            │
                            ▼
                      ┌───────────┐
                      │ UI Render │
                      │  System   │
                      └───────────┘
```

## Feature Components

It currently supports four primary UI features, each with distinct functionality and use cases.

In this view you can see the `Preview`, `Help Panel`, and `Status Bar`

```text
╭──────────────────────── Channel ─────────────────────────╮╭───────────────────────── PREVIEW ────────────────────────╮
│>                                                  1 / 1  ││                                                         ▲│
╰──────────────────────────────────────────────────────────╯│                                                         █│
╭──────────────────────── Results ─────────────────────────╮│                                                         ║│
│> TELEVISION                                              ││                                                         ║│
│                                                          ││                                                         ║│
│                                                          ││                                                         ║│
│                                                          ││                                                         ║│
│                                                          ││                  ╭─────────────── Help ────────────────╮║│
│                                                          ││                  │ Global                              │║│
│                                                          ││                  │ Quit: Esc                           │║│
│                                                          ││                  │ Quit: Ctrl-c                        │║│
│                                                          ││                  │ Toggle preview: Ctrl-o              │║│
│                                                          ││                  │ Toggle help: Ctrl-h                 │║│
│                                                          ││                  │ Toggle status bar: F12              │║│
│                                                          ││                  │                                     │║│
│                                                          ││                  │ Channel                             │║│
│                                                          ││                  │ Navigate up: Up                     │║│
│                                                          ││                  │ Navigate up: Ctrl-p                 │║│
│                                                          ││                  │ Navigate up: Ctrl-k                 │║│
│                                                          ││                  │ Navigate down: Down                 │║│
│                                                          ││                  │ Navigate down: Ctrl-n               │║│
│                                                          ││                  │ Navigate down: Ctrl-j               │║│
│                                                          ││                  │ ...                                 │║│
│                                                          ││                  ╰─────────────────────────────────────╯▼│
╰──────────────────────────────────────────────────────────╯╰──────────────────────────────────────────────────────────╯
  CHANNEL custom           [Hint] Remote Control: Ctrl-t • Hide Preview: Ctrl-o • Help: Ctrl-h                  v0.00.0
```

And here you can see the `Remote Control`

```text
╭────────────────────────────────────────────────────── Channel ───────────────────────────────────────────────────────╮
│>                                                                                                               1 / 1 │
╰──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯
╭────────────────────────────────────────────────────── Results ───────────────────────────────────────────────────────╮
│> TELEVISION                                                                                                          │
│                                                                                                                      │
│                      ╭──────── Channels ─────────╮╭────── Description ───────╮                                       │
│                      │> alias                    ││A channel to select from  │  _____________                        │
│                      │                           ││shell aliases             │ /             \                       │
│                      │                           ││                          │ | (*)     (#) |                       │
│                      │                           ││                          │ |             |                       │
│                      │                           ││                          │ | (1) (2) (3) |                       │
│                      │                           ││                          │ | (4) (5) (6) |                       │
│                      │                           ││                          │ | (7) (8) (9) |                       │
│                      │                           ││                          │ |      _      |                       │
│                      │                           ││                          │ |     | |     |                       │
│                      │                           ││                          │ |  (_¯(0)¯_)  |                       │
│                      │                           ││                          │ |     | |     |                       │
│                      │                           ││                          │ |      ¯      |                       │
│                      │                           ││                          │ |             |                       │
│                      │                           ││                          │ | === === === |                       │
│                      ╰───────────────────────────╯╰──────────────────────────╯ |             |                       │
│                      ╭───────── Search ──────────╮╭─── Requirements [OK] ────╮ |     TV      |                       │
│                      │>                          ││                          │ `-------------´                       │
│                      ╰───────────────────────────╯╰──────────────────────────╯                                       │
│                                                                                                                      │
│                                                                                                                      │
│                                                                                                                      │
╰──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯
  REMOTE                              [Hint] Back to Channel: Ctrl-t • Help: Ctrl-h                             v0.00.0
```

### Preview Panel

Displays contextual information about the currently selected entry

**Default State**: Enabled and Visible
**Configuration Files options**:

- `size`: Width percentage (1-99)
- `header`: Optional template for panel header
- `footer`: Optional template for panel footer
- `scrollbar`: Whether to show scroll indicators

**CLI Flags**: `--no-preview`, `--hide-preview`, `--show-preview`, `--preview-*` flags

### Status Bar

Shows application status, mode information, and available actions

**Default State**: Enabled and Visible
**Configuration**:

- `separator_open`: Opening separator character/string
- `separator_close`: Closing separator character/string

**CLI Controls**: `--no-status-bar`, `--hide-status-bar`, `--show-status-bar`

### Help Panel

Displays contextual help and keyboard shortcuts

**Default State**: Enabled but Hidden
**Configuration**:

- `show_categories`: Whether to group shortcuts by category

**CLI Controls**: `--no-help-panel`, `--hide-help-panel`, `--show-help-panel`

### Remote Control

Provides channel switching and management interface

**Default State**: Enabled but Hidden
**Configuration**:

- `show_channel_descriptions`: Include channel descriptions in listing
- `sort_alphabetically`: Sort channels alphabetically vs. by usage

**CLI Controls**: `--no-remote`, `--hide-remote`, `--show-remote`

## State Management

Logical state transitions with are enforced with built-in constraints:

```text
┌─────────────┐    enable()     ┌─────────────┐
│  Disabled   │────────────────►│   Active    │
│  enabled=F  │                 │  enabled=T  │
│  visible=F  │                 │  visible=T  │
└─────────────┘                 └─────────────┘
       ▲                               │
       │                               │ hide()
       │                               ▼
       │                        ┌─────────────┐
       └────────────────────────│   Hidden    │
         disable()              │  enabled=T  │
                                │  visible=F  │
                                └─────────────┘
                                       │
                                       │ show()
                                       ▼
                                ┌─────────────┐
                                │   Active    │
                                │  enabled=T  │
                                │  visible=T  │
                                └─────────────┘
```

## Configuration System

The UI Features system configuration follows a layered priority system:

1. **CLI Flags** (Highest Priority)
2. **Channel Configuration**
3. **User Configuration File**
4. **Built-in Defaults** (Lowest Priority)

### Configuration Formats

**TOML Configuration Syntax**

```toml
[ui.features]
preview_panel = { enabled = true, visible = true }
help_panel = { enabled = true, visible = false }
status_bar = { enabled = true, visible = true }
remote_control = { enabled = true, visible = false }

[ui.preview_panel]
size = 50
header = "{}"
footer = ""
scrollbar = true

[ui.status_bar]
separator_open = ""
separator_close = ""

[ui.remote_control]
show_channel_descriptions = true
sort_alphabetically = true
```

### Configuration Inheritance

**User Global Configuration**

```toml
# ~/.config/television/config.toml
[ui.features]
help_panel = { enabled = true, visible = true }  # Always show help for learning
```

**Channel-Level Configuration**

```toml
# ~/.config/television/cable/development.toml
[ui.features]
preview_panel = { enabled = true, visible = true }
status_bar = { enabled = true, visible = false }  # Hidden by default for focus
```

**Runtime Override Examples**

```bash
# Override channel defaults
tv development --show-status-bar --hide-preview

# Force features on/off
tv files --no-remote --show-help-panel

# Mixed visibility control
tv git-log --hide-status-bar --show-preview
```

### Default UI Feature States

| UI Feature         | Default Enabled | Default Visible | Rationale                                       |
| ------------------ | --------------- | --------------- | ----------------------------------------------- |
| **Preview Panel**  | ✅              | ✅              | Core functionality                              |
| **Status Bar**     | ✅              | ✅              | Shows mode and contextual hint to the user      |
| **Help Panel**     | ✅              | ❌              | Available on-demand to avoid clutter            |
| **Remote Control** | ✅              | ❌              | Available on-demand, disrupts regular operation |

### Feature State Persistence

**What Persists Across Sessions**

- ✅ **Configuration file settings** - Feature states defined in `~/.config/television/config.toml`
- ✅ **Channel-specific defaults** - Feature configurations built into channel definitions

**What Does Not Persist**

- ❌ **Runtime toggles** - Keyboard shortcuts like `Tab` (preview) or `F2` (status bar) are session-only
- ❌ **CLI flag overrides** - `--hide-preview`, `--show-status-bar` etc. apply only to current session
- ❌ **Temporary state changes** - Any feature visibility changes made during application use

## Examples

### Basic Feature Control

**Hide Preview Panel**

```bash
tv files --hide-preview
```

**Disable All Optional Features**

```bash
tv files --no-preview --no-status-bar --no-remote --no-help-panel
```

**Show All Features**

```bash
tv files --show-preview --show-status-bar --show-help-panel
```

### Channel-Specific Configuration

**Create Development Channel with Custom Features**

```toml
# ~/.config/television/cable/dev-focused.toml
[ui.features]
preview_panel = { enabled = true, visible = true }
status_bar = { enabled = true, visible = false }    # Clean interface
help_panel = { enabled = true, visible = false }    # Help on-demand
remote_control = { enabled = false }                # Single-channel focus
```

**Usage**

```bash
tv dev-focused /path/to/project
```

### Runtime Feature Management

**Quick Interface Cleanup**

```bash
# Start with full interface
tv files

# Runtime toggles (using default keybindings):
# Ctrl+O - Toggle preview panel
# F12    - Toggle status bar
# Ctrl-H - Toggle help panel
```
