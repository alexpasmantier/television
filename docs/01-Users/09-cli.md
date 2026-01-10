```text
A very fast, portable and hackable fuzzy finder for the terminal

Usage: tv [OPTIONS] [CHANNEL] [PATH] [COMMAND]

Commands:
  list-channels    Lists the available channels
  init             Initializes shell completion ("tv init zsh")
  update-channels  Downloads the latest collection of channel prototypes from github and saves them to the local configuration directory
  help             Print this message or the help of the given subcommand(s)

Arguments:
  [CHANNEL]
          Which channel shall we watch?
          
          Channels provide predefined configurations including source commands,
          preview commands, UI settings, and more.
          
          To list available channels, use the `list-channels` subcommand.
          
          To pull the latest collection of channels from github, use the
          `update-channels` subcommand.

  [PATH]
          The working directory to start the application in.
          
          Defaults to the current directory.

Options:
  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Source:
  -s, --source-command <STRING>
          Source command to use for the current channel.
          
          When a channel is specified: This overrides the command defined in the channel prototype.
          When no channel is specified: This creates an ad-hoc channel with the given command.
          
          Example: `find . -name '*.rs'`

      --ansi
          Whether tv should extract and parse ANSI style codes from the source command output.
          
          This is useful when the source command outputs colored text or other ANSI styles and you
          want `tv` to preserve them in the UI. It does come with a slight performance cost but
          which should go mostly unnoticed for typical human interaction workloads.
          
          Example: `tv --source-command="echo -e 'Red'" --ansi`

      --source-display <STRING>
          Source display template to use for the current channel.
          
          When a channel is specified: This overrides the display template defined in the channel prototype.
          When no channel is specified: This flag requires --source-command to be set.
          
          The template is used to format each entry in the results list.
          Example: `{split:/:-1}` (show only the last path segment)

      --source-output <STRING>
          Source output template to use for the current channel.
          
          When a channel is specified: This overrides the output template defined in the channel prototype.
          When no channel is specified: This flag requires --source-command to be set.
          
          The template is used to format the final output when an entry is selected.
          Example: "{}" (output the full entry)

      --source-entry-delimiter <STRING>
          The delimiter byte to use for splitting the source's command output into entries.
          
          This can be useful when the source command outputs multiline entries and you want to
          rely on another delimiter to split the entries such a null byte or a custom character.

Preview:
  -p, --preview-command <STRING>
          Preview command to use for the current channel.
          
          When a channel is specified: This overrides the preview command defined in the channel prototype.
          When no channel is specified: This enables preview functionality for the ad-hoc channel.
          
          Example: "cat {}" (where {} will be replaced with the entry)
          
          Parts of the entry can be extracted positionally using the `delimiter`
          option.
          Example: "echo {0} {1}" will split the entry by the delimiter and pass
          the first two fields to the command.

      --preview-header <STRING>
          Preview header template
          
          When a channel is specified: This overrides the header defined in the channel prototype.
          When no channel is specified: This flag requires --preview-command to be set.
          
          The given value is parsed as a `MultiTemplate`. It is evaluated for every
          entry and its result is displayed above the preview panel.

      --preview-footer <STRING>
          Preview footer template
          
          When a channel is specified: This overrides the footer defined in the channel prototype.
          When no channel is specified: This flag requires --preview-command to be set.
          
          The given value is parsed as a `MultiTemplate`. It is evaluated for every
          entry and its result is displayed below the preview panel.

      --cache-preview
          Whether to cache the preview command output for each entry.
          
          This can be useful when the preview command is expensive to run
          and you want to avoid running it multiple times for the same entry.
          
          This is enabled by default since most channels will benefit from it.
          
          This can be disabled for special cases e.g. where the preview command output changes
          frequently and/or you want live udpates.

      --preview-offset <STRING>
          A preview line number offset template to use to scroll the preview to for each
          entry.
          
          When a channel is specified: This overrides the offset defined in the channel prototype.
          When no channel is specified: This flag requires --preview-command to be set.
          
          This template uses the same syntax as the `preview` option and will be formatted
          using the currently selected entry.

      --no-preview
          Disable the preview panel entirely on startup.
          
          This flag works identically in both channel mode and ad-hoc mode.
          When set, no preview panel will be shown regardless of channel configuration
          or preview-related flags.

      --hide-preview
          Hide the preview panel on startup (only works if feature is enabled).
          
          This flag works identically in both channel mode and ad-hoc mode.
          The preview remains functional and can be toggled visible later.

      --show-preview
          Show the preview panel on startup (only works if feature is enabled).
          
          This flag works identically in both channel mode and ad-hoc mode.
          This overrides any channel configuration that might have it disabled.

      --preview-border <PREVIEW_BORDER>
          Sets the preview panel border type.
          
          Available options are: `none`, `plain`, `rounded`, `thick`.
          
          [possible values: none, plain, rounded, thick]

      --preview-padding <STRING>
          Sets the preview panel padding.
          
          Format: `top=INTEGER;left=INTEGER;bottom=INTEGER;right=INTEGER`
          
          Example: `--preview-padding='top=1;left=2;bottom=1;right=2'`

      --preview-word-wrap
          Enables preview panel word wrap.
          
          Example: `--preview-word-wrap`

      --hide-preview-scrollbar
          Hide preview panel scrollbar.

      --preview-size <INTEGER>
          Percentage of the screen to allocate to the preview panel (1-99).
          
          When a channel is specified: This overrides any `preview_size` defined in configuration files or channel prototypes.
          When no channel is specified: This flag requires --preview-command to be set.

Input:
  -i, --input <STRING>
          Input text to pass to the channel to prefill the prompt.
          
          This flag works identically in both channel mode and ad-hoc mode.
          
          This can be used to provide a default value for the prompt upon
          startup.

      --input-header <STRING>
          Input field header template.
          
          When a channel is specified: Overrides the input header defined in the channel prototype.
          When no channel is specified: Sets the input header for the ad-hoc channel.
          
          The given value is parsed as a `MultiTemplate`. It is evaluated against
          the current channel name and the resulting text is shown as the input
          field title. Defaults to the current channel name when omitted.

      --input-prompt <STRING>
          Input prompt string
          
          When a channel is specified: This overrides the prompt defined in the channel prototype.
          When no channel is specified: Sets the input prompt for the ad-hoc channel.
          
          The given value is used as the prompt string shown before the input field.
          Defaults to ">" when omitted.

      --input-position <INPUT_POSITION>
          Input bar position.
          
          Sets whether the input panel is shown at the top or bottom of the UI.
          
          [possible values: top, bottom]

      --input-border <INPUT_BORDER>
          Sets the input panel border type.
          
          [possible values: none, plain, rounded, thick]

      --input-padding <STRING>
          Sets the input panel padding.
          
          Format: `top=INTEGER;left=INTEGER;bottom=INTEGER;right=INTEGER`
          
          Example: `--input-padding='top=1;left=2;bottom=1;right=2'`

UI:
      --no-status-bar
          Disable the status bar entirely on startup.
          
          This flag works identically in both channel mode and ad-hoc mode.
          When set, no status bar will be shown regardless of channel configuration
          or status bar-related flags.

      --hide-status-bar
          Hide the status bar on startup (only works if feature is enabled).
          
          This flag works identically in both channel mode and ad-hoc mode.
          The status bar remains functional and can be toggled visible later.

      --show-status-bar
          Show the status bar on startup (only works if feature is enabled).
          
          This flag works identically in both channel mode and ad-hoc mode.
          This overrides any channel configuration that might have it disabled.

      --results-border <RESULTS_BORDER>
          Sets the results panel border type.
          
          [possible values: none, plain, rounded, thick]

      --results-padding <STRING>
          Sets the results panel padding.
          
          Format: `top=INTEGER;left=INTEGER;bottom=INTEGER;right=INTEGER`
          
          Example: `--results-padding='top=1;left=2;bottom=1;right=2'`

      --layout <LAYOUT>
          Layout orientation for the UI.
          
          When a channel is specified: Overrides the layout/orientation defined in the channel prototype.
          When no channel is specified: Sets the layout orientation for the ad-hoc channel.
          
          [possible values: landscape, portrait]

      --no-remote
          Disable the remote control.
          
          This flag works identically in both channel mode and ad-hoc mode.
          
          This will disable the remote control panel and associated actions
          entirely. This is useful when the remote control is not needed or
          when the user wants `tv` to run in single-channel mode (e.g. when
          using it as a file picker for a script or embedding it in a larger
          application).

      --hide-remote
          Hide the remote control on startup (only works if feature is enabled).
          
          This flag works identically in both channel mode and ad-hoc mode.
          The remote control remains functional and can be toggled visible later.

      --show-remote
          Show the remote control on startup (only works if feature is enabled).
          
          This flag works identically in both channel mode and ad-hoc mode.

      --no-help-panel
          Disable the help panel entirely on startup.
          
          This flag works identically in both channel mode and ad-hoc mode.
          When set, no help panel will be shown regardless of channel configuration
          or help panel-related flags.

      --hide-help-panel
          Hide the help panel on startup (only works if feature is enabled).
          
          This flag works identically in both channel mode and ad-hoc mode.
          The help panel remains functional and can be toggled visible later.

      --show-help-panel
          Show the help panel on startup (only works if feature is enabled).
          
          This flag works identically in both channel mode and ad-hoc mode.
          This overrides any channel configuration that might have it disabled.

      --ui-scale <INTEGER>
          Change the display size in relation to the available area.
          
          This flag works identically in both channel mode and ad-hoc mode.
          
          This will crop the UI to a centered rectangle of the specified
          percentage of the available area.

      --height <INTEGER>
          Height in lines for non-fullscreen mode.
          
          This flag works identically in both channel mode and ad-hoc mode.
          
          When specified, the picker will be displayed as a non-fullscreen interface.

      --width <INTEGER>
          Width in columns for non-fullscreen mode.
          
          This flag can only be used in combination with --inline or --height.
          When specified, the picker will be constrained to the specified width.

      --inline
          Use all available empty space at the bottom of the terminal as an inline interface.
          
          This flag works identically in both channel mode and ad-hoc mode.
          
          When enabled, the picker will be displayed as an inline interface that uses
          all available empty space at the bottom of the terminal. If there is insufficient
          space to meet the minimum height the terminal will scroll.

Behavior:
  -t, --tick-rate <INT>
          The application's tick rate.
          
          This flag works identically in both channel mode and ad-hoc mode.
          
          The tick rate is the number of times the application will update per
          second. This can be used to control responsiveness and CPU usage on
          very slow machines or very fast ones but the default should be a good
          compromise for most users.

      --watch <FLOAT>
          Watch mode: reload the source command every N seconds.
          
          When a channel is specified: Overrides the watch interval defined in the channel prototype.
          When no channel is specified: Sets the watch interval for the ad-hoc channel.
          
          When set to a positive number, the application will automatically
          reload the source command at the specified interval. This is useful
          for monitoring changing data sources. Set to 0 to disable (default).

      --autocomplete-prompt <STRING>
          Try to guess the channel from the provided input prompt.
          
          This flag automatically selects channel mode by guessing the appropriate channel.
          It conflicts with manually specifying a channel since it determines the channel automatically.
          
          This can be used to automatically select a channel based on the input
          prompt by using the `shell_integration` mapping in the configuration
          file.

      --exact
          Use substring matching instead of fuzzy matching.
          
          This flag works identically in both channel mode and ad-hoc mode.
          
          This will use substring matching instead of fuzzy matching when
          searching for entries. This is useful when the user wants to search for
          an exact match instead of a fuzzy match e.g. to improve performance.

      --select-1
          Automatically select and output the first entry if there is only one
          entry.
          
          This flag works identically in both channel mode and ad-hoc mode.
          
          Note that most channels stream entries asynchronously which means that
          knowing if there's only one entry will require waiting for the channel
          to finish loading first.
          
          For most channels and workloads this shouldn't be a problem since the
          loading times are usually very short and will go unnoticed by the user.

      --take-1
          Take the first entry from the list after the channel has finished loading.
          
          This flag works identically in both channel mode and ad-hoc mode.
          
          This will wait for the channel to finish loading all entries and then
          automatically select and output the first entry. Unlike `select_1`, this
          will always take the first entry regardless of how many entries are available.

      --take-1-fast
          Take the first entry from the list as soon as it becomes available.
          
          This flag works identically in both channel mode and ad-hoc mode.
          
          This will immediately select and output the first entry as soon as it
          appears in the results, without waiting for the channel to finish loading.
          This is the fastest option when you just want the first result.

Keybindings:
  -k, --keybindings <STRING>
          Keybindings to override the default keybindings.
          
          This flag works identically in both channel mode and ad-hoc mode.
          
          This can be used to override the default keybindings with a custom subset.
          The keybindings are specified as a semicolon separated list of keybinding
          expressions using the configuration file formalism.
          
          Example: `tv --keybindings='quit="esc";select_next_entry=["down","ctrl-j"]'`

      --expect <STRING>
          Keys that can be used to confirm the current selection in addition to the default ones
          (typically `enter`).
          
          When this is set, confirming the selection will first output an extra line with the key
          that was used to confirm the selection before outputting the selected entry.
          
          Example: `tv --expect='ctrl-q'` will output `ctr-q\n<selected_entry>` when `ctrl-q` is
          pressed to confirm the selection.

Configuration:
      --config-file <PATH>
          Provide a custom configuration file to use.
          
          This flag works identically in both channel mode and ad-hoc mode.

      --cable-dir <PATH>
          Provide a custom cable directory to use.
          
          This flag works identically in both channel mode and ad-hoc mode.

History:
      --global-history
          Use global history instead of channel-specific history.
          
          This flag only works in channel mode.
          
          When enabled, history navigation will show entries from all channels.
          When disabled (default), history navigation is scoped to the current channel.
```
