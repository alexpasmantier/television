# Theme Color Overrides

This directory contains examples of how to use Television's theme color override feature.

## What are Theme Overrides?

Theme overrides allow you to customize specific colors from any theme without creating a complete custom theme file. This is perfect for making small adjustments to existing themes to better match your preferences or terminal setup.

## How to Use

1. **Copy the example configuration** to your Television config directory:
   ```bash
   cp examples/theme_overrides_example.toml ~/.config/television/config.toml
   ```

2. **Customize the overrides** by editing the `[ui.theme_overrides]` section in your config file.

3. **Restart Television** to see your changes.

## Example Configuration

See `theme_overrides_example.toml` for a complete example that demonstrates:

- Using a base theme (`gruvbox-dark`)
- Overriding background color to pure black
- Customizing text colors for better readability
- Making selection and match highlights more visible
- Using both hex colors (`#ff0000`) and ANSI color names (`red`)

## Available Color Properties

You can override any of these color properties:

### General Colors
- `background` - Background color
- `border_fg` - Border foreground color
- `text_fg` - General text color
- `dimmed_text_fg` - Dimmed text color

### Input Colors
- `input_text_fg` - Input text color
- `result_count_fg` - Result count color

### Result Colors
- `result_name_fg` - Result name color
- `result_line_number_fg` - Line number color
- `result_value_fg` - Result value color
- `selection_bg` - Selection background color
- `selection_fg` - Selection foreground color
- `match_fg` - Match highlight color

### Preview Colors
- `preview_title_fg` - Preview title color

### Mode Colors
- `channel_mode_fg` - Channel mode foreground color
- `channel_mode_bg` - Channel mode background color
- `remote_control_mode_fg` - Remote control mode foreground color
- `remote_control_mode_bg` - Remote control mode background color

## Color Formats

Colors can be specified in two formats:

1. **ANSI color names**: `"red"`, `"bright-blue"`, `"white"`, etc.
2. **Hex values**: `"#ff0000"`, `"#1e1e2e"`, `"#ffffff"`, etc.

## Use Cases

- **Dark backgrounds**: Override just the background for a darker appearance
- **Better contrast**: Adjust text colors for improved readability
- **Highlight matches**: Make search results more visible
- **Custom selection**: Match selection colors to your terminal theme
- **Accessibility**: Improve color contrast for better accessibility

## Tips

- Only specify the colors you want to change - others will keep their original values
- You can mix hex colors and ANSI color names in the same configuration
- Invalid colors will cause Television to fall back to the base theme
- Test your changes by restarting Television and checking the appearance 