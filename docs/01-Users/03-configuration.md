# Configuration

TV uses a single configuration file written in [TOML](https://toml.io/en/) format to manage its core settings.

## User configuration

Locations where `television` expects the user configuration file to be located for each platform:

| Platform |                 Value                  |
| -------- | :------------------------------------: |
| Linux    | `$HOME/.config/television/config.toml` |
| macOS    | `$HOME/.config/television/config.toml` |
| Windows  |   `%LocalAppData%\television\config`   |

Or, if you'd rather use the XDG Base Directory Specification, tv will look for the configuration file in
`$XDG_CONFIG_HOME/television/config.toml` if the environment variable is set.

## Default configuration file

**latest default config file: [config.toml](https://github.com/alexpasmantier/television/blob/main/.config/config.toml)**

## Option reference

### General Settings

| Option            | Type    | Default   | Description                                                                                                              |
| ----------------- | ------- | --------- | ------------------------------------------------------------------------------------------------------------------------ |
| `tick_rate`       | integer | `50`      | Application tick rate in milliseconds. Controls how frequently the UI updates.                                           |
| `default_channel` | string  | `"files"` | The default channel to use when no channel is specified on the command line.                                             |
| `history_size`    | integer | `200`     | Maximum number of entries to keep in the search history. Set to `0` to disable history functionality.                    |
| `global_history`  | boolean | `false`   | When `true`, history navigation shows entries from all channels. When `false`, history is scoped to the current channel. |

### UI Configuration

Top-level UI settings under the `[ui]` section:

| Option        | Type            | Default       | Description                                                                    |
| ------------- | --------------- | ------------- | ------------------------------------------------------------------------------ |
| `ui_scale`    | integer (0-100) | `100`         | Percentage of terminal space to allocate for the Television UI.                |
| `orientation` | string          | `"landscape"` | UI orientation. Valid values: `"landscape"`, `"portrait"`.                     |
| `theme`       | string          | `"default"`   | Theme name to use for the UI. See [Available Themes](#available-themes) below. |

#### Available Themes

Built-in themes: `default`, `television`, `gruvbox-dark`, `gruvbox-light`, `catppuccin`, `nord-dark`, `solarized-dark`, `solarized-light`, `dracula`, `monokai`, `onedark`, `tokyonight`.

You can also create custom themes by placing them in `$CONFIG_DIR/television/themes/` (see [themes](./08-themes.md) for
details).

### UI Components

#### Input Bar (`[ui.input_bar]`)

| Option        | Type   | Default                                  | Description                                                              |
| ------------- | ------ | ---------------------------------------- | ------------------------------------------------------------------------ |
| `position`    | string | `"top"`                                  | Position of the input bar. Valid values: `"top"`, `"bottom"`.            |
| `prompt`      | string | `">"`                                    | The input prompt string displayed before user input.                     |
| `header`      | string | `null`                                   | Optional header text displayed above the input bar.                      |
| `border_type` | string | `"rounded"`                              | Border style. Valid values: `"none"`, `"plain"`, `"rounded"`, `"thick"`. |
| `padding`     | object | `{left: 0, right: 0, top: 0, bottom: 0}` | Padding around the input bar.                                            |

#### Status Bar (`[ui.status_bar]`)

| Option            | Type    | Default | Description                                  |
| ----------------- | ------- | ------- | -------------------------------------------- |
| `separator_open`  | string  | `""`    | Opening character for status bar separators. |
| `separator_close` | string  | `""`    | Closing character for status bar separators. |
| `hidden`          | boolean | `false` | Whether to hide the status bar by default.   |

#### Results Panel (`[ui.results_panel]`)

| Option        | Type   | Default                                  | Description                                                              |
| ------------- | ------ | ---------------------------------------- | ------------------------------------------------------------------------ |
| `border_type` | string | `"rounded"`                              | Border style. Valid values: `"none"`, `"plain"`, `"rounded"`, `"thick"`. |
| `padding`     | object | `{left: 0, right: 0, top: 0, bottom: 0}` | Padding around the results panel.                                        |

#### Preview Panel (`[ui.preview_panel]`)

| Option        | Type            | Default                                  | Description                                                                        |
| ------------- | --------------- | ---------------------------------------- | ---------------------------------------------------------------------------------- |
| `size`        | integer (0-100) | `50`                                     | Preview panel size as percentage of screen width (landscape) or height (portrait). |
| `header`      | string          | `null`                                   | Optional header template for the preview panel.                                    |
| `footer`      | string          | `null`                                   | Optional footer template for the preview panel.                                    |
| `scrollbar`   | boolean         | `true`                                   | Whether to show a scrollbar in the preview panel.                                  |
| `border_type` | string          | `"rounded"`                              | Border style. Valid values: `"none"`, `"plain"`, `"rounded"`, `"thick"`.           |
| `padding`     | object          | `{left: 0, right: 0, top: 0, bottom: 0}` | Padding around the preview panel.                                                  |
| `hidden`      | boolean         | `false`                                  | Whether to hide the preview panel by default.                                      |

#### Help Panel (`[ui.help_panel]`)

| Option            | Type    | Default | Description                                    |
| ----------------- | ------- | ------- | ---------------------------------------------- |
| `show_categories` | boolean | `true`  | Whether to split the help panel by categories. |
| `hidden`          | boolean | `true`  | Whether to hide the help panel by default.     |
| `disabled`        | boolean | `false` | Whether to completely disable the help panel.  |

#### Remote Control (`[ui.remote_control]`)

| Option                      | Type    | Default | Description                                                     |
| --------------------------- | ------- | ------- | --------------------------------------------------------------- |
| `show_channel_descriptions` | boolean | `true`  | Whether to show channel descriptions in remote control mode.    |
| `sort_alphabetically`       | boolean | `true`  | Whether to sort channels alphabetically in remote control mode. |
| `disabled`                  | boolean | `false` | Whether to disable the remote control feature.                  |

### Theme Overrides (`[ui.theme_overrides]`)

Override specific colors from the selected theme without creating a full theme file. Colors can be specified as ANSI color names (e.g., `"red"`, `"bright-blue"`) or hex values (e.g., `"#ff0000"`).

**Available ANSI colors**: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `bright-black`, `bright-red`, `bright-green`, `bright-yellow`, `bright-blue`, `bright-magenta`, `bright-cyan`, `bright-white`.

| Option                   | Description                                    |
| ------------------------ | ---------------------------------------------- |
| `background`             | Background color                               |
| `border_fg`              | Border foreground color                        |
| `text_fg`                | General text foreground color                  |
| `dimmed_text_fg`         | Dimmed text foreground color                   |
| `input_text_fg`          | Input text foreground color                    |
| `result_count_fg`        | Result count foreground color                  |
| `result_name_fg`         | Result name foreground color                   |
| `result_line_number_fg`  | Result line number foreground color            |
| `result_value_fg`        | Result value foreground color                  |
| `selection_bg`           | Selection background color                     |
| `selection_fg`           | Selection foreground color                     |
| `match_fg`               | Match highlight foreground color               |
| `preview_title_fg`       | Preview title foreground color                 |
| `channel_mode_fg`        | Channel mode indicator foreground color        |
| `channel_mode_bg`        | Channel mode indicator background color        |
| `remote_control_mode_fg` | Remote control mode indicator foreground color |
| `remote_control_mode_bg` | Remote control mode indicator background color |

### Keybindings (`[keybindings]`)

Map keyboard keys to actions. Keys can be specified as:

- Single characters: `a`, `b`, `1`, etc.
- Special keys: `enter`, `esc`, `tab`, `backtab`, `space`, `backspace`, `delete`, `home`, `end`, `pageup`, `pagedown`, `up`, `down`, `left`, `right`
- Control keys: `ctrl-a`, `ctrl-b`, `ctrl-c`, etc.
- Function keys: `f1`, `f2`, ..., `f12`

**Available Actions**:

| Action                          | Description                             |
| ------------------------------- | --------------------------------------- |
| `delete_prev_char`              | Delete the character before the cursor  |
| `delete_prev_word`              | Delete the previous word                |
| `delete_next_char`              | Delete the character after the cursor   |
| `delete_line`                   | Delete the current line                 |
| `go_to_prev_char`               | Move cursor to previous character       |
| `go_to_next_char`               | Move cursor to next character           |
| `go_to_input_start`             | Move cursor to start of input           |
| `go_to_input_end`               | Move cursor to end of input             |
| `toggle_selection_down`         | Toggle selection and move down          |
| `toggle_selection_up`           | Toggle selection and move up            |
| `confirm_selection`             | Confirm current selection               |
| `select_next_entry`             | Select next entry in results            |
| `select_prev_entry`             | Select previous entry in results        |
| `select_next_page`              | Select next page of results             |
| `select_prev_page`              | Select previous page of results         |
| `copy_entry_to_clipboard`       | Copy selected entry to clipboard        |
| `scroll_preview_up`             | Scroll preview up by one line           |
| `scroll_preview_down`           | Scroll preview down by one line         |
| `scroll_preview_half_page_up`   | Scroll preview up by half page          |
| `scroll_preview_half_page_down` | Scroll preview down by half page        |
| `quit`                          | Quit the application                    |
| `toggle_remote_control`         | Toggle remote control mode              |
| `toggle_help`                   | Toggle help panel                       |
| `toggle_status_bar`             | Toggle status bar visibility            |
| `toggle_preview`                | Toggle preview panel visibility         |
| `toggle_layout`                 | Switch between landscape and portrait   |
| `cycle_sources`                 | Cycle through available source commands |
| `reload_source`                 | Reload the current source               |
| `select_prev_history`           | Navigate to previous history entry      |
| `select_next_history`           | Navigate to next history entry          |

### Event Bindings (`[events]`)

Map non-keyboard events to actions:

| Event               | Description             |
| ------------------- | ----------------------- |
| `mouse-scroll-up`   | Mouse scroll up event   |
| `mouse-scroll-down` | Mouse scroll down event |

### Shell Integration (`[shell_integration]`)

This section is a very quick overview of the shell integration options. For more details on what this is and how to set
it up, see [this page](./05-shell-integration.md).

Configure shell integration for smart command completion:

| Option             | Type   | Default   | Description                                     |
| ------------------ | ------ | --------- | ----------------------------------------------- |
| `fallback_channel` | string | `"files"` | Default channel when no command trigger matches |

#### Channel Triggers (`[shell_integration.channel_triggers]`)

Map shell commands to specific channels. Format: `channel_name = ["command1", "command2"]`

**Example**:

```toml
[shell_integration.channel_triggers]
"git-branch" = ["git checkout", "git branch"]
"files" = ["cat", "less", "vim"]
```

#### Shell Integration Keybindings (`[shell_integration.keybindings]`)

| Option               | Type   | Default    | Description                                       |
| -------------------- | ------ | ---------- | ------------------------------------------------- |
| `smart_autocomplete` | string | `"ctrl-t"` | Keybinding to trigger smart autocomplete in shell |
| `command_history`    | string | `"ctrl-r"` | Keybinding to trigger command history search      |
