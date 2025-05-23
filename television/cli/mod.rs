use rustc_hash::FxHashMap;
use std::path::Path;

use anyhow::{anyhow, Result};
use tracing::debug;

use crate::{
    cable,
    channels::{
        preview::PreviewCommand,
        prototypes::{Cable, ChannelPrototype},
    },
    cli::args::{Cli, Command},
    config::{get_config_dir, get_data_dir, KeyBindings},
};

pub mod args;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct PostProcessedCli {
    pub channel: Option<String>,
    pub preview_command: Option<PreviewCommand>,
    pub no_preview: bool,
    pub tick_rate: Option<f64>,
    pub frame_rate: Option<f64>,
    pub input: Option<String>,
    pub custom_header: Option<String>,
    pub command: Option<Command>,
    pub working_directory: Option<String>,
    pub autocomplete_prompt: Option<String>,
    pub keybindings: Option<KeyBindings>,
    pub exact: bool,
    pub select_1: bool,
    pub no_remote: bool,
    pub no_help: bool,
    pub ui_scale: u16,
}

impl Default for PostProcessedCli {
    fn default() -> Self {
        Self {
            channel: None,
            preview_command: None,
            no_preview: false,
            tick_rate: None,
            frame_rate: None,
            input: None,
            custom_header: None,
            command: None,
            working_directory: None,
            autocomplete_prompt: None,
            keybindings: None,
            exact: false,
            select_1: false,
            no_remote: false,
            no_help: false,
            ui_scale: 100,
        }
    }
}

pub fn post_process(cli: Cli, cable: &Cable) -> PostProcessedCli {
    // Parse literal keybindings passed through the CLI
    let keybindings = cli.keybindings.as_ref().map(|kb| {
        parse_keybindings_literal(kb, CLI_KEYBINDINGS_DELIMITER)
            .unwrap_or_else(|e| cli_parsing_error_exit(&e.to_string()))
    });

    // Parse the preview command if provided
    let preview_command = cli.preview.as_ref().map(|preview| PreviewCommand {
        command: preview.clone(),
        delimiter: cli.delimiter.clone(),
        offset_expr: cli.preview_offset.clone(),
    });

    // Determine channel and working_directory
    let (channel, working_directory) = match &cli.channel {
        Some(c) if !cable.has_channel(c) => {
            if cli.working_directory.is_none() && Path::new(c).exists() {
                (None, Some(c.clone()))
            } else {
                unknown_channel_exit(c);
            }
        }
        _ => (cli.channel.clone(), cli.working_directory.clone()),
    };

    PostProcessedCli {
        channel,
        preview_command,
        no_preview: cli.no_preview,
        tick_rate: cli.tick_rate,
        frame_rate: cli.frame_rate,
        input: cli.input,
        custom_header: cli.custom_header,
        command: cli.command,
        working_directory,
        autocomplete_prompt: cli.autocomplete_prompt,
        keybindings,
        exact: cli.exact,
        select_1: cli.select_1,
        no_remote: cli.no_remote,
        no_help: cli.no_help,
        ui_scale: cli.ui_scale,
    }
}

fn cli_parsing_error_exit(message: &str) -> ! {
    eprintln!("Error parsing CLI arguments: {message}\n");
    std::process::exit(1);
}

pub fn unknown_channel_exit(channel: &str) -> ! {
    eprintln!("Channel not found: {channel}\n");
    std::process::exit(1);
}

const CLI_KEYBINDINGS_DELIMITER: char = ';';

/// Parse a keybindings literal into a `KeyBindings` struct.
///
/// The formalism used is the same as the one used in the configuration file:
/// ```ignore
///     quit="esc";select_next_entry=["down","ctrl-j"]
/// ```
/// Parsing it globally consists of splitting by the delimiter, reconstructing toml key-value pairs
/// and parsing that using logic already implemented in the configuration module.
fn parse_keybindings_literal(
    cli_keybindings: &str,
    delimiter: char,
) -> Result<KeyBindings> {
    let toml_definition = cli_keybindings
        .split(delimiter)
        .fold(String::new(), |acc, kb| acc + kb + "\n");

    toml::from_str(&toml_definition).map_err(|e| anyhow!(e))
}

