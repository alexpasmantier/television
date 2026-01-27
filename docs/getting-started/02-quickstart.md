# Quickstart

Welcome to Television! This guide will get you up and running in under 5 minutes.

## What is Television?

Television (`tv`) is a fast, portable fuzzy finder for the terminal. Think of it as a universal search tool that can search through:
- Files and directories
- Text content (like grep, but interactive)
- Git repositories, branches, logs
- Environment variables
- Docker containers
- And anything else you can pipe to it

## Basic Usage

### Your First Search

After [installing tv](./01-installation.md), open a terminal and run:

```sh
tv
```

This launches tv with the default channel (usually `files`). You'll see:
- An input bar at the top for typing your search
- A results panel showing matching entries
- A preview panel (if configured) showing content

**Try it:** Type a few characters and watch results filter in real-time.

### Using Built-in Channels

Channels are predefined search configurations. Try these common ones:

```sh
tv files      # Browse files (default)
tv text       # Search file contents with ripgrep
tv git-repos  # Find git repositories
tv env        # Search environment variables
```

To see all available channels:

```sh
tv list-channels
```

### Piping Data

Pipe any command's output into tv:

```sh
# Search through logs
cat /var/log/syslog | tv

# Filter git history
git log --oneline | tv

# Search command output
ps aux | tv
```

## Navigation and Selection

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| <kbd>↑</kbd> / <kbd>↓</kbd> | Navigate results |
| <kbd>Ctrl</kbd>+<kbd>j</kbd> / <kbd>k</kbd> | Navigate results (vim-style) |
| <kbd>Enter</kbd> | Select current entry |
| <kbd>Tab</kbd> | Toggle selection (multi-select) |
| <kbd>Ctrl</kbd>+<kbd>y</kbd> | Copy entry to clipboard |
| <kbd>PageUp</kbd> / <kbd>PageDown</kbd> | Scroll preview |
| <kbd>Ctrl</kbd>+<kbd>o</kbd> | Toggle preview panel |
| <kbd>Ctrl</kbd>+<kbd>h</kbd> | Show help panel |
| <kbd>Esc</kbd> or <kbd>Ctrl</kbd>+<kbd>c</kbd> | Quit |

### Multi-Select

Select multiple entries with <kbd>Tab</kbd>, then press <kbd>Enter</kbd> to output all selected items:

```sh
# Select multiple files, then open in editor
nvim $(tv files)
```

### Remote Control (Channel Switching)

Press <kbd>Ctrl</kbd>+<kbd>t</kbd> to open the "remote control" - a channel picker that lets you switch between channels without restarting tv.

## Ad-hoc Channels

Create temporary channels on the fly with CLI flags:

```sh
# Custom source command
tv --source-command "find . -name '*.rs'"

# With preview
tv --source-command "fd -t f" --preview-command "bat -n --color=always '{}'"

# Adjust preview size
tv --source-command "ls -la" --preview-command "file '{}'" --preview-size 70
```

## Search Patterns

Tv supports multiple search patterns:

| Pattern | Type | Example |
|---------|------|---------|
| `foo` | Fuzzy match | Matches "foo", "foobar", "folder_foo" |
| `'foo` | Substring (exact) | Contains "foo" exactly |
| `^foo` | Prefix | Starts with "foo" |
| `foo$` | Suffix | Ends with "foo" |
| `!foo` | Negate | Doesn't match "foo" |

Combine patterns with spaces (AND logic):

```
# Files containing "test", starting with "src", not ending with ".bak"
test ^src !.bak$
```

## Output and Integration

Tv outputs selected entries to stdout:

```sh
# Open selected file in editor
vim $(tv files)

# Change to selected directory
cd $(tv dirs)

# Copy selected file
cp $(tv files) /destination/
```

## Shell Integration

To enable shell integration:

```sh
# Zsh
echo 'eval "$(tv init zsh)"' >> ~/.zshrc

# Bash
echo 'eval "$(tv init bash)"' >> ~/.bashrc

# Fish
tv init fish | source  # Add to config.fish
```

This enables:
- <kbd>Ctrl</kbd>+<kbd>T</kbd>: Smart autocomplete based on current command
- <kbd>Ctrl</kbd>+<kbd>R</kbd>: Search shell history

## Updating Channels

Get the latest community channels:

```sh
tv update-channels
```

## What's Next?

- [Create your first custom channel](./03-first-channel.md)
- [Learn about configuration](../user-guide/02-configuration.md)
- [Explore keybinding customization](../user-guide/03-keybindings.md)
- [Tips and tricks](../advanced/02-tips-and-tricks.md)
