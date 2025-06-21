use clap::{Parser, Subcommand, ValueEnum};

#[allow(clippy::struct_excessive_bools)]
#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Which channel shall we watch?
    ///
    /// A list of the available channels can be displayed using the
    /// `list-channels` command. The channel can also be changed from within
    /// the application.
    #[arg(value_enum, index = 1, verbatim_doc_comment)]
    pub channel: Option<String>,

    /// A preview line number offset template to use to scroll the preview to for each
    /// entry.
    ///
    /// This template uses the same syntax as the `preview` option and will be formatted
    /// using the currently selected entry.
    #[arg(long, value_name = "STRING", verbatim_doc_comment)]
    pub preview_offset: Option<String>,

    /// Disable the preview panel entirely on startup.
    #[arg(long, default_value = "false", verbatim_doc_comment)]
    pub no_preview: bool,

    /// The application's tick rate.
    ///
    /// The tick rate is the number of times the application will update per
    /// second. This can be used to control responsiveness and CPU usage on
    /// very slow machines or very fast ones but the default should be a good
    /// compromise for most users.
    #[arg(short, long, value_name = "FLOAT", verbatim_doc_comment)]
    pub tick_rate: Option<f64>,

    /// Watch mode: reload the source command every N seconds.
    ///
    /// When set to a positive number, the application will automatically
    /// reload the source command at the specified interval. This is useful
    /// for monitoring changing data sources. Set to 0 to disable (default).
    #[arg(long, value_name = "FLOAT", verbatim_doc_comment)]
    pub watch: Option<f64>,

    /// [DEPRECATED] Frame rate, i.e. number of frames to render per second.
    ///
    /// This option is deprecated and will be removed in a future release.
    #[arg(short, long, value_name = "FLOAT", verbatim_doc_comment)]
    pub frame_rate: Option<f64>,

    /// Keybindings to override the default keybindings.
    ///
    /// This can be used to override the default keybindings with a custom subset
    /// The keybindings are specified as a semicolon separated list of keybinding
    /// expressions using the configuration file formalism.
    ///
    /// Example: `tv --keybindings='quit="esc";select_next_entry=["down","ctrl-j"]'`
    #[arg(short, long, value_name = "STRING", verbatim_doc_comment)]
    pub keybindings: Option<String>,

    /// Input text to pass to the channel to prefill the prompt.
    ///
    /// This can be used to provide a default value for the prompt upon
    /// startup.
    #[arg(short, long, value_name = "STRING", verbatim_doc_comment)]
    pub input: Option<String>,

    /// Input field header template.
    ///
    /// The given value is parsed as a `MultiTemplate`. It is evaluated against
    /// the current channel name and the resulting text is shown as the input
    /// field title. Defaults to the current channel name when omitted.
    #[arg(long = "input-header", value_name = "STRING", verbatim_doc_comment)]
    pub input_header: Option<String>,

    /// Preview header template
    ///
    /// The given value is parsed as a `MultiTemplate`. It is evaluated for every
    /// entry and its result is displayed above the preview panel.
    #[arg(
        long = "preview-header",
        value_name = "STRING",
        verbatim_doc_comment
    )]
    pub preview_header: Option<String>,

    /// Preview footer template
    ///
    /// The given value is parsed as a `MultiTemplate`. It is evaluated for every
    /// entry and its result is displayed below the preview panel.
    #[arg(
        long = "preview-footer",
        value_name = "STRING",
        verbatim_doc_comment
    )]
    pub preview_footer: Option<String>,

    /// Source command to use for the current channel.
    ///
    /// This overrides the command defined in the channel prototype.
    /// Example: `find . -name '*.rs'`
    #[arg(
        long = "source-command",
        value_name = "STRING",
        verbatim_doc_comment
    )]
    pub source_command: Option<String>,

    /// Source display template to use for the current channel.
    ///
    /// This overrides the display template defined in the channel prototype.
    /// The template is used to format each entry in the results list.
    /// Example: `{split:/:-1}` (show only the last path segment)
    #[arg(
        long = "source-display",
        value_name = "STRING",
        verbatim_doc_comment
    )]
    pub source_display: Option<String>,

    /// Source output template to use for the current channel.
    ///
    /// This overrides the output template defined in the channel prototype.
    /// The template is used to format the final output when an entry is selected.
    /// Example: "{}" (output the full entry)
    #[arg(
        long = "source-output",
        value_name = "STRING",
        verbatim_doc_comment
    )]
    pub source_output: Option<String>,

    /// Preview command to use for the current channel.
    ///
    /// This overrides the preview command defined in the channel prototype.
    /// Example: "cat {}" (where {} will be replaced with the entry)
    ///
    /// Parts of the entry can be extracted positionally using the `delimiter`
    /// option.
    /// Example: "echo {0} {1}" will split the entry by the delimiter and pass
    /// the first two fields to the command.
    #[arg(
        short,
        long = "preview-command",
        value_name = "STRING",
        verbatim_doc_comment
    )]
    pub preview_command: Option<String>,

    /// Layout orientation for the UI.
    ///
    /// This overrides the layout/orientation defined in the channel prototype.
    /// Options are "landscape" or "portrait".
    #[arg(long = "layout", value_enum, verbatim_doc_comment)]
    pub layout: Option<LayoutOrientation>,

    /// The working directory to start the application in.
    ///
    /// This can be used to specify a different working directory for the
    /// application to start in. This is useful when the application is
    /// started from a different directory than the one the user wants to
    /// interact with.
    #[arg(value_name = "PATH", index = 2, verbatim_doc_comment)]
    pub working_directory: Option<String>,

    /// Try to guess the channel from the provided input prompt.
    ///
    /// This can be used to automatically select a channel based on the input
    /// prompt by using the `shell_integration` mapping in the configuration
    /// file.
    #[arg(long, value_name = "STRING", verbatim_doc_comment)]
    pub autocomplete_prompt: Option<String>,

    /// Use substring matching instead of fuzzy matching.
    ///
    /// This will use substring matching instead of fuzzy matching when
    /// searching for entries. This is useful when the user wants to search for
    /// an exact match instead of a fuzzy match e.g. to improve performance.
    #[arg(long, default_value = "false", verbatim_doc_comment)]
    pub exact: bool,

    /// Automatically select and output the first entry if there is only one
    /// entry.
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
    /// This will disable the remote control panel and associated actions
    /// entirely. This is useful when the remote control is not needed or
    /// when the user wants `tv` to run in single-channel mode (e.g. when
    /// using it as a file picker for a script or embedding it in a larger
    /// application).
    #[arg(long, default_value = "false", verbatim_doc_comment)]
    pub no_remote: bool,

    /// Disable the help panel.
    ///
    /// This will disable the help panel and associated toggling actions
    /// entirely. This is useful when the help panel is not needed or
    /// when the user wants `tv` to run with a minimal interface (e.g. when
    /// using it as a file picker for a script or embedding it in a larger
    /// application).
    #[arg(long, default_value = "false", verbatim_doc_comment)]
    pub no_help: bool,

    /// Change the display size in relation to the available area.
    ///
    /// This will crop the UI to a centered rectangle of the specified
    /// percentage of the available area (e.g. 0.5 for 50% x 50%).
    #[arg(
        long,
        value_name = "INTEGER",
        default_value = "100",
        verbatim_doc_comment
    )]
    pub ui_scale: u16,

    /// Percentage of the screen to allocate to the preview panel (1-99).
    ///
    /// This value overrides any `preview_size` defined in configuration files or channel prototypes.
    #[arg(long, value_name = "INTEGER", verbatim_doc_comment)]
    pub preview_size: Option<u16>,

    /// Provide a custom configuration file to use.
    #[arg(long, value_name = "PATH", verbatim_doc_comment)]
    pub config_file: Option<String>,

    /// Provide a custom cable directory to use.
    #[arg(long, value_name = "PATH", verbatim_doc_comment)]
    pub cable_dir: Option<String>,

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
    UpdateChannels,
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
