# Troubleshooting

This guide helps diagnose and resolve common issues with television.

## Installation Issues

### Command Not Found

If `tv` isn't recognized after installation:

1. **Check PATH**: Ensure the installation directory is in your PATH
2. **New shell session**: Open a new terminal or run `source ~/.bashrc` (or equivalent)
3. **Verify installation**: `which tv` or `where tv` (Windows)

### Permission Denied

```sh
chmod +x /path/to/tv  # Linux/macOS
```

### Cargo Install Fails

```sh
# Update Rust
rustup update

# Try with --locked flag
cargo install --locked television
```

## Display Issues

### Colors Look Wrong

1. **Check terminal support**: Ensure your terminal supports 256 colors or truecolor
2. **TERM variable**: Try `export TERM=xterm-256color`
3. **Theme mismatch**: Some themes assume dark/light background

```toml
[ui]
theme = "default"  # Try different themes
```

### Characters Not Rendering

Ensure your terminal font supports the characters tv uses. Try a Nerd Font or similar.

### UI Garbled or Misaligned

1. **Terminal resize**: Try resizing your terminal
2. **Clear screen**: Press <kbd>Ctrl</kbd>+<kbd>L</kbd> or restart tv
3. **Alternate screen**: Some terminals don't support alternate screen properly

```sh
# Force redraw
tv files  # Then Ctrl+L
```

### Preview Not Showing

1. **No preview configured**: Some channels don't have previews
2. **Preview command missing**: Install required tools (bat, cat, etc.)
3. **Preview hidden**: Press <kbd>Ctrl</kbd>+<kbd>O</kbd> to toggle
4. **Preview disabled**: Check for `--no-preview` or `hidden = true` in config

## Channel Issues

### Channel Not Found

```sh
# List available channels
tv list-channels

# Update community channels
tv update-channels
```

### Source Command Fails

Test the source command directly:

```sh
# If channel uses: fd -t f
fd -t f  # Run it directly to see errors
```

Common causes:
- Missing required tool (check `requirements` in channel)
- Wrong working directory
- Permission issues

### No Results Showing

1. **Empty source**: Source command returns nothing
2. **Filter too restrictive**: Clear your input
3. **Wrong directory**: Check working directory

```sh
# Start in specific directory
tv files /path/to/search
```

### Preview Command Fails

Test preview command manually:

```sh
# If preview uses: bat -n --color=always '{}'
bat -n --color=always "test_file.txt"
```

## Configuration Issues

### Config Not Loading

Check config location:

```sh
# Linux/macOS
ls -la ~/.config/television/config.toml

# Or with custom location
echo $TELEVISION_CONFIG
```

### Config Syntax Error

Validate TOML syntax:

```sh
# Install a TOML validator
pip install toml-cli
toml-cli validate ~/.config/television/config.toml
```

Common TOML errors:
- Missing quotes around strings with special characters
- Unclosed brackets
- Invalid escape sequences

### Keybindings Not Working

1. **Check binding syntax**: Use lowercase (`ctrl-a`, not `Ctrl-A`)
2. **Conflicts**: Channel keybindings override global ones
3. **Terminal intercepts**: Some terminals capture certain key combinations

Test a simple binding:

```toml
[keybindings]
f1 = "toggle_help"  # F1 is rarely intercepted
```

## Shell Integration Issues

### Ctrl+T / Ctrl+R Not Working

1. **Init not sourced**: Ensure `eval "$(tv init zsh)"` is in your shell config
2. **Reload shell**: `source ~/.zshrc` or open new terminal
3. **Key conflict**: Another tool might be using these keys

```sh
# Check if tv init generates output
tv init zsh
```

### Wrong Channel Selected

Adjust channel triggers in config:

```toml
[shell_integration.channel_triggers]
"files" = ["cat", "vim", "code"]
"dirs" = ["cd", "ls"]
```

## Performance Issues

### Slow Startup

1. **Source command slow**: Test the source command independently

### Memory Usage

1. **Limit source output**: try piping into `head -n N` in source command. Start with N=100.
3. **Disable caching**: `--no-cache-preview`

## Logs

Tv writes logs to help diagnose issues:

| Platform | Location |
|----------|----------|
| Linux | `~/.local/share/television/television.log` |
| macOS | `~/Library/Application Support/television/television.log` |
| Windows | `%LocalAppData%\television\television.log` |

Or if `$TELEVISION_DATA` is set: `$TELEVISION_DATA/television.log`

To see logs during development:

```sh
tail -f ~/.local/share/television/television.log
```

## Debug Mode

Run tv in debug mode for more information:

```sh
RUST_LOG=debug tv files
```

And check the logs.

## Getting Help

If you can't resolve an issue:

1. **Check GitHub Issues**: [television issues](https://github.com/alexpasmantier/television/issues)
2. **Discord**: Join the community Discord
3. **Create an issue**: Include:
   - tv version (`tv --version`)
   - OS and terminal
   - Steps to reproduce
   - Relevant config/channel files
   - Log output

## Reset to Defaults

If all else fails, start fresh:

```sh
# Backup and remove config
mv ~/.config/television ~/.config/television.bak

# Run tv - which creates new default config
tv files
```
