use rustc_hash::FxHashMap;
use std::path::Path;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand, ValueEnum};
use tracing::debug;

use crate::channels::{
    cable::CableChannelPrototype, entry::PreviewCommand, CliTvChannel,
};
use crate::utils::shell::Shell as UtilShell;
use crate::{
    cable,
    config::{get_config_dir, get_data_dir},
};

#[derive(Parser, Debug)]
#[command(author, version = version(), about)]
pub struct Cli {
    /// Which channel shall we watch?
    #[arg(value_enum, default_value = "files", index = 1)]
    pub channel: String,

    /// Use a custom preview command (currently only supported by the stdin channel)
    #[arg(short, long, value_name = "STRING")]
    pub preview: Option<String>,

    /// Disable the preview pane
    #[arg(long, default_value = "false")]
    pub no_preview: bool,

    /// The delimiter used to extract fields from the entry to provide to the preview command
    /// (defaults to " ")
    #[arg(long, value_name = "STRING", default_value = " ", value_parser = delimiter_parser)]
    pub delimiter: String,

    /// Tick rate, i.e. number of ticks per second
    #[arg(short, long, value_name = "FLOAT")]
    pub tick_rate: Option<f64>,

    /// [DEPRECATED] Frame rate, i.e. number of frames per second
    #[arg(short, long, value_name = "FLOAT")]
    pub frame_rate: Option<f64>,

    /// Passthrough keybindings (comma separated, e.g. "q,ctrl-w,ctrl-t") These keybindings will
    /// trigger selection of the current entry and be passed through to stdout along with the entry
    /// to be handled by the parent process.
    #[arg(long, value_name = "STRING")]
    pub passthrough_keybindings: Option<String>,

    /// Input text to pass to the channel to prefill the prompt
    #[arg(short, long, value_name = "STRING")]
    pub input: Option<String>,

    /// The working directory to start in
    #[arg(value_name = "PATH", index = 2)]
    pub working_directory: Option<String>,

    /// Try to guess the channel from the provided input prompt
    #[arg(long, value_name = "STRING")]
    pub autocomplete_prompt: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug, PartialEq, Clone)]