pub fn list_channels() {
    for c in cable::load_cable().unwrap_or_default().keys() {
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
    fallback_channel: &str,
    cable: &Cable,
) -> ChannelPrototype {
    debug!("Guessing channel from prompt: {}", prompt);
    // git checkout -qf
    // --- -------- --- <---------
    let fallback = cable
        .get(fallback_channel)
        .expect("Fallback channel not found in cable channels")
        .clone();
    if prompt.trim().is_empty() {
        return fallback;
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
                return cable.get_channel(channel);
            }
            // if the word matches the top of the stack, pop it
            if stack.last() == Some(&word) {
                stack.pop();
            }
        }
        // if the stack is empty, we have a match
        if stack.is_empty() {
            return cable.get_channel(channel);
        }
        // reset the stack
        stack.clear();
    }

    debug!("No match found, falling back to default channel");
    fallback
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
    use crate::{action::Action, config::Binding, event::Key};

    use super::*;

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_from_cli() {
        let cli = Cli {
            channel: Some("files".to_string()),
            preview: Some("bat -n --color=always {}".to_string()),
            delimiter: ":".to_string(),
            working_directory: Some("/home/user".to_string()),
            ..Default::default()
        };

        let cable = cable::load_cable().unwrap_or_default();
        let post_processed_cli = post_process(cli, &cable);

        assert_eq!(
            post_processed_cli.preview_command,
            Some(PreviewCommand {
                command: "bat -n --color=always {}".to_string(),
                delimiter: ":".to_string(),
                offset_expr: None,
            })
        );
        assert_eq!(post_processed_cli.tick_rate, None);
        assert_eq!(post_processed_cli.frame_rate, None);
        assert_eq!(
            post_processed_cli.working_directory,
            Some("/home/user".to_string())
        );
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_from_cli_no_args() {
        let cli = Cli {
            channel: Some(".".to_string()),
            delimiter: ":".to_string(),
            ..Default::default()
        };

        let cable = cable::load_cable().unwrap_or_default();
        let post_processed_cli = post_process(cli, &cable);

        assert_eq!(
            post_processed_cli.working_directory,
            Some(".".to_string())
        );
        assert_eq!(post_processed_cli.command, None);
    }

    #[test]
    fn test_custom_keybindings() {
        let cli = Cli {
            channel: Some("files".to_string()),
            preview: Some(":env_var:".to_string()),
            delimiter: ":".to_string(),
            keybindings: Some(
                "quit=\"esc\";select_next_entry=[\"down\",\"ctrl-j\"]"
                    .to_string(),
            ),
            ..Default::default()
        };

        let cable = cable::load_cable().unwrap_or_default();
        let post_processed_cli = post_process(cli, &cable);

        let mut expected = KeyBindings::default();
        expected.insert(Action::Quit, Binding::SingleKey(Key::Esc));
        expected.insert(
            Action::SelectNextEntry,
            Binding::MultipleKeys(vec![Key::Down, Key::Ctrl('j')]),
        );

        assert_eq!(post_processed_cli.keybindings, Some(expected));
    }

    /// Returns a tuple containing a command mapping and a fallback channel.
    fn guess_channel_from_prompt_setup<'a>(
    ) -> (FxHashMap<String, String>, &'a str, Cable) {
        let mut command_mapping = FxHashMap::default();
        command_mapping.insert("vim".to_string(), "files".to_string());
        command_mapping.insert("export".to_string(), "env".to_string());

        (
            command_mapping,
            "env",
            cable::load_cable().unwrap_or_default(),
        )
    }

    #[test]
    fn test_guess_channel_from_prompt_present() {
        let prompt = "vim -d file1";

        let (command_mapping, fallback, channels) =
            guess_channel_from_prompt_setup();

        let channel = guess_channel_from_prompt(
            prompt,
            &command_mapping,
            fallback,
            &channels,
        );

        assert_eq!(channel.name, "files");
    }

    #[test]
    fn test_guess_channel_from_prompt_fallback() {
        let prompt = "git checkout ";

        let (command_mapping, fallback, channels) =
            guess_channel_from_prompt_setup();

        let channel = guess_channel_from_prompt(
            prompt,
            &command_mapping,
            fallback,
            &channels,
        );

        assert_eq!(channel.name, fallback);
    }

    #[test]
    fn test_guess_channel_from_prompt_empty() {
        let prompt = "";

        let (command_mapping, fallback, channels) =
            guess_channel_from_prompt_setup();

        let channel = guess_channel_from_prompt(
            prompt,
            &command_mapping,
            fallback,
            &channels,
        );

        assert_eq!(channel.name, fallback);
    }
}
