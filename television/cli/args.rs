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

    /// A preview command to use with the stdin channel.
    ///
    /// If provided, the preview command will be executed and formatted using
    /// the entry.
    /// Example: "bat -n --color=always {}" (where {} will be replaced with
    /// the entry)
    ///
    /// Parts of the entry can be extracted positionally using the `delimiter`
    /// option.
    /// Example: "echo {0} {1}" will split the entry by the delimiter and pass
    /// the first two fields to the command.
    #[arg(short, long, value_name = "STRING", verbatim_doc_comment)]
    pub preview: Option<String>,

    /// Disable the preview panel entirely on startup.
    #[arg(long, default_value = "false", verbatim_doc_comment)]
    pub no_preview: bool,

    /// The delimiter used to extract fields from the entry to provide to the
    /// preview command.
    ///
    /// See the `preview` option for more information.
    #[arg(long, value_name = "STRING", default_value = " ", value_parser = delimiter_parser, verbatim_doc_comment)]
    pub delimiter: String,

    /// The application's tick rate.
    ///
    /// The tick rate is the number of times the application will update per
    /// second. This can be used to control responsiveness and CPU usage on
    /// very slow machines or very fast ones but the default should be a good
    /// compromise for most users.
    #[arg(short, long, value_name = "FLOAT", verbatim_doc_comment)]
    pub tick_rate: Option<f64>,

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

    /// Input fields header title
    ///
    /// This can be used to give the input field a custom title e.g. the current
    /// working directory.
    /// The default value for the input header is the current channel.
    #[arg(long, value_name = "STRING", verbatim_doc_comment)]
    pub custom_header: Option<String>,

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
    #[arg(long, default_value = "false", verbatim_doc_comment)]
    pub select_1: bool,

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
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Cmd,
}

#[allow(clippy::unnecessary_wraps)]
fn delimiter_parser(s: &str) -> Result<String, String> {
    Ok(match s {
        "" => " ".to_string(),
        "\\t" => "\t".to_string(),
        _ => s.to_string(),
    })
}
