# Themes

Builtin themes are available in the [themes](https://github.com/alexpasmantier/television/tree/main/themes) directory. Feel free to experiment and maybe even contribute your own!

|          ![catppuccin](../../assets/catppuccin.png "catppuccin") catppuccin           | ![gruvbox](../../assets/gruvbox.png "gruvbox") gruvbox-dark |
| :-----------------------------------------------------------------------------------: | :---------------------------------------------------------: |
| ![solarized-dark](../../assets/solarized-dark.png "gruvbox-light") **solarized-dark** |       ![nord](../../assets/nord.png "nord") **nord**        |

## Custom Themes

You may create your own custom themes by adding them to the `themes` directory in your configuration folder and then referring to them by file name (without the extension) in the configuration file.

```
config_location/
├── themes/
│   └── my_theme.toml
└── config.toml
```

_my_theme.toml_

```toml
# general
background = '#1e1e2e'
border_fg = '#6c7086'
text_fg = '#cdd6f4'
dimmed_text_fg = '#6c7086'
# input
input_text_fg = '#f38ba8'
result_count_fg = '#f38ba8'
# results
result_name_fg = '#89b4fa'
result_line_number_fg = '#f9e2af'
result_value_fg = '#b4befe'
selection_fg = '#a6e3a1'
selection_bg = '#313244'
match_fg = '#f38ba8'
# preview
preview_title_fg = '#fab387'
# modes
channel_mode_fg = '#1e1e2e'
channel_mode_bg = '#f5c2e7'
remote_control_mode_fg = '#1e1e2e'
remote_control_mode_bg = '#a6e3a1'
send_to_channel_mode_fg = '#89dceb'
```

## Theme Color Overrides

Instead of creating a complete custom theme, you can override specific colors from any theme directly in your configuration file. This is useful for making small adjustments to existing themes without creating a full theme file.

### Configuration

Add a `[ui.theme_overrides]` section to your `config.toml` file:

```toml
[ui]
theme = "gruvbox-dark"

[ui.theme_overrides]
background = "#000000"
text_fg = "#ffffff"
selection_bg = "#444444"
match_fg = "#ff0000"
```

### Available Color Properties

You can override any of the following color properties:

**General:**
- `background` - Background color
- `border_fg` - Border foreground color
- `text_fg` - General text color
- `dimmed_text_fg` - Dimmed text color

**Input:**
- `input_text_fg` - Input text color
- `result_count_fg` - Result count color

**Results:**
- `result_name_fg` - Result name color
- `result_line_number_fg` - Line number color
- `result_value_fg` - Result value color
- `selection_bg` - Selection background color
- `selection_fg` - Selection foreground color
- `match_fg` - Match highlight color

**Preview:**
- `preview_title_fg` - Preview title color

**Modes:**
- `channel_mode_fg` - Channel mode foreground color
- `channel_mode_bg` - Channel mode background color
- `remote_control_mode_fg` - Remote control mode foreground color
- `remote_control_mode_bg` - Remote control mode background color

### Color Formats

Colors can be specified in two formats:

1. **ANSI color names**: `"red"`, `"bright-blue"`, `"white"`, etc.
2. **Hex values**: `"#ff0000"`, `"#1e1e2e"`, `"#ffffff"`, etc.

### Example Use Cases

- **Dark background**: Override just the background color for a darker appearance
- **Highlight matches**: Change the match color to make search results more visible
- **Custom selection**: Modify selection colors to match your terminal theme
- **Accessibility**: Adjust text colors for better contrast

The overrides are applied on top of the base theme, so any colors you don't specify will retain their original values from the selected theme.