pub enum Command {
    /// Lists available channels
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

impl From<Shell> for UtilShell {
    fn from(val: Shell) -> Self {
        match val {
            Shell::Bash => UtilShell::Bash,
            Shell::Zsh => UtilShell::Zsh,
            Shell::Fish => UtilShell::Fish,
            Shell::PowerShell => UtilShell::PowerShell,
            Shell::Cmd => UtilShell::Cmd,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PostProcessedCli {
    pub channel: ParsedCliChannel,
    pub preview_command: Option<PreviewCommand>,
    pub no_preview: bool,
    pub tick_rate: Option<f64>,
    pub frame_rate: Option<f64>,
    pub passthrough_keybindings: Vec<String>,
    pub input: Option<String>,
    pub command: Option<Command>,
    pub working_directory: Option<String>,
    pub autocomplete_prompt: Option<String>,
}

impl Default for PostProcessedCli {
    fn default() -> Self {
        Self {
            channel: ParsedCliChannel::Builtin(CliTvChannel::Files),
            preview_command: None,
            no_preview: false,
            tick_rate: None,
            frame_rate: None,
            passthrough_keybindings: Vec::new(),
            input: None,
            command: None,
            working_directory: None,
            autocomplete_prompt: None,
        }
    }
}

impl From<Cli> for PostProcessedCli {
    fn from(cli: Cli) -> Self {
        let passthrough_keybindings = cli
            .passthrough_keybindings
            .unwrap_or_default()
            .split(',')
            .map(std::string::ToString::to_string)
            .collect();

        let preview_command = cli.preview.map(|preview| PreviewCommand {
            command: preview,
            delimiter: cli.delimiter.clone(),
        });

        let channel: ParsedCliChannel;
        let working_directory: Option<String>;

        match parse_channel(&cli.channel) {
            Ok(p) => {
                channel = p;
                working_directory = cli.working_directory;
            }
            Err(_) => {
                // if the path is provided as first argument and it exists, use it as the working
                // directory and default to the files channel
                if cli.working_directory.is_none()
                    && Path::new(&cli.channel).exists()
                {
                    channel = ParsedCliChannel::Builtin(CliTvChannel::Files);
                    working_directory = Some(cli.channel.clone());
                } else {
                    unknown_channel_exit(&cli.channel);
                    unreachable!();
                }
            }
        }

        Self {
            channel,
            preview_command,
            no_preview: cli.no_preview,
            tick_rate: cli.tick_rate,
            frame_rate: cli.frame_rate,
            passthrough_keybindings,
            input: cli.input,
            command: cli.command,
            working_directory,
            autocomplete_prompt: cli.autocomplete_prompt,
        }
    }
}

fn unknown_channel_exit(channel: &str) {
    eprintln!("Unknown channel: {channel}\n");
    std::process::exit(1);
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedCliChannel {
    Builtin(CliTvChannel),
    Cable(CableChannelPrototype),
}

fn parse_channel(channel: &str) -> Result<ParsedCliChannel> {
    let cable_channels = cable::load_cable_channels().unwrap_or_default();
    // try to parse the channel as a cable channel
    cable_channels
        .iter()
        .find(|(k, _)| k.to_lowercase() == channel)
        .map_or_else(
            || {
                // try to parse the channel as a builtin channel
                CliTvChannel::try_from(channel)
                    .map(ParsedCliChannel::Builtin)
                    .map_err(|_| anyhow!("Unknown channel: {}", channel))
            },
            |(_, v)| Ok(ParsedCliChannel::Cable(v.clone())),
        )
}

pub fn list_cable_channels() -> Vec<String> {
    cable::load_cable_channels()
        .unwrap_or_default()
        .iter()
        .map(|(k, _)| k.clone())
        .collect()
}

pub fn list_builtin_channels() -> Vec<String> {
    CliTvChannel::all_channels()
        .iter()
        .map(std::string::ToString::to_string)
        .collect()
}

pub fn list_channels() {
    println!("\x1b[4mBuiltin channels:\x1b[0m");
    for c in list_builtin_channels() {
        println!("\t{c}");
    }
    println!("\n\x1b[4mCustom channels:\x1b[0m");
    for c in list_cable_channels().iter().map(|c| c.to_lowercase()) {
        println!("\t{c}");
    }
}

/// Backtrack from the end of the prompt and try to match each word to a known command
/// if a match is found, return the corresponding channel
/// if no match is found, throw an error
///
/// ## Example:
/// ```ignore
/// use television::channels::CliTvChannel;
/// use television::cli::ParsedCliChannel;
/// use television::cli::guess_channel_from_prompt;
///
/// let prompt = "ls -l";
/// let command_mapping = hashmap! {
///     "ls".to_string() => "files".to_string(),
///     "cat".to_string() => "files".to_string(),
/// };
/// let channel = guess_channel_from_prompt(prompt, &command_mapping).unwrap();
///
/// assert_eq!(channel, ParsedCliChannel::Builtin(CliTvChannel::Files));
/// ```
/// NOTE: this is a very naive implementation and needs to be improved
/// - it should be able to handle prompts containing multiple commands
///   e.g. "ls -l && cat <CTRL+T>"
/// - it should be able to handle commands within delimiters (quotes, brackets, etc.)
pub fn guess_channel_from_prompt(
    prompt: &str,
    command_mapping: &FxHashMap<String, String>,
) -> Result<ParsedCliChannel> {
    debug!("Guessing channel from prompt: {}", prompt);
    // git checkout -qf
    // --- -------- --- <---------
    if prompt.trim().is_empty() {
        return match command_mapping.get("") {
            Some(channel) => parse_channel(channel),
            None => Err(anyhow!("No channel found for prompt: {}", prompt)),
        };
    }
    let rev_prompt_words = prompt.split_whitespace().rev();
    let mut stack = Vec::new();
    // for each patern
    for (pattern, channel) in command_mapping {
        if pattern.trim().is_empty() {
            continue;
        }
        // push every word of the pattern onto the stack
        stack.extend(pattern.split_whitespace());
        for word in rev_prompt_words.clone() {
            // if the stack is empty, we have a match
            if stack.is_empty() {
                return parse_channel(channel);
            }
            // if the word matches the top of the stack, pop it
            if stack.last() == Some(&word) {
                stack.pop();
            }
        }
        // if the stack is empty, we have a match
        if stack.is_empty() {
            return parse_channel(channel);
        }
        // reset the stack
        stack.clear();
    }
    Err(anyhow!("No channel found for prompt: {}", prompt))
}

#[allow(clippy::unnecessary_wraps)]
fn delimiter_parser(s: &str) -> Result<String, String> {
    Ok(match s {
        "" => " ".to_string(),
        "\\t" => "\t".to_string(),
        _ => s.to_string(),
    })
}

const VERSION_MESSAGE: &str = env!("CARGO_PKG_VERSION");

pub fn version() -> String {
    let author = clap::crate_authors!();

    // let current_exe_path = PathBuf::from(clap::crate_name!()).display().to_string();
    let config_dir_path = get_config_dir().display().to_string();
    let data_dir_path = get_data_dir().display().to_string();

    format!(
        "\
{VERSION_MESSAGE}

           _______________
          |,----------.  |\\
          ||           |=| |
          ||          || | |
          ||       . _o| | |
          |`-----------' |/
           ~~~~~~~~~~~~~~~
  __      __         _     _
 / /____ / /__ _  __(_)__ (_)__  ___ 
/ __/ -_) / -_) |/ / (_-</ / _ \\/ _ \\
\\__/\\__/_/\\__/|___/_/___/_/\\___/_//_/

Authors: {author}

Config directory: {config_dir_path}
Data directory: {data_dir_path}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_from_cli() {
        let cli = Cli {
            channel: "files".to_string(),
            preview: Some("bat -n --color=always {}".to_string()),
            no_preview: false,
            delimiter: ":".to_string(),
            tick_rate: Some(50.0),
            frame_rate: Some(60.0),
            passthrough_keybindings: Some("q,ctrl-w,ctrl-t".to_string()),
            input: None,
            command: None,
            working_directory: Some("/home/user".to_string()),
            autocomplete_prompt: None,
        };

        let post_processed_cli: PostProcessedCli = cli.into();

        assert_eq!(
            post_processed_cli.channel,
            ParsedCliChannel::Builtin(CliTvChannel::Files)
        );
        assert_eq!(
            post_processed_cli.preview_command,
            Some(PreviewCommand {
                command: "bat -n --color=always {}".to_string(),
                delimiter: ":".to_string()
            })
        );
        assert_eq!(post_processed_cli.tick_rate, Some(50.0));
        assert_eq!(post_processed_cli.frame_rate, Some(60.0));
        assert_eq!(
            post_processed_cli.passthrough_keybindings,
            vec!["q".to_string(), "ctrl-w".to_string(), "ctrl-t".to_string()]
        );
        assert_eq!(
            post_processed_cli.working_directory,
            Some("/home/user".to_string())
        );
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_from_cli_no_args() {
        let cli = Cli {
            channel: ".".to_string(),
            preview: None,
            no_preview: false,
            delimiter: ":".to_string(),
            tick_rate: Some(50.0),
            frame_rate: Some(60.0),
            passthrough_keybindings: None,
            input: None,
            command: None,
            working_directory: None,
            autocomplete_prompt: None,
        };

        let post_processed_cli: PostProcessedCli = cli.into();

        assert_eq!(
            post_processed_cli.channel,
            ParsedCliChannel::Builtin(CliTvChannel::Files)
        );
        assert_eq!(
            post_processed_cli.working_directory,
            Some(".".to_string())
        );
        assert_eq!(post_processed_cli.command, None);
    }
}
