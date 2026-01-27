# Keybindings

## Default Keybindings

Default keybindings are as follows:

|                                                              Key                                                              | Description                                        |
| :---------------------------------------------------------------------------------------------------------------------------: | -------------------------------------------------- |
| <kbd>↑</kbd> / <kbd>↓</kbd> or <kbd>Ctrl</kbd> + <kbd>p</kbd> / <kbd>n</kbd> or <kbd>Ctrl</kbd> + <kbd>k</kbd> / <kbd>j</kbd> | Navigate through the list of entries               |
|                                          <kbd>Ctrl</kbd> + <kbd>↑</kbd> / <kbd>↓</kbd>                                         | Navigate to previous / next history entry          |
|                                            <kbd>PageUp</kbd> / <kbd>PageDown</kbd>                                            | Scroll the preview pane by half a page             |
|                                                       <kbd>Enter</kbd>                                                        | Select the current entry                           |
|                                              <kbd>Tab</kbd> / <kbd>BackTab</kbd>                                              | Toggle selection and move to next / previous entry |
|                                                <kbd>Ctrl</kbd> + <kbd>y</kbd>                                                 | Copy the selected entry to the clipboard           |
|                                                <kbd>Ctrl</kbd> + <kbd>r</kbd>                                                 | Reload the current source                          |
|                                                <kbd>Ctrl</kbd> + <kbd>s</kbd>                                                 | Cycle through source commands (channel mode only)  |
|                                                <kbd>Ctrl</kbd> + <kbd>f</kbd>                                                 | Cycle through preview commands (channel mode only) |
|                                                <kbd>Ctrl</kbd> + <kbd>t</kbd>                                                 | Toggle remote control mode                         |
|                                                <kbd>Ctrl</kbd> + <kbd>h</kbd>                                                 | Toggle the help panel                              |
|                                                <kbd>Ctrl</kbd> + <kbd>o</kbd>                                                 | Toggle the preview panel                           |
|                                                       <kbd>F12</kbd>                                                         | Toggle the status bar                              |
|                                                <kbd>Ctrl</kbd> + <kbd>l</kbd>                                                 | Switch between landscape and portrait layout       |
|                                                <kbd>Ctrl</kbd> + <kbd>x</kbd>                                                 | Toggle the action picker                           |
|                                                <kbd>Esc</kbd> / <kbd>Ctrl</kbd> + <kbd>c</kbd>                                 | Quit the application                               |

### Input Editing Defaults

|                                        Key                                        | Description                               |
| :-------------------------------------------------------------------------------: | ----------------------------------------- |
|                                 <kbd>Backspace</kbd>                              | Delete the previous character             |
|                                    <kbd>Ctrl</kbd> + <kbd>w</kbd>                 | Delete the previous word                  |
|                                    <kbd>Ctrl</kbd> + <kbd>u</kbd>                 | Delete the current line                   |
|                                    <kbd>Delete</kbd>                              | Delete the next character                 |
|                                      <kbd>←</kbd> / <kbd>→</kbd>                  | Move the cursor left / right              |
|                                 <kbd>Home</kbd> / <kbd>End</kbd>                  | Move to the start / end of input          |
|                        <kbd>Ctrl</kbd> + <kbd>a</kbd> / <kbd>e</kbd>               | Move to the start / end of input          |

These keybindings are all configurable via tv's configuration file (see [Configuration](./02-configuration.md)).

# Keybindings Guide

Following this are some configuration presets you can use for your bindings. Most of these will probably match an existing program.

:::note
**This list is maintained by the community, so feel free to contribute your own ideas too! 😊**
:::

## Emacs

```toml
# Television already has some pretty Emacsy keybinds.
# This just makes them "Emacsier".
[keybindings]
scroll_preview_half_page_down = "alt-v"
scroll_preview_half_page_up = "ctrl-v"
toggle_remote_control = "alt-x" # Like execute-extended-command
toggle_help = "ctrl-h"

```
