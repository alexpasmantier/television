# Television CLI Reference

Television (`tv`) is a cross-platform, fast and extensible general purpose fuzzy finder TUI. This document provides a comprehensive reference for all CLI options, modes, restrictions, and usage patterns.

## Table of Contents

- [Overview](#overview)
- [Operating Modes](#operating-modes)
- [Basic Usage](#basic-usage)
- [Arguments](#arguments)
- [Options](#options)
- [Subcommands](#subcommands)
- [Flag Dependencies and Restrictions](#flag-dependencies-and-restrictions)
- [Configuration](#configuration)
- [Template System](#template-system)
- [Examples](#examples)
- [Advanced Usage](#advanced-usage)

## Overview

Television supports two primary operating modes that determine how CLI flags are interpreted and validated:

1. **Channel Mode**: When a channel is specified, the application uses the channel's configuration as a base and CLI flags act as overrides
2. **Ad-hoc Mode**: When no channel is specified, the application creates a custom channel from CLI flags with stricter validation

## Operating Modes

### Channel Mode

**Activated when**: A channel name is provided as the first argument or via `--autocomplete-prompt`

**Behavior**:
- Channel provides base configuration (source commands, preview commands, UI settings)
- CLI flags act as **overrides** to channel defaults
- More permissive validation - allows most combination of flags
- Minimal dependency checking since channel provides sensible defaults

**Example**:
```bash
tv files --preview-command "bat -n --color=always {}"
```

### Ad-hoc Mode

**Activated when**: No channel is specified and no `--autocomplete-prompt` is used

**Behavior**:
- Creates a custom channel on-the-fly from CLI flags
- Requires `--source-command` to generate any entries
- **Stricter validation** ensures necessary components are present
- All functionality depends on explicitly provided flags

**Example**:
```bash
tv --source-command "find . -name '*.rs'" --preview-command "bat -n --color=always {}"
```

## Basic Usage

```
tv [OPTIONS] [CHANNEL] [PATH]
```

### Arguments

Television has intelligent positional argument handling with special path detection logic.

#### Position 1: `[CHANNEL]`
**Purpose**: Channel name to activate Channel Mode

- **Standard behavior**: When a valid channel name is provided, activates Channel Mode
- **Special path detection**: If the argument exists as a path on the filesystem, it's automatically treated as a working directory instead
- **Effect when path detected**: Switches to Ad-hoc Mode and uses the path as the working directory
- **Required**: No (falls back to `default_channel` from the global config)
- **Examples**:
  ```bash
  tv files               # Uses 'files' channel
  tv /home/user/docs     # Auto-detects path, uses as working directory
  tv ./projects          # Auto-detects relative path
  ```

#### Position 2: `[PATH]` 
**Purpose**: Working directory to start in

- **Behavior**: Sets the working directory for the application
- **Required**: No  
- **Precedence**: Only used if Position 1 was not detected as a path
- **Default**: Current directory
- **Example**: `tv files /home/user/projects`

#### ⚡ Smart Path Detection Logic

Television automatically detects when the first argument is a filesystem path:

1. **Path Check**: If Position 1 exists as a file or directory on the filesystem
2. **Mode Switch**: Automatically switches to Ad-hoc Mode (no channel)
3. **Directory Assignment**: Uses the detected path as the working directory
4. **Requirement**: When this happens, `--source-command` becomes required (Ad-hoc Mode rules apply)

**Examples of Smart Detection**:
```bash
# No arguments - uses default_channel from config
tv

# Channel name provided - Channel Mode
tv files

# Existing path provided - triggers path detection → uses default_channel
tv /home/user/docs    # Uses default_channel in /home/user/docs directory

# Non-existent path - treated as channel name → error if channel doesn't exist
tv /nonexistent/path  # Error: Channel not found

# Channel + explicit working directory - Channel Mode
tv files /home/user/docs

# The key nuance: same name, different behavior based on filesystem
tv myproject          # Channel Mode (if 'myproject' is a channel name)
tv ./myproject        # Channel Mode with default_channel (if './myproject' directory exists)

# Ambiguous case - path detection takes precedence
tv docs               # If 'docs' directory exists → default_channel + path detection
                      # If 'docs' directory doesn't exist → 'docs' channel
```

> **💡 Tip**: This smart detection makes Television intuitive - you can just specify a directory and it automatically knows you want to work in that location.

## Options

Television's options are organized by functionality. Each option behaves differently depending on whether you're using Channel Mode (with a channel specified) or Ad-hoc Mode (no channel).

### 🎯 Source and Data Options

#### `--source-command <STRING>`
**Purpose**: Defines the command that generates entries for the picker

- **Channel Mode**: Overrides the channel's default source command
- **Ad-hoc Mode**: ⚠️ **Required** - without this, no entries will be generated
- **Example**: `--source-command "find . -name '*.py'"`

#### `--source-display <STRING>`
**Purpose**: Template for formatting how entries appear in the results list

- **Channel Mode**: Overrides the channel's display format
- **Ad-hoc Mode**: Customize how entries are shown (default: entries as-is)
- **Requires**: `--source-command` (in ad-hoc mode)
- **Example**: `--source-display "{split:/:-1} ({split:/:0..-1|join:-})"`

#### `--source-output <STRING>`
**Purpose**: Template for formatting the final output when an entry is selected

- **Channel Mode**: Overrides the channel's output format
- **Ad-hoc Mode**: Customize what gets returned (default: entries as-is)
- **Requires**: `--source-command` (in ad-hoc mode)
- **Example**: `--source-output "code {}"`

### 👁️ Preview Options

#### `-p, --preview-command <STRING>`
**Purpose**: Command to generate preview content for the selected entry

- **Channel Mode**: Overrides the channel's preview command
- **Ad-hoc Mode**: Enables preview functionality with the specified command
- **Requires**: `--source-command` (in ad-hoc mode)
- **Conflicts**: Cannot use with `--no-preview`
- **Example**: `--preview-command "bat -n --color=always {}"`

#### `--preview-header <STRING>`
**Purpose**: Template for text displayed above the preview panel

- **Both Modes**: Sets custom header text
- **Requires**: `--preview-command` (in ad-hoc mode)
- **Conflicts**: Cannot use with `--no-preview`
- **Example**: `--preview-header "File: {split:/:-1|upper}"`

#### `--preview-footer <STRING>`
**Purpose**: Template for text displayed below the preview panel

- **Both Modes**: Sets custom footer text
- **Requires**: `--preview-command` (in ad-hoc mode)
- **Conflicts**: Cannot use with `--no-preview`

#### `--preview-offset <STRING>`
**Purpose**: Template that determines the scroll position in the preview

- **Both Modes**: Controls where preview content starts displaying
- **Requires**: `--preview-command` (in ad-hoc mode)
- **Conflicts**: Cannot use with `--no-preview`
- **Example**: `--preview-offset "10"` (start at line 10)

#### `--preview-size <INTEGER>`
**Purpose**: Width of the preview panel as a percentage

- **Both Modes**: Controls preview panel size
- **Default**: 50% of screen width
- **Range**: 1-99
- **Conflicts**: Cannot use with `--no-preview`

#### `--no-preview`
**Purpose**: Completely disables the preview panel

- **Both Modes**: Turns off all preview functionality
- **Conflicts**: Cannot use with any `--preview-*` flags
- **Use Case**: Faster performance, simpler interface

#### `--no-status-bar`
**Purpose**: Disables the bottom status bar on startup

- **Both Modes**: Hides the single-line status information bar
- **Use Case**: Minimal interface or very small terminals

### 🎨 Interface and Layout Options

#### `--layout <LAYOUT>`
**Purpose**: Controls the overall interface orientation

- **Channel Mode**: Overrides channel's layout setting
- **Ad-hoc Mode**: Sets interface layout
- **Values**: `landscape` (side-by-side), `portrait` (stacked)
- **Default**: `landscape`

#### `--input-header <STRING>`
**Purpose**: Template for text displayed above the input field

- **Channel Mode**: Overrides channel's input header
- **Ad-hoc Mode**: Sets custom input header
- **Default**: Channel name (channel mode) or empty (ad-hoc mode)
- **Example**: `--input-header "Search files:"`

#### `--ui-scale <INTEGER>`
**Purpose**: Scales the entire interface size

- **Both Modes**: Adjusts display size as a percentage
- **Default**: 100%
- **Range**: 10-100%
- **Use Case**: Adapt to different screen sizes or preferences

#### `--no-remote`
**Purpose**: Hides the remote control panel

- **Both Modes**: Removes remote control information from display
- **Use Case**: Simpler interface when remote features aren't needed

### ⌨️ Input and Interaction Options

#### `-i, --input <STRING>`
**Purpose**: Pre-fills the input prompt with specified text

- **Both Modes**: Starts with text already in the search box
- **Use Case**: Continue a previous search or provide default query
- **Example**: `-i "main.py"`

#### `-k, --keybindings <STRING>`
**Purpose**: Overrides default keyboard shortcuts

- **Both Modes**: Customizes key bindings for actions
- **Format**: `action1=["key1","key2"];action2=["key3"]`
- **Example**: `-k 'quit=["q","esc"];select=["enter","space"]'`

#### `--exact`
**Purpose**: Changes matching behavior from fuzzy to exact substring matching

- **Channel Mode**: Overrides channel's matching mode
- **Ad-hoc Mode**: Enables exact matching
- **Default**: Fuzzy matching
- **Use Case**: When you need precise substring matches

### ⚡ Selection Behavior Options

> **Note**: These options are mutually exclusive - only one can be used at a time.

#### `--select-1`
**Purpose**: Automatically selects and returns the entry if only one is found

- **Both Modes**: Bypasses interactive selection when there's only one match
- **Use Case**: Scripting scenarios where single results should be auto-selected

#### `--take-1`
**Purpose**: Takes the first entry after loading completes

- **Both Modes**: Automatically selects first item once all entries are loaded
- **Use Case**: Scripts that always want the first/best result

#### `--take-1-fast`
**Purpose**: Takes the first entry immediately as it appears

- **Both Modes**: Selects first item as soon as it's available
- **Use Case**: Maximum speed scripts that don't care about all options

### ⚙️ Performance and Monitoring Options

#### `-t, --tick-rate <FLOAT>`
**Purpose**: Controls how frequently the interface updates (times per second)

- **Both Modes**: Sets UI refresh rate
- **Default**: Auto-calculated based on system performance
- **Validation**: Must be positive number
- **Example**: `--tick-rate 30` (30 updates per second)

#### `--watch <FLOAT>`
**Purpose**: Automatically re-runs the source command at regular intervals

- **Channel Mode**: Overrides channel's watch interval
- **Ad-hoc Mode**: Enables live monitoring mode
- **Default**: 0 (disabled)
- **Units**: Seconds between updates
- **Conflicts**: Cannot use with selection options (`--select-1`, `--take-1`, `--take-1-fast`)
- **Example**: `--watch 2.0` (update every 2 seconds)

### 📁 Directory and Configuration Options

#### `[PATH]` (Positional Argument 2)
**Purpose**: Sets the working directory for the command

- **Both Modes**: Changes to specified directory before running
- **Default**: Current directory
- **Example**: `tv files /home/user/projects`

#### `--config-file <PATH>`
**Purpose**: Uses a custom configuration file instead of the default

- **Both Modes**: Loads settings from specified file
- **Default**: `~/.config/tv/config.toml` (Linux/macOS) or `%APPDATA%\tv\config.toml` (Windows)
- **Use Case**: Multiple configurations for different workflows

#### `--cable-dir <PATH>`
**Purpose**: Uses a custom directory for channel definitions

- **Both Modes**: Loads channels from specified directory
- **Default**: `~/.config/tv/cable/` (Linux/macOS) or `%APPDATA%\tv\cable\` (Windows)
- **Use Case**: Custom channel collections or shared team channels

### 🔧 Special Mode Options

#### `--autocomplete-prompt <STRING>`
**Purpose**: ⚡ **Activates Channel Mode** - Auto-detects channel from shell command

- **Effect**: Switches to Channel Mode automatically
- **Behavior**: Analyzes the provided command to determine appropriate channel
- **Conflicts**: Cannot use with `[CHANNEL]` positional argument
- **Use Case**: Shell integration and smart channel detection
- **Example**: `--autocomplete-prompt "git log --oneline"`

## Subcommands

### `list-channels`
Lists all available channels in the cable directory.

```bash
tv list-channels
```

### `init <SHELL>`
Generates shell completion script for the specified shell.

**Supported shells**: `bash`, `zsh`, `fish`, `powershell`, `cmd`

```bash
tv init zsh > ~/.zshrc.d/tv-completion.zsh
```

### `update-channels`
Downloads the latest channel prototypes from GitHub.

```bash
tv update-channels
```

## Usage Rules and Restrictions

> **Note**: Detailed requirements and conflicts for each flag are covered in the [Options](#options) section above. This section provides a high-level overview of the key rules.

### 🎯 Ad-hoc Mode Requirements

When using Television without a channel, certain flags become mandatory:

- **`--source-command` is required** - without this, no entries will be generated
- **Preview dependencies** - all `--preview-*` flags require `--preview-command` to be functional
- **Source formatting dependencies** - `--source-display` and `--source-output` require `--source-command`

### 🚫 Mutually Exclusive Options

These option groups cannot be used together:

- **Selection behavior**: Only one of `--select-1`, `--take-1`, or `--take-1-fast`
- **Preview control**: `--no-preview` conflicts with all `--preview-*` flags  
- **Channel selection**: Cannot use both `[CHANNEL]` argument and `--autocomplete-prompt`
- **Watch vs selection**: `--watch` cannot be used with auto-selection flags

### ✅ Channel Mode Benefits

Channels provide sensible defaults, making the tool more flexible:
- Preview and source flags work independently (channel provides missing pieces)
- All UI options have reasonable defaults
- Less strict validation since channels fill in the gaps

## Configuration

### ⚡ Configuration Priority

Television uses a layered configuration system where each layer can override the previous:

1. **CLI flags** - Highest priority, overrides everything
2. **Channel configuration** - Channel-specific settings  
3. **User config file** - Personal preferences
4. **Built-in defaults** - Fallback values

### 📁 Configuration Locations

#### User Configuration File
- **Linux/macOS**: `~/.config/tv/config.toml`
- **Windows**: `%APPDATA%\tv\config.toml`

#### Channel Definitions (Cable Directory)  
- **Linux/macOS**: `~/.config/tv/cable/`
- **Windows**: `%APPDATA%\tv\cable\`

> **Tip**: Use `--config-file` and `--cable-dir` flags to override these default locations

## Template System

Television uses a powerful template system for dynamic content generation. Templates are enclosed in curly braces `{}` and support complex operations.

### Template-Enabled Flags

| Flag Category | Flags Using Templates |
|---------------|----------------------|
| **Source** | `--source-command`, `--source-display`, `--source-output` |
| **Preview** | `--preview-command`, `--preview-offset` |
| **Headers** | `--input-header`, `--preview-header`, `--preview-footer` |

### Basic Template Syntax

Templates support a wide range of operations that can be chained together:

```text
{operation1|operation2|operation3}
```

### Core Template Operations

| Operation | Description | Example |
|-----------|-------------|---------|
| `{}` | Full entry (passthrough) | `{}` → original entry |
| `{split:SEPARATOR:RANGE}` | Split text and extract parts | `{split:/:‑1}` → last path component |
| `{upper}` | Convert to uppercase | `{upper}` → "HELLO" |
| `{lower}` | Convert to lowercase | `{lower}` → "hello" |
| `{trim}` | Remove whitespace | `{trim}` → "text" |
| `{append:TEXT}` | Add text to end | `{append:.txt}` → "file.txt" |
| `{prepend:TEXT}` | Add text to beginning | `{prepend:/home/}` → "/home/file" |

### Advanced Template Operations

| Operation | Description | Example |
|-----------|-------------|---------|
| `{replace:s/PATTERN/REPLACEMENT/FLAGS}` | Regex find and replace | `{replace:s/\\.py$/.backup/}` |
| `{regex_extract:PATTERN}` | Extract matching text | `{regex_extract:\\d+}` → extract numbers |
| `{filter:PATTERN}` | Keep items matching pattern | `{split:,:..\|filter:^test}` |
| `{sort}` | Sort list items | `{split:,:..\|sort}` |
| `{unique}` | Remove duplicates | `{split:,:..\|unique}` |
| `{join:SEPARATOR}` | Join list with separator | `{split:,:..\|join:-}` |

### Template Examples

```text
# File path manipulation
{split:/:-1}                    # Get filename from path
{split:/:0..-1|join:/}          # Get directory from path

# Text processing
{split: :..|map:{upper}|join:_} # "hello world" → "HELLO_WORLD"
{trim|replace:s/\s+/_/g}        # Replace spaces with underscores

# Data extraction
{regex_extract:@(.+)}           # Extract email domain
{split:,:..|filter:^[A-Z]}      # Filter items starting with uppercase
```

### Range Specifications

| Syntax | Description |
|--------|-------------|
| `N` | Single index (0-based) |
| `N..M` | Range exclusive (items N to M-1) |
| `N..=M` | Range inclusive (items N to M) |
| `N..` | From N to end |
| `..M` | From start to M-1 |
| `..` | All items |
| `-1` | Last item |
| `-N` | N-th from end |


For complete template documentation, see the [Template System Documentation](https://github.com/lalvarezt/string_pipeline/blob/main/docs/template-system.md).

## Examples

> **Note**: More detailed examples with explanations are included in each option's documentation above.

### 🎯 Quick Start Examples

#### Channel Mode (Recommended)
```bash
# Basic usage - use built-in channels
tv files                    # Browse files in current directory
tv git-log                  # Browse git commit history
tv docker-images            # Browse Docker images

# Channel + customization
tv files --preview-command "bat -n --color=always {}"
tv git-log --layout portrait
```

#### Ad-hoc Mode (Custom Commands)
```bash
# Simple custom finder
tv --source-command "find . -name '*.md'"

# Live system monitoring  
tv --source-command "ps aux | tail -n +2" \
   --watch 1.0 \
   --no-preview
```