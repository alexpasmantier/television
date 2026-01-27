# Create Your First Channel

This tutorial walks you through creating a custom channel from scratch. By the end, you'll have a working channel with preview and custom actions.

## Prerequisites

This tutorial uses `tldr` (simplified man pages) as an example. Install it first:

```sh
# macOS
brew install tldr

# Arch Linux
pacman -S tldr

# Using npm
npm install -g tldr

# Using pip
pip install tldr
```

Run `tldr --list` to verify it works before continuing.

## What is a Channel?

A channel is a TOML configuration file that tells tv:
- **What to search** (the source command)
- **How to preview** results (optional)
- **Custom keybindings and actions** (optional)

Channels live in `~/.config/television/cable/` (or `%LocalAppData%\television\cable\` on Windows).

## Step 1: Create the Channel File

Let's build a channel to browse and preview TLDR pages (quick command references).

Create a new file:

```sh
mkdir -p ~/.config/television/cable
touch ~/.config/television/cable/tldr.toml
```

## Step 2: Add Basic Configuration

Open `tldr.toml` and add the minimum required fields:

```toml
[metadata]
name = "tldr"
description = "Browse and preview TLDR help pages"
requirements = ["tldr"]

[source]
command = "tldr --list"
```

**What this does:**
- `name`: Identifier used to invoke the channel (`tv tldr`)
- `description`: Shown in the remote control and help
- `requirements`: Lists required binaries - tv checks these exist at runtime and warns if missing
- `source.command`: The shell command that produces searchable entries

**Test it:**

```sh
tv tldr
```

You should see a list of TLDR pages. Try typing to filter them.

## Step 3: Add Preview

Let's add a preview so you can see the content before selecting:

```toml
[metadata]
name = "tldr"
description = "Browse and preview TLDR help pages"
requirements = ["tldr"]

[source]
command = "tldr --list"

[preview]
command = "tldr '{}'"
```

The `{}` is a placeholder that gets replaced with the selected entry.

**Test it:**

```sh
tv tldr
```

Now when you navigate entries, you'll see the TLDR content in the preview panel.

## Step 4: Add a Custom Action

Let's add an action to open the TLDR page in a pager:

```toml
[metadata]
name = "tldr"
description = "Browse and preview TLDR help pages"
requirements = ["tldr"]

[source]
command = "tldr --list"

[preview]
command = "tldr '{}'"

[keybindings]
ctrl-e = "actions:open"

[actions.open]
description = "Open TLDR page in pager"
command = "tldr '{}' | less"
mode = "fork"
```

**What's new:**
- `keybindings.ctrl-e`: Maps Ctrl+E to trigger the "open" action
- `actions.open`: Defines a custom action with a command
- `mode = "fork"`: Runs the command and returns to tv when done

**Action modes:**
- `fork`: Run command, return to tv afterward
- `execute`: Replace tv with the command (doesn't return)

## Step 5: Customize the UI

Add some UI tweaks:

```toml
[metadata]
name = "tldr"
description = "Browse and preview TLDR help pages"
requirements = ["tldr"]

[source]
command = "tldr --list"

[preview]
command = "tldr '{}'"

[ui]
preview_panel = { size = 60 }  # 60% width for preview

[keybindings]
shortcut = "f3"  # Press F3 anywhere to switch to this channel
ctrl-e = "actions:open"

[actions.open]
description = "Open TLDR page in pager"
command = "tldr '{}' | less"
mode = "fork"
```

## Complete Channel

Here's the final channel:

```toml
[metadata]
name = "tldr"
description = "Browse and preview TLDR help pages"
requirements = ["tldr"]

[source]
command = "tldr --list"

[preview]
command = "tldr '{}'"

[ui]
preview_panel = { size = 60 }

[keybindings]
shortcut = "f3"
ctrl-e = "actions:open"

[actions.open]
description = "Open TLDR page in pager"
command = "tldr '{}' | less"
mode = "fork"
```

## More Examples

### Docker Containers Channel

```toml
[metadata]
name = "docker-containers"
description = "Browse and manage Docker containers"
requirements = ["docker"]

[source]
command = "docker ps -a --format '{{.Names}}\\t{{.Status}}\\t{{.Image}}'"

[preview]
command = "docker inspect '{split:\\t:0}'"

[keybindings]
ctrl-l = "actions:logs"
ctrl-x = "actions:stop"

[actions.logs]
description = "View container logs"
command = "docker logs -f '{split:\\t:0}'"
mode = "fork"

[actions.stop]
description = "Stop container"
command = "docker stop '{split:\\t:0}'"
mode = "fork"
```

### Recently Modified Files

```toml
[metadata]
name = "recent-files"
description = "Recently modified files"
requirements = ["fd"]

[source]
command = "fd -t f --changed-within 7d"

[preview]
command = "bat -n --color=always '{}'"
```

## Channel Specification Quick Reference

| Section | Purpose |
|---------|---------|
| `[metadata]` | Channel name, description, requirements |
| `[source]` | Command that produces entries |
| `[preview]` | Command to generate previews |
| `[ui]` | Layout and display options |
| `[keybindings]` | Custom key mappings |
| `[actions.*]` | Custom action definitions |

## What's Next?

- [Full channel specification](../reference/03-channel-spec.md)
- [Template system for complex formatting](../advanced/01-template-system.md)
- [Community channels for inspiration](../community/channels-unix.md)
