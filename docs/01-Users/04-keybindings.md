# Keybindings

## Default Keybindings

Default keybindings are as follows:

|                                                              Key                                                              | Description                                        |
| :---------------------------------------------------------------------------------------------------------------------------: | -------------------------------------------------- |
| <kbd>↑</kbd> / <kbd>↓</kbd> or <kbd>Ctrl</kbd> + <kbd>p</kbd> / <kbd>n</kbd> or <kbd>Ctrl</kbd> + <kbd>k</kbd> / <kbd>j</kbd> | Navigate through the list of entries               |
|                                            <kbd>PageUp</kbd> / <kbd>PageDown</kbd>                                            | Scroll the preview pane up / down                  |
|                                                       <kbd>Enter</kbd>                                                        | Select the current entry                           |
|                                              <kbd>Tab</kbd> / <kbd>BackTab</kbd>                                              | Toggle selection and move to next / previous entry |
|                                                <kbd>Ctrl</kbd> + <kbd>y</kbd>                                                 | Copy the selected entry to the clipboard           |
|                                                <kbd>Ctrl</kbd> + <kbd>t</kbd>                                                 | Toggle remote control mode                         |
|                                                <kbd>Ctrl</kbd> + <kbd>h</kbd>                                                 | Toggle the help panel                              |
|                                                <kbd>Ctrl</kbd> + <kbd>o</kbd>                                                 | Toggle the preview panel                           |
|                                                        <kbd>Esc</kbd>                                                         | Quit the application                               |

These keybindings are all configurable via tv's configuration file (see [Configuration](./configuration)).

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
