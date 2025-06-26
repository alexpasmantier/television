# Television Features System
The Television Features System is a UI component management framework that provides dual-state control over interface elements. This document provides an in-depth analysis of the features functionality, its architecture, purpose, and configuration.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Module Analysis](#module-analysis)
- [Feature Components](#feature-components)
- [State Management](#state-management)
- [Configuration System](#configuration-system)
- [Integration Points](#integration-points)
- [Runtime Behavior](#runtime-behavior)
- [Advanced Usage](#advanced-usage)
- [Examples](#examples)

## Overview

Television's Features System enables granular control over UI components through a dual-state paradigm. Each feature can be independently controlled along two dimensions:

- **Enabled/Disabled**: Whether the feature's functionality is available
- **Visible/Hidden**: Whether the feature is displayed in the interface

This design pattern allows for UI management where features can exist in three meaningful states: **Active** (enabled and visible), **Hidden** (enabled but not visible), and **Disabled** (completely inactive).

### Key Benefits

- **Flexible UI Control**: Users can customize their interface without losing functionality
- **Runtime Toggleability**: Features can be dynamically shown/hidden during application use  
- **Configuration Persistence**: Feature states defined in configuration files persist across sessions
- **CLI Override Support**: Command-line flags can override configured feature states for individual sessions
- **Channel-Specific Defaults**: Different channels can have different feature configurations

## Architecture

The Features System sits at the intersection of several core Television modules, acting as a central coordination point for UI state management.

### Context Diagram

```text
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   CLI Module    │───►│ Features System │◄───│ Config Module   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │  Screen/Layout  │
                       │     Module      │
                       └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │    UI Render    │
                       │     System      │
                       └─────────────────┘
```

#### Integration Dependencies

| Module | Relationship | Purpose |
|--------|--------------|---------|
| **`config`** | Consumes | Feature state persistence and default values |
| **`cli`** | Modifies | Command-line overrides for feature states |
| **`screen/layout`** | Queries | Determines which components to render |
| **`action`** | Triggers | Runtime toggling via `Toggle<Feature>` actions |  
| **`channels`** | Inherits | Channel-specific feature configurations |
| **`keybindings`** | Binds | Keyboard shortcuts for feature toggling |

## Module Analysis

### Core Module: `television/features.rs`

The central features module defines the complete feature management system.

#### Key Components

**`FeatureState` Struct**
```rust
pub struct FeatureState {
    pub enabled: bool,
    pub visible: bool,
}
```

- **Purpose**: Represents the dual-state nature of each feature
- **Methods**: Provides convenience constructors (`enabled()`, `disabled()`, `hidden()`) and state manipulation (`toggle_enabled()`, `toggle_visible()`, `show()`, `hide()`)
- **Logic**: Enforces the rule that disabled features are automatically hidden

**`Features` Struct**
```rust
pub struct Features {
    pub preview_panel: FeatureState,
    pub help_panel: FeatureState, 
    pub status_bar: FeatureState,
    pub remote_control: FeatureState,
}
```

- **Purpose**: Container for all feature states in the application
- **Design**: Each UI component gets its own dedicated field
- **Methods**: Provides unified interface for querying and modifying feature states

**`FeatureFlags` Enum**
```rust
pub enum FeatureFlags {
    PreviewPanel,
    HelpPanel,
    StatusBar,
    RemoteControl,
}
```

- **Purpose**: Type-safe identifiers for features used in actions and runtime operations
- **Usage**: Enables compile-time verification of feature references
- **Serialization**: Supports configuration file parsing and storage

### Configuration Integration: `television/config/ui.rs`

The UI configuration module integrates features into the broader configuration system.

#### Integration Points

**`UiConfig` Struct**
```rust
pub struct UiConfig {
    // ... other UI settings ...
    pub features: Features,
    
    // Feature-specific configurations
    pub status_bar: StatusBarConfig,
    pub preview_panel: PreviewPanelConfig,
    pub help_panel: HelpPanelConfig,
    pub remote_control: RemoteControlConfig,
}
```

- **Design Pattern**: Features struct controls visibility/enablement while separate config structs control feature behavior
- **Modularity**: Each feature can have its own complex configuration independent of its state
- **Defaults**: Provides sensible defaults for both feature states and feature-specific settings

### Action System Integration: `television/action.rs`

Features integrate with Television's action system for runtime control.

**`Toggle<Feature>` Action**
```rust
pub enum Action {
    // ... other actions ...
    TogglePreview,
    ToggleHelp,
    ToggleStatusBar,
    ToggleRemoteControl,
    // ... more actions ...
}
```

- **Purpose**: Enables runtime toggling of feature visibility
- **Keybinding Support**: Can be bound to keyboard shortcuts
- **State Logic**: Toggles visibility for enabled features, no-op for disabled features

### Layout System Integration: `television/screen/layout.rs`

The layout system queries feature states to determine screen composition.

**Feature-Aware Layout Logic**
```rust
if ui_config.features.is_visible(FeatureFlags::PreviewPanel) {
    // Determine the desired preview percentage (as configured by the user) 
}

if ui_config.features.is_visible(FeatureFlags::StatusBar) {
    // Reserve space for status bar
}
```

- **Conditional Rendering**: Layout calculations only account for visible features
- **Dynamic Adaptation**: Screen real estate is redistributed when features are hidden
- **Performance**: Feature checks are lightweight boolean operations

## Feature Components

Television currently supports four primary UI features, each with distinct functionality and use cases.

```
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

```
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

**Purpose**: Displays contextual information about the currently selected entry

**Default State**: Enabled and Visible
**Configuration**: `PreviewPanelConfig`
- `size`: Width percentage (1-99)
- `header`: Optional template for panel header
- `footer`: Optional template for panel footer  
- `scrollbar`: Whether to show scroll indicators

**CLI Controls**: `--no-preview`, `--hide-preview`, `--show-preview`, `--preview-*` flags

**Behavior**:
- **Active**: Shows preview content based on `--preview-command`
- **Hidden**: Panel space is reclaimed by results list, but preview functionality remains available for toggling
- **Disabled**: No preview functionality, all preview-related keybindings are removed

### Status Bar

**Purpose**: Shows application status, mode information, and available actions

**Default State**: Enabled and Visible
**Configuration**: `StatusBarConfig`
- `separator_open`: Opening separator character/string
- `separator_close`: Closing separator character/string

**CLI Controls**: `--no-status-bar`, `--hide-status-bar`, `--show-status-bar`

**Behavior**:
- **Active**: Displays status information at bottom of screen
- **Hidden**: Status area is reclaimed for main content
- **Disabled**: No status information available

### Help Panel

**Purpose**: Displays contextual help and keyboard shortcuts

**Default State**: Enabled but Hidden
**Configuration**: `HelpPanelConfig`
- `show_categories`: Whether to group shortcuts by category

**CLI Controls**: `--no-help-panel`, `--hide-help-panel`, `--show-help-panel`

**Behavior**:
- **Active**: Shows help overlay when toggled
- **Hidden**: Help can be toggled on-demand
- **Disabled**: No help functionality available

### Remote Control

**Purpose**: Provides channel switching and management interface

**Default State**: Enabled but Hidden  
**Configuration**: `RemoteControlConfig`
- `show_channel_descriptions`: Include channel descriptions in listing
- `sort_alphabetically`: Sort channels alphabetically vs. by usage

**CLI Controls**: `--no-remote`, `--hide-remote`, `--show-remote`

**Behavior**:
- **Active**: Remote control interface is displayed
- **Hidden**: Channel switching available via keybindings but interface hidden
- **Disabled**: No channel switching functionality

## State Management

### State Transitions

The Features System enforces logical state transitions with built-in constraints:

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

### State Manipulation Methods



**`FeatureState` Methods**

| Method | Effect | Constraints |
|--------|--------|-------------|
| `enable()` | Sets `enabled=true`, `visible=true` | Always succeeds |
| `disable()` | Sets `enabled=false`, `visible=false` | Always succeeds |
| `show()` | Sets `visible=true` | Only if `enabled=true` |
| `hide()` | Sets `visible=false` | Always succeeds |
| `toggle_enabled()` | Flips enabled state, hides if disabling | Auto-hides when disabling |
| `toggle_visible()` | Flips visible state | Only if `enabled=true` |

**`Features` Collection Methods**

All `FeatureState` methods are available at the collection level:

```rust
features.enable(FeatureFlags::PreviewPanel);
features.hide(FeatureFlags::StatusBar);
features.toggle_visible(FeatureFlags::HelpPanel);
```

### Query Methods

**State Queries**

| Method | Returns | Purpose |
|--------|---------|---------|
| `is_active(flag)` | `enabled && visible` | Should component be rendered and functional? |
| `is_enabled(flag)` | `enabled` | Is functionality available (may be hidden)? |
| `is_visible(flag)` | `visible` | Should component be displayed? |

## Configuration System

### Configuration Hierarchy

Television's feature configuration follows a layered priority system:

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

### Default Feature States

| Feature | Default Enabled | Default Visible | Rationale |
|---------|----------------|-----------------|-----------|
| **Preview Panel** | ✅ | ✅ | Core functionality for file inspection |
| **Status Bar** | ✅ | ✅ | Important for user orientation |
| **Help Panel** | ✅ | ❌ | Available on-demand to avoid clutter |
| **Remote Control** | ✅ | ❌ | Channel switching available but interface hidden |

## Integration Points

### CLI Flag Processing

**Feature-Specific Flag Groups**

Each feature supports three CLI control patterns:

1. **Disable**: `--no-{feature}` - Completely turns off functionality
2. **Hide**: `--hide-{feature}` - Keeps functionality but hides interface
3. **Show**: `--show-{feature}` - Forces interface to be visible

**Flag Processing Logic** (in `television/main.rs`)
```rust
// Preview panel example
if args.no_preview {
    config.ui.features.disable(FeatureFlags::PreviewPanel);
} else if args.hide_preview {
    config.ui.features.hide(FeatureFlags::PreviewPanel);
} else if args.show_preview {
    config.ui.features.enable(FeatureFlags::PreviewPanel);
}
```

### Keybinding Integration

**Default Feature Keybindings** (from `television/screen/keybindings.rs`)

| Feature | Default Key | Action |
|---------|-------------|--------|
| Preview Panel | `Ctrl+O` | `ToggleFeature(PreviewPanel)` |
| Help Panel | `Ctrl+H` | `ToggleFeature(HelpPanel)` |
| Status Bar | `F12` | `ToggleFeature(StatusBar)` |
| Remote Control | `Ctrl+T` | `ToggleFeature(RemoteControl)` |

**Custom Keybinding Configuration**
```bash
tv files -k 'toggle_preview="p";toggle_help=["h","F10"]'
```

## Runtime Behavior

### Feature Toggle Actions

**Toggle Behavior**

| Feature State | Toggle Result | Effect |
|---------------|---------------|--------|
| **Active** | Hidden | Interface disappears, functionality retained |
| **Hidden** | Active | Interface appears, functionality restored |
| **Disabled** | No Change | Toggle ignored, no visual feedback |

### Dynamic Layout Recalculation

When features are toggled, the layout system automatically recalculates:

1. **Space Redistribution**: Hidden components' screen real estate is redistributed
2. **Constraint Adjustment**: Layout constraints are updated to reflect new component set  
3. **Focus Management**: Focus is redirected if the current component becomes hidden
4. **Render Optimization**: Only affected screen regions are redrawn

### Feature State Persistence

**What Persists Across Sessions**
- ✅ **Configuration file settings** - Feature states defined in `~/.config/television/config.toml`
- ✅ **Channel-specific defaults** - Feature configurations built into channel definitions

**What Does NOT Persist**
- ❌ **Runtime toggles** - Keyboard shortcuts like `Tab` (preview) or `F2` (status bar) are session-only
- ❌ **CLI flag overrides** - `--hide-preview`, `--show-status-bar` etc. apply only to current session
- ❌ **Temporary state changes** - Any feature visibility changes made during application use

**Persistence Examples**

```bash
# This setting persists across sessions
echo '[ui.features]
preview_panel = { enabled = true, visible = false }' >> ~/.config/television/config.toml

# These are session-only
tv files --hide-preview              # Temporary override
tv files                             # Back to configured defaults
# Press Tab to toggle preview        # Temporary toggle
# Exit and restart - preview is back to configured state
```

**Making Changes Persistent**

To make feature state changes permanent, you must edit configuration files:

```toml
# ~/.config/television/config.toml - Global defaults
[ui.features]
help_panel = { enabled = true, visible = true }  # Always show help

# ~/.config/television/cable/my-channel.toml - Channel-specific
[ui.features]
preview_panel = { enabled = true, visible = false }  # Start hidden for this channel
status_bar = { enabled = true, visible = false }     # Clean interface
```

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
tv files --show-preview --show-status-bar --show-remote --show-help-panel
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
# Ctrl+T - Toggle remote control
```