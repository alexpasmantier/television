use clap::{Parser, Subcommand, ValueEnum};

/// Television CLI arguments structure.
///
/// This CLI supports two primary modes of operation:
///
/// # Channel Mode (when `channel` is specified)
/// In this mode, the specified channel provides base configuration (source commands,
/// preview commands, UI settings, etc.) and CLI flags act as **overrides** to those defaults.
/// This mode is more permissive and allows any combination of flags since they override
/// sensible channel defaults.
///
/// # Ad-hoc Mode (when `channel` is not specified)
/// In this mode, the CLI creates a custom channel on-the-fly based on the provided flags.
/// This mode has **stricter validation** to ensure the resulting channel is functional:
/// - Source-related flags (`--source-display`, `--source-output`) require `--source-command`
/// - Preview-related flags (`--preview-*`) require `--preview-command`
///
/// This validation prevents creating broken ad-hoc channels that reference non-existent commands.
#[allow(clippy::struct_excessive_bools)]
#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Which channel shall we watch?
    ///
    /// When specified: The application operates in 'channel mode' where the selected
    /// channel provides the base configuration, and CLI flags act as overrides.
    ///
    /// When omitted: The application operates in 'ad-hoc mode' where you must provide
    /// at least --source-command to create a custom channel. In this mode, preview
    /// and source flags have stricter validation rules.
    ///
    /// A list of the available channels can be displayed using the
    /// `list-channels` command. The channel can also be changed from within
    /// the application.
    #[arg(index = 1, verbatim_doc_comment)]
    pub channel: Option<String>,

    /// A preview line number offset template to use to scroll the preview to for each
    /// entry.
    ///
    /// When a channel is specified: This overrides the offset defined in the channel prototype.
    /// When no channel is specified: This flag requires --preview-command to be set.
    ///
    /// This template uses the same syntax as the `preview` option and will be formatted
    /// using the currently selected entry.
    #[arg(long, value_name = "STRING", verbatim_doc_comment)]
    pub preview_offset: Option<String>,

    /// Disable the preview panel entirely on startup.
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    /// When set, no preview panel will be shown regardless of channel configuration
    /// or preview-related flags.
    #[arg(long, default_value = "false", verbatim_doc_comment, conflicts_with_all = ["preview_offset", "preview_header", "preview_footer", "preview_size", "preview_command", "hide_preview", "show_preview"])]
    pub no_preview: bool,

    /// Hide the preview panel on startup (only works if feature is enabled).
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    /// The preview remains functional and can be toggled visible later.
    #[arg(long, default_value = "false", verbatim_doc_comment, conflicts_with_all = ["no_preview", "show_preview"])]
    pub hide_preview: bool,

    /// Show the preview panel on startup (only works if feature is enabled).
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    /// This overrides any channel configuration that might have it disabled.
    #[arg(long, default_value = "false", verbatim_doc_comment, conflicts_with_all = ["no_preview", "hide_preview"])]
    pub show_preview: bool,

    /// Disable the status bar entirely on startup.
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    /// When set, no status bar will be shown regardless of channel configuration
    /// or status bar-related flags.
    #[arg(long, default_value = "false", verbatim_doc_comment, conflicts_with_all = ["hide_status_bar", "show_status_bar"])]
    pub no_status_bar: bool,

    /// Hide the status bar on startup (only works if feature is enabled).
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    /// The status bar remains functional and can be toggled visible later.
    #[arg(long, default_value = "false", verbatim_doc_comment, conflicts_with_all = ["no_status_bar", "show_status_bar"])]
    pub hide_status_bar: bool,

    /// Show the status bar on startup (only works if feature is enabled).
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    /// This overrides any channel configuration that might have it disabled.
    #[arg(long, default_value = "false", verbatim_doc_comment, conflicts_with_all = ["no_status_bar", "hide_status_bar"])]
    pub show_status_bar: bool,

    /// The application's tick rate.
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    ///
    /// The tick rate is the number of times the application will update per
    /// second. This can be used to control responsiveness and CPU usage on
    /// very slow machines or very fast ones but the default should be a good
    /// compromise for most users.
    #[arg(short, long, value_name = "INT", verbatim_doc_comment, value_parser = validate_positive_int)]
    pub tick_rate: Option<u64>,

    /// Watch mode: reload the source command every N seconds.
    ///
    /// When a channel is specified: Overrides the watch interval defined in the channel prototype.
    /// When no channel is specified: Sets the watch interval for the ad-hoc channel.
    ///
    /// When set to a positive number, the application will automatically
    /// reload the source command at the specified interval. This is useful
    /// for monitoring changing data sources. Set to 0 to disable (default).
    #[arg(long, value_name = "FLOAT", verbatim_doc_comment, value_parser = validate_non_negative_float, conflicts_with_all = ["select_1", "take_1", "take_1_fast"])]
    pub watch: Option<f64>,

    /// Keybindings to override the default keybindings.
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    ///
    /// This can be used to override the default keybindings with a custom subset.
    /// The keybindings are specified as a semicolon separated list of keybinding
    /// expressions using the configuration file formalism.
    ///
    /// Example: `tv --keybindings='quit="esc";select_next_entry=["down","ctrl-j"]'`
    #[arg(short, long, value_name = "STRING", verbatim_doc_comment)]
    pub keybindings: Option<String>,

    /// Keys that can be used to confirm the current selection in addition to the default ones
    /// (typically `enter`).
    ///
    /// When this is set, confirming the selection will first output an extra line with the key
    /// that was used to confirm the selection before outputting the selected entry.
    ///
    /// Example: `tv --expect='ctrl-q'` will output `ctr-q\n<selected_entry>` when `ctrl-q` is
    /// pressed to confirm the selection.
    #[arg(long, value_name = "STRING", verbatim_doc_comment)]
    pub expect: Option<String>,

    /// Input text to pass to the channel to prefill the prompt.
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    ///
    /// This can be used to provide a default value for the prompt upon
    /// startup.
    #[arg(short, long, value_name = "STRING", verbatim_doc_comment)]
    pub input: Option<String>,

    /// Input field header template.
    ///
    /// When a channel is specified: Overrides the input header defined in the channel prototype.
    /// When no channel is specified: Sets the input header for the ad-hoc channel.
    ///
    /// The given value is parsed as a `MultiTemplate`. It is evaluated against
    /// the current channel name and the resulting text is shown as the input
    /// field title. Defaults to the current channel name when omitted.
    #[arg(long, value_name = "STRING", verbatim_doc_comment)]
    pub input_header: Option<String>,

    /// Input prompt string
    ///
    /// When a channel is specified: This overrides the prompt defined in the channel prototype.
    /// When no channel is specified: Sets the input prompt for the ad-hoc channel.
    ///
    /// The given value is used as the prompt string shown before the input field.
    /// Defaults to ">" when omitted.
    #[arg(long, value_name = "STRING", verbatim_doc_comment)]
    pub input_prompt: Option<String>,

    /// Sets the input panel border type.
    ///
    /// Available options are: `none`, `plain`, `rounded`, `thick`.
    #[arg(long, value_enum, verbatim_doc_comment)]
    pub input_border: Option<BorderType>,

    /// Sets the input panel padding.
    ///
    /// Format: `top=INTEGER;left=INTEGER;bottom=INTEGER;right=INTEGER`
    ///
    /// Example: `--input-padding='top=1;left=2;bottom=1;right=2'`
    #[arg(long, value_name = "STRING", verbatim_doc_comment)]
    pub input_padding: Option<String>,

    /// Preview header template
    ///
    /// When a channel is specified: This overrides the header defined in the channel prototype.
    /// When no channel is specified: This flag requires --preview-command to be set.
    ///
    /// The given value is parsed as a `MultiTemplate`. It is evaluated for every
    /// entry and its result is displayed above the preview panel.
    #[arg(
        long,
        value_name = "STRING",
        verbatim_doc_comment,
        conflicts_with = "no_preview"
    )]
    pub preview_header: Option<String>,

    /// Preview footer template
    ///
    /// When a channel is specified: This overrides the footer defined in the channel prototype.
    /// When no channel is specified: This flag requires --preview-command to be set.
    ///
    /// The given value is parsed as a `MultiTemplate`. It is evaluated for every
    /// entry and its result is displayed below the preview panel.
    #[arg(
        long,
        value_name = "STRING",
        verbatim_doc_comment,
        conflicts_with = "no_preview"
    )]
    pub preview_footer: Option<String>,

    /// Sets the preview panel border type.
    ///
    /// Available options are: `none`, `plain`, `rounded`, `thick`.
    #[arg(
        long,
        value_enum,
        verbatim_doc_comment,
        conflicts_with = "no_preview"
    )]
    pub preview_border: Option<BorderType>,

    /// Sets the preview panel padding.
    ///
    /// Format: `top=INTEGER;left=INTEGER;bottom=INTEGER;right=INTEGER`
    ///
    /// Example: `--preview-padding='top=1;left=2;bottom=1;right=2'`
    #[arg(
        long,
        value_name = "STRING",
        verbatim_doc_comment,
        conflicts_with = "no_preview"
    )]
    pub preview_padding: Option<String>,

    /// Hide preview panel scrollbar.
    #[arg(
        long,
        default_value = "false",
        verbatim_doc_comment,
        conflicts_with = "no_preview"
    )]
    pub hide_preview_scrollbar: bool,

    /// Source command to use for the current channel.
    ///
    /// When a channel is specified: This overrides the command defined in the channel prototype.
    /// When no channel is specified: This creates an ad-hoc channel with the given command.
    ///
    /// Example: `find . -name '*.rs'`
    #[arg(short, long, value_name = "STRING", verbatim_doc_comment)]
    pub source_command: Option<String>,

    /// Whether tv should extract and parse ANSI style codes from the source command output.
    ///
    /// This is useful when the source command outputs colored text or other ANSI styles and you
    /// want `tv` to preserve them in the UI. It does come with a slight performance cost but
    /// which should go mostly unnoticed for typical human interaction workloads.
    ///
    /// Example: `tv --source-command="echo -e '\x1b[31mRed\x1b[0m'" --ansi`
    #[arg(long, default_value = "false", verbatim_doc_comment)]
    pub ansi: bool,

    /// Source display template to use for the current channel.
    ///
    /// When a channel is specified: This overrides the display template defined in the channel prototype.
    /// When no channel is specified: This flag requires --source-command to be set.
    ///
    /// The template is used to format each entry in the results list.
    /// Example: `{split:/:-1}` (show only the last path segment)
    #[arg(long, value_name = "STRING", verbatim_doc_comment)]
    pub source_display: Option<String>,

    /// Source output template to use for the current channel.
    ///
    /// When a channel is specified: This overrides the output template defined in the channel prototype.
    /// When no channel is specified: This flag requires --source-command to be set.
    ///
    /// The template is used to format the final output when an entry is selected.
    /// Example: "{}" (output the full entry)
    #[arg(long, value_name = "STRING", verbatim_doc_comment)]
    pub source_output: Option<String>,

    /// Sets the results panel border type.
    ///
    /// Available options are: `none`, `plain`, `rounded`, `thick`.
    #[arg(long, value_enum, verbatim_doc_comment)]
    pub results_border: Option<BorderType>,

    /// Sets the results panel padding.
    ///
    /// Format: `top=INTEGER;left=INTEGER;bottom=INTEGER;right=INTEGER`
    ///
    /// Example: `--results-padding='top=1;left=2;bottom=1;right=2'`
    #[arg(long, value_name = "STRING", verbatim_doc_comment)]
    pub results_padding: Option<String>,

    /// The delimiter byte to use for splitting the source's command output into entries.
    ///
    /// This can be useful when the source command outputs multiline entries and you want to
    /// rely on another delimiter to split the entries such a null byte or a custom character.
    #[arg(long, value_name = "STRING", verbatim_doc_comment)]
    pub source_entry_delimiter: Option<String>,

    /// Preview command to use for the current channel.
    ///
    /// When a channel is specified: This overrides the preview command defined in the channel prototype.
    /// When no channel is specified: This enables preview functionality for the ad-hoc channel.
    ///
    /// Example: "cat {}" (where {} will be replaced with the entry)
    ///
    /// Parts of the entry can be extracted positionally using the `delimiter`
    /// option.
    /// Example: "echo {0} {1}" will split the entry by the delimiter and pass
    /// the first two fields to the command.
    #[arg(
        short,
        long,
        value_name = "STRING",
        verbatim_doc_comment,
        conflicts_with = "no_preview"
    )]
    pub preview_command: Option<String>,

    /// Whether to cache the preview command output for each entry.
    ///
    /// This can be useful when the preview command is expensive to run
    /// and you want to avoid running it multiple times for the same entry.
    #[arg(
        long,
        default_value = "false",
        verbatim_doc_comment,
        conflicts_with = "no_preview"
    )]
    pub cache_preview: bool,

    /// Layout orientation for the UI.
    ///
    /// When a channel is specified: Overrides the layout/orientation defined in the channel prototype.
    /// When no channel is specified: Sets the layout orientation for the ad-hoc channel.
    ///
    /// Options are "landscape" or "portrait".
    #[arg(long, value_enum, verbatim_doc_comment)]
    pub layout: Option<LayoutOrientation>,

    /// The working directory to start the application in.
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    ///
    /// This can be used to specify a different working directory for the
    /// application to start in. This is useful when the application is
    /// started from a different directory than the one the user wants to
    /// interact with.
    #[arg(value_name = "PATH", index = 2, verbatim_doc_comment)]
    pub working_directory: Option<String>,

    /// Try to guess the channel from the provided input prompt.
    ///
    /// This flag automatically selects channel mode by guessing the appropriate channel.
    /// It conflicts with manually specifying a channel since it determines the channel automatically.
    ///
    /// This can be used to automatically select a channel based on the input
    /// prompt by using the `shell_integration` mapping in the configuration
    /// file.
    #[arg(long, value_name = "STRING", verbatim_doc_comment)]
    pub autocomplete_prompt: Option<String>,

    /// Use substring matching instead of fuzzy matching.
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    ///
    /// This will use substring matching instead of fuzzy matching when
    /// searching for entries. This is useful when the user wants to search for
    /// an exact match instead of a fuzzy match e.g. to improve performance.
    #[arg(long, default_value = "false", verbatim_doc_comment)]
    pub exact: bool,

    /// Automatically select and output the first entry if there is only one
    /// entry.
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    ///
    /// Note that most channels stream entries asynchronously which means that
    /// knowing if there's only one entry will require waiting for the channel
    /// to finish loading first.
    ///
    /// For most channels and workloads this shouldn't be a problem since the
    /// loading times are usually very short and will go unnoticed by the user.
    #[arg(
        long,
        default_value = "false",
        group = "selection_mode",
        verbatim_doc_comment
    )]
    pub select_1: bool,

    /// Take the first entry from the list after the channel has finished loading.
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    ///
    /// This will wait for the channel to finish loading all entries and then
    /// automatically select and output the first entry. Unlike `select_1`, this
    /// will always take the first entry regardless of how many entries are available.
    #[arg(
        long,
        default_value = "false",
        group = "selection_mode",
        verbatim_doc_comment
    )]
    pub take_1: bool,

    /// Take the first entry from the list as soon as it becomes available.
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    ///
    /// This will immediately select and output the first entry as soon as it
    /// appears in the results, without waiting for the channel to finish loading.
    /// This is the fastest option when you just want the first result.
    #[arg(
        long,
        default_value = "false",
        group = "selection_mode",
        verbatim_doc_comment
    )]
    pub take_1_fast: bool,

    /// Disable the remote control.
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    ///
    /// This will disable the remote control panel and associated actions
    /// entirely. This is useful when the remote control is not needed or
    /// when the user wants `tv` to run in single-channel mode (e.g. when
    /// using it as a file picker for a script or embedding it in a larger
    /// application).
    #[arg(long, default_value = "false", verbatim_doc_comment, conflicts_with_all = ["hide_remote", "show_remote"])]
    pub no_remote: bool,

    /// Hide the remote control on startup (only works if feature is enabled).
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    /// The remote control remains functional and can be toggled visible later.
    #[arg(long, default_value = "false", verbatim_doc_comment, conflicts_with_all = ["no_remote", "show_remote"])]
    pub hide_remote: bool,

    /// Show the remote control on startup (only works if feature is enabled).
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    #[arg(long, default_value = "false", verbatim_doc_comment, conflicts_with_all = ["no_remote", "hide_remote"])]
    pub show_remote: bool,

    /// Disable the help panel entirely on startup.
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    /// When set, no help panel will be shown regardless of channel configuration
    /// or help panel-related flags.
    #[arg(long, default_value = "false", verbatim_doc_comment, conflicts_with_all = ["hide_help_panel", "show_help_panel"])]
    pub no_help_panel: bool,

    /// Hide the help panel on startup (only works if feature is enabled).
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    /// The help panel remains functional and can be toggled visible later.
    #[arg(long, default_value = "false", verbatim_doc_comment, conflicts_with_all = ["no_help_panel", "show_help_panel"])]
    pub hide_help_panel: bool,

    /// Show the help panel on startup (only works if feature is enabled).
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    /// This overrides any channel configuration that might have it disabled.
    #[arg(long, default_value = "false", verbatim_doc_comment, conflicts_with_all = ["no_help_panel", "hide_help_panel"])]
    pub show_help_panel: bool,

    /// Change the display size in relation to the available area.
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    ///
    /// This will crop the UI to a centered rectangle of the specified
    /// percentage of the available area.
    #[arg(
        long,
        value_name = "INTEGER",
        verbatim_doc_comment,
        value_parser = clap::value_parser!(u16).range(10..=100)
    )]
    pub ui_scale: Option<u16>,

    /// Percentage of the screen to allocate to the preview panel (1-99).
    ///
    /// When a channel is specified: This overrides any `preview_size` defined in configuration files or channel prototypes.
    /// When no channel is specified: This flag requires --preview-command to be set.
    #[arg(long, value_name = "INTEGER", verbatim_doc_comment, value_parser = clap::value_parser!(u16).range(1..=99), conflicts_with = "no_preview")]
    pub preview_size: Option<u16>,

    /// Provide a custom configuration file to use.
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    #[arg(long, value_name = "PATH", verbatim_doc_comment, value_parser = validate_file_path)]
    pub config_file: Option<String>,

    /// Provide a custom cable directory to use.
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    #[arg(long, value_name = "PATH", verbatim_doc_comment, value_parser = validate_directory_path)]
    pub cable_dir: Option<String>,

    /// Use global history instead of channel-specific history.
    ///
    /// This flag only works in channel mode.
    ///
    /// When enabled, history navigation will show entries from all channels.
    /// When disabled (default), history navigation is scoped to the current channel.
    #[arg(long, verbatim_doc_comment)]
    pub global_history: bool,

    /// Height in lines for non-fullscreen mode.
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    ///
    /// When specified, the picker will be displayed as a non-fullscreen interface.
    #[arg(
        long,
        value_name = "INTEGER",
        verbatim_doc_comment,
        conflicts_with = "inline",
        // minimum value with status-bar disabled is 6
        // TODO: revisit if/when input can be toggled
        value_parser = clap::value_parser!(u16).range(6..)
    )]
    pub height: Option<u16>,

    /// Width in columns for non-fullscreen mode.
    ///
    /// This flag can only be used in combination with --inline or --height.
    /// When specified, the picker will be constrained to the specified width.
    #[arg(
        long,
        value_name = "INTEGER",
        verbatim_doc_comment,
        value_parser = clap::value_parser!(u16).range(20..)
    )]
    pub width: Option<u16>,

    /// Use all available empty space at the bottom of the terminal as an inline interface.
    ///
    /// This flag works identically in both channel mode and ad-hoc mode.
    ///
    /// When enabled, the picker will be displayed as an inline interface that uses
    /// all available empty space at the bottom of the terminal. If there is insufficient
    /// space to meet the minimum height the terminal will scroll.
    #[arg(
        long,
        default_value = "false",
        verbatim_doc_comment,
        conflicts_with = "height"
    )]
    pub inline: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug, PartialEq, Clone)]
