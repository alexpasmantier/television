use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Which channel shall we watch?
    ///
    /// A list of the available channels can be displayed using the
    /// `list-channels` command. The channel can also be changed from within
    /// the application.
    #[arg(
        value_enum,
        default_value = "files",
        index = 1,
        verbatim_doc_comment
    )]
    pub channel: String,

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

    /// Passthrough keybindings (comma separated, e.g. "q,ctrl-w,ctrl-t")
    ///
    /// These keybindings will trigger selection of the current entry and be
    /// passed through to stdout along with the entry to be handled by the
    /// parent process.
    #[arg(long, value_name = "STRING", verbatim_doc_comment)]
    pub passthrough_keybindings: Option<String>,

    /// Input text to pass to the channel to prefill the prompt.
    ///
    /// This can be used to provide a default value for the prompt upon
    /// startup.
    #[arg(short, long, value_name = "STRING", verbatim_doc_comment)]
    pub input: Option<String>,

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
