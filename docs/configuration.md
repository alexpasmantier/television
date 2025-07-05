# Configuration

## Default configuration reference

TV's configuration is done through a single TOML file, which allows you to customize the behavior and appearance of the
application.

**Default configuration: [config.toml](https://github.com/alexpasmantier/television/blob/main/.config/config.toml)**

## User configuration

Locations where `television` expects the user configuration file to be located for each platform:

| Platform |                 Value                  |
| -------- | :------------------------------------: |
| Linux    | `$HOME/.config/television/config.toml` |
| macOS    | `$HOME/.config/television/config.toml` |
| Windows  |   `%LocalAppData%\television\config`   |

Or, if you'd rather use the XDG Base Directory Specification, tv will look for the configuration file in
`$XDG_CONFIG_HOME/television/config.toml` if the environment variable is set.