pub enum Command {
    /// Lists the available channels.
    ListChannels,
    /// Initializes shell completion ("tv init zsh")
    #[clap(name = "init")]
    InitShell {
        /// The shell for which to generate the autocompletion script
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Downloads the latest collection of default channel prototypes from github
    /// and saves them to the local configuration directory.
    UpdateChannels {
        /// Force update on already existing channels.
        #[arg(long, default_value = "false")]
        force: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Cmd,
    Nu,
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum LayoutOrientation {
    Landscape,
    Portrait,
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum BorderType {
    None,
    Plain,
    Rounded,
    Thick,
}

// Add validator functions
fn validate_positive_int(s: &str) -> Result<u64, String> {
    match s.parse::<u64>() {
        Ok(val) if val > 0 => Ok(val),
        Ok(_) => Err("Value must be a positive integer".to_string()),
        Err(_) => Err("Invalid integer format".to_string()),
    }
}

fn validate_non_negative_float(s: &str) -> Result<f64, String> {
    match s.parse::<f64>() {
        Ok(val) if val >= 0.0 => Ok(val),
        Ok(_) => Err("Value must be non-negative".to_string()),
        Err(_) => Err("Invalid number format".to_string()),
    }
}

fn validate_file_path(s: &str) -> Result<String, String> {
    use std::path::Path;
    let path = Path::new(s);
    if path.exists() && path.is_file() {
        Ok(s.to_string())
    } else {
        Err(format!("File does not exist: {}", s))
    }
}

fn validate_directory_path(s: &str) -> Result<String, String> {
    use std::path::Path;
    let path = Path::new(s);
    if path.exists() && path.is_dir() {
        Ok(s.to_string())
    } else {
        Err(format!("Directory does not exist: {}", s))
    }
}
