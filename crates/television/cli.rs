use std::path::Path;

use clap::{Parser, Subcommand};
use color_eyre::{eyre::eyre, Result};

use crate::{
    cable,
    config::{get_config_dir, get_data_dir},
};
use television_channels::{
    cable::CableChannelPrototype, channels::CliTvChannel,
    entry::PreviewCommand,
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

    /// The delimiter used to extract fields from the entry to provide to the preview command
    /// (defaults to ":")
    #[arg(long, value_name = "STRING", default_value = " ", value_parser = delimiter_parser)]
    pub delimiter: String,

    /// Tick rate, i.e. number of ticks per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 50.0)]
    pub tick_rate: f64,

    /// Frame rate, i.e. number of frames per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 60.0)]
    pub frame_rate: f64,

    /// Passthrough keybindings (comma separated, e.g. "q,ctrl-w,ctrl-t") These keybindings will
    /// trigger selection of the current entry and be passed through to stdout along with the entry
    /// to be handled by the parent process.
    #[arg(long, value_name = "STRING")]
    pub passthrough_keybindings: Option<String>,

    /// The working directory to start in
    #[arg(value_name = "PATH", index = 2)]
    pub working_directory: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum Command {
    /// Lists available channels
    ListChannels,
}

#[derive(Debug)]
pub struct PostProcessedCli {
    pub channel: ParsedCliChannel,
    pub preview_command: Option<PreviewCommand>,
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub passthrough_keybindings: Vec<String>,
    pub command: Option<Command>,
    pub working_directory: Option<String>,
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

        match channel_parser(&cli.channel) {
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
            tick_rate: cli.tick_rate,
            frame_rate: cli.frame_rate,
            passthrough_keybindings,
            command: cli.command,
            working_directory,
        }
    }
}

fn unknown_channel_exit(channel: &str) {
    eprintln!("Unknown channel: {channel}\n");
    // print the list of channels
    list_channels();
    std::process::exit(1);
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedCliChannel {
    Builtin(CliTvChannel),
    Cable(CableChannelPrototype),
}

fn channel_parser(channel: &str) -> Result<ParsedCliChannel> {
    let cable_channels = cable::load_cable_channels().unwrap_or_default();
    CliTvChannel::try_from(channel)
        .map(ParsedCliChannel::Builtin)
        .or_else(|_| {
            cable_channels
                .iter()
                .find(|(k, _)| k.to_lowercase() == channel)
                .map_or_else(
                    || Err(eyre!("Unknown channel: {}", channel)),
                    |(_, v)| Ok(ParsedCliChannel::Cable(v.clone())),
                )
        })
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

fn delimiter_parser(s: &str) -> Result<String, String> {
    Ok(match s {
        "" => ":".to_string(),
        "\\t" => "\t".to_string(),
        _ => s.to_string(),
    })
}

const VERSION_MESSAGE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\ntarget triple: ",
    env!("VERGEN_CARGO_TARGET_TRIPLE"),
    "\nbuild: ",
    env!("VERGEN_RUSTC_SEMVER"),
    " (",
    env!("VERGEN_BUILD_DATE"),
    ")"
);

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
            delimiter: ":".to_string(),
            tick_rate: 50.0,
            frame_rate: 60.0,
            passthrough_keybindings: Some("q,ctrl-w,ctrl-t".to_string()),
            command: None,
            working_directory: Some("/home/user".to_string()),
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
        assert_eq!(post_processed_cli.tick_rate, 50.0);
        assert_eq!(post_processed_cli.frame_rate, 60.0);
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
            delimiter: ":".to_string(),
            tick_rate: 50.0,
            frame_rate: 60.0,
            passthrough_keybindings: None,
            command: None,
            working_directory: None,
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
