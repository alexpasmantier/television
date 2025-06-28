use rustc_hash::FxHashMap;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use tracing::debug;

use crate::{
    cable::{self, Cable},
    channels::prototypes::{ChannelPrototype, Template},
    cli::args::{Cli, Command},
    config::{
        DEFAULT_PREVIEW_SIZE, DEFAULT_UI_SCALE, KeyBindings, get_config_dir,
        get_data_dir,
    },
    screen::layout::Orientation,
    utils::paths::expand_tilde,
};

pub mod args;

/// # CLI Use Cases
///
/// The CLI interface supports two primary use cases:
///
/// ## 1. Channel-based mode (channel is specified)
/// When a channel is provided, the CLI operates in **override mode**:
/// - The channel provides the base configuration (source, preview, UI settings)
/// - All CLI flags act as **overrides** to the channel's defaults
/// - Most restrictions are enforced at the clap level using `conflicts_with`
/// - Templates and keybindings are validated after clap parsing
/// - More permissive - allows any combination of flags as they override channel defaults
///
/// ## 2. Ad-hoc mode (no channel specified)
/// When no channel is provided, the CLI creates an **ad-hoc channel**:
/// - Stricter validation rules apply for interdependent flags
/// - `--preview-*` flags require `--preview-command` to be set
/// - `--source-*` flags require `--source-command` to be set
/// - This ensures the ad-hoc channel has all necessary components to function
///
/// The validation logic in `post_process()` enforces these constraints for ad-hoc mode
/// while allowing full flexibility in channel-based mode.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct PostProcessedCli {
    // Channel and source configuration
    pub channel: Option<String>,
    pub source_command_override: Option<Template>,
    pub source_display_override: Option<Template>,
    pub source_output_override: Option<Template>,
    pub working_directory: Option<PathBuf>,
    pub autocomplete_prompt: Option<String>,

    // Preview configuration
    pub preview_command_override: Option<Template>,
    pub preview_offset_override: Option<Template>,
    pub no_preview: bool,
    pub preview_size: Option<u16>,
    pub preview_header: Option<String>,
    pub preview_footer: Option<String>,

    // Input configuration
    pub input: Option<String>,
    pub input_header: Option<String>,

    // UI and layout configuration
    pub layout: Option<Orientation>,
    pub ui_scale: u16,
    pub no_remote: bool,

    // Behavior and matching configuration
    pub exact: bool,
    pub select_1: bool,
    pub take_1: bool,
    pub take_1_fast: bool,
    pub keybindings: Option<KeyBindings>,

    // Performance configuration
    pub tick_rate: Option<f64>,
    pub watch_interval: Option<f64>,

    // Configuration sources
    pub config_file: Option<PathBuf>,
    pub cable_dir: Option<PathBuf>,

    // Command handling
    pub command: Option<Command>,
}

impl Default for PostProcessedCli {
    fn default() -> Self {
        Self {
            // Channel and source configuration
            channel: None,
            source_command_override: None,
            source_display_override: None,
            source_output_override: None,
            working_directory: None,
            autocomplete_prompt: None,

            // Preview configuration
            preview_command_override: None,
            preview_offset_override: None,
            no_preview: false,
            preview_size: Some(DEFAULT_PREVIEW_SIZE),
            preview_header: None,
            preview_footer: None,

            // Input configuration
            input: None,
            input_header: None,

            // UI and layout configuration
            layout: None,
            ui_scale: DEFAULT_UI_SCALE,
            no_remote: false,

            // Behavior and matching configuration
            exact: false,
            select_1: false,
            take_1: false,
            take_1_fast: false,
            keybindings: None,

            // Performance configuration
            tick_rate: None,
            watch_interval: None,

            // Configuration sources
            config_file: None,
            cable_dir: None,

            // Command handling
            command: None,
        }
    }
}

/// Post-processes the raw CLI arguments into a structured format with validation.
///
/// This function handles the two main CLI use cases:
///
/// **Channel-based mode**: When `cli.channel` is provided, all flags are treated as
/// overrides to the channel's configuration. Validation is minimal since the channel
/// provides sensible defaults.
///
/// **Ad-hoc mode**: When no channel is specified, stricter validation ensures that
/// interdependent flags are used correctly:
/// - Preview flags (`--preview-offset`, `--preview-size`, etc.) require `--preview-command`
/// - Source flags (`--source-display`, `--source-output`) require `--source-command`
///
/// This prevents creating broken ad-hoc channels that reference non-existent commands.
pub fn post_process(cli: Cli) -> PostProcessedCli {
    // Parse literal keybindings passed through the CLI
    let keybindings = cli.keybindings.as_ref().map(|kb| {
        parse_keybindings_literal(kb, CLI_KEYBINDINGS_DELIMITER)
            .unwrap_or_else(|e| cli_parsing_error_exit(&e.to_string()))
    });

    // Parse preview overrides if provided
    let preview_command_override =
        cli.preview_command.as_ref().map(|preview_cmd| {
            Template::parse(preview_cmd).unwrap_or_else(|e| {
                cli_parsing_error_exit(&format!(
                    "Error parsing preview command: {e}"
                ))
            })
        });

    let preview_offset_override =
        cli.preview_offset.as_ref().map(|offset_str| {
            Template::parse(offset_str).unwrap_or_else(|e| {
                cli_parsing_error_exit(&format!(
                    "Error parsing preview offset: {e}"
                ))
            })
        });

    // Validate interdependent flags for ad-hoc mode (when no channel is specified)
    // This ensures ad-hoc channels have all necessary components to function properly
    validate_adhoc_mode_constraints(&cli);

    // Determine channel and working_directory
    let (channel, working_directory) = match &cli.channel {
        Some(c) if Path::new(c).exists() => {
            // If the channel is a path, use it as the working directory
            (None, Some(PathBuf::from(c)))
        }
        _ => (
            cli.channel.clone(),
            cli.working_directory.as_ref().map(PathBuf::from),
        ),
    };

    // Parse source overrides if any source fields are provided
    let source_command_override =
        cli.source_command.as_ref().map(|source_cmd| {
            Template::parse(source_cmd).unwrap_or_else(|e| {
                cli_parsing_error_exit(&format!(
                    "Error parsing source command: {e}"
                ))
            })
        });

    let source_display_override =
        cli.source_display.as_ref().map(|display_str| {
            Template::parse(display_str).unwrap_or_else(|e| {
                cli_parsing_error_exit(&format!(
                    "Error parsing source display: {e}"
                ))
            })
        });

    let source_output_override =
        cli.source_output.as_ref().map(|output_str| {
            Template::parse(output_str).unwrap_or_else(|e| {
                cli_parsing_error_exit(&format!(
                    "Error parsing source output: {e}"
                ))
            })
        });

    // Determine layout
    let layout: Option<Orientation> =
        cli.layout.map(|layout_enum| match layout_enum {
            args::LayoutOrientation::Landscape => Orientation::Landscape,
            args::LayoutOrientation::Portrait => Orientation::Portrait,
        });

    PostProcessedCli {
        // Channel and source configuration
        channel,
        source_command_override,
        source_display_override,
        source_output_override,
        working_directory,
        autocomplete_prompt: cli.autocomplete_prompt,

        // Preview configuration
        preview_command_override,
        preview_offset_override,
        no_preview: cli.no_preview,
        preview_size: cli.preview_size,
        preview_header: cli.preview_header,
        preview_footer: cli.preview_footer,

        // Input configuration
        input: cli.input,
        input_header: cli.input_header,

        // UI and layout configuration
        layout,
        ui_scale: cli.ui_scale,
        no_remote: cli.no_remote,

        // Behavior and matching configuration
        exact: cli.exact,
        select_1: cli.select_1,
        take_1: cli.take_1,
        take_1_fast: cli.take_1_fast,
        keybindings,

        // Performance configuration
        tick_rate: cli.tick_rate,
        watch_interval: cli.watch,

        // Configuration sources
        config_file: cli.config_file.map(|p| expand_tilde(&p)),
        cable_dir: cli.cable_dir.map(|p| expand_tilde(&p)),

        // Command handling
        command: cli.command,
    }
}

/// Validates interdependent flags when operating in ad-hoc mode (no channel specified).
///
/// In ad-hoc mode, certain flags require their corresponding command to be specified:
/// - Source-related flags (`--source-display`, `--source-output`) require `--source-command`
/// - Preview-related flags (`--preview-offset`, `--preview-size`, etc.) require `--preview-command`
///
/// This validation ensures that ad-hoc channels have all necessary components to function.
/// When a channel is specified, these validations are skipped as the channel provides defaults.
fn validate_adhoc_mode_constraints(cli: &Cli) {
    // Skip validation if a channel is specified (channel-based mode)
    if cli.channel.is_some() {
        return;
    }

    // Validate source-related flags in ad-hoc mode
    if cli.source_command.is_none() {
        let source_flags = [
            ("--source-display", cli.source_display.is_some()),
            ("--source-output", cli.source_output.is_some()),
            ("--preview-command", cli.preview_command.is_some()),
        ];

        for (flag_name, is_set) in source_flags {
            if is_set {
                cli_parsing_error_exit(&format!(
                    "{} requires a source command when no channel is specified. \
                     Either specify a channel (which may have its own source command) or provide --source-command.",
                    flag_name
                ));
            }
        }
    }

    // Validate preview-related flags in ad-hoc mode
    if cli.preview_command.is_none() {
        let preview_flags = [
            ("--preview-offset", cli.preview_offset.is_some()),
            ("--preview-size", cli.preview_size.is_some()),
            ("--preview-header", cli.preview_header.is_some()),
            ("--preview-footer", cli.preview_footer.is_some()),
        ];

        for (flag_name, is_set) in preview_flags {
            if is_set {
                cli_parsing_error_exit(&format!(
                    "{} requires a preview command when no channel is specified. \
                     Either specify a channel (which may have its own preview command) or provide --preview-command.",
                    flag_name
                ));
            }
        }
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

pub fn list_channels<P>(cable_dir: P)
where
    P: AsRef<Path>,
{
    let channels =
        cable::load_cable(cable_dir).expect("Failed to load cable channels");
    for c in channels.keys() {
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
            preview_command: Some("bat -n --color=always {}".to_string()),
            working_directory: Some("/home/user".to_string()),
            ..Default::default()
        };

        let post_processed_cli = post_process(cli);

        assert_eq!(
            post_processed_cli.preview_command_override.unwrap().raw(),
            "bat -n --color=always {}".to_string(),
        );
        assert_eq!(post_processed_cli.tick_rate, None);
        assert_eq!(
            post_processed_cli.working_directory,
            Some(PathBuf::from("/home/user"))
        );
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_from_cli_no_args() {
        let cli = Cli {
            channel: Some(".".to_string()),
            ..Default::default()
        };

        let post_processed_cli = post_process(cli);

        assert_eq!(
            post_processed_cli.working_directory,
            Some(PathBuf::from("."))
        );
        assert_eq!(post_processed_cli.command, None);
    }

    #[test]
    fn test_custom_keybindings() {
        let cli = Cli {
            channel: Some("files".to_string()),
            preview_command: Some(":env_var:".to_string()),
            keybindings: Some(
                "quit=\"esc\";select_next_entry=[\"down\",\"ctrl-j\"]"
                    .to_string(),
            ),
            ..Default::default()
        };

        let post_processed_cli = post_process(cli);

        let mut expected = KeyBindings::default();
        expected.insert(Action::Quit, Binding::SingleKey(Key::Esc));
        expected.insert(
            Action::SelectNextEntry,
            Binding::MultipleKeys(vec![Key::Down, Key::Ctrl('j')]),
        );

        assert_eq!(post_processed_cli.keybindings, Some(expected));
    }

    /// Returns a tuple containing a command mapping and a fallback channel.
    fn guess_channel_from_prompt_setup<'a>()
    -> (FxHashMap<String, String>, &'a str, Cable) {
        let mut command_mapping = FxHashMap::default();
        command_mapping.insert("vim".to_string(), "files".to_string());
        command_mapping.insert("export".to_string(), "env".to_string());

        (
            command_mapping,
            "env",
            Cable::from_prototypes(vec![
                ChannelPrototype::new("files", "fd -t f"),
                ChannelPrototype::new("env", "env"),
                ChannelPrototype::new("git", "git status"),
            ]),
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

        assert_eq!(channel.metadata.name, "files");
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

        assert_eq!(channel.metadata.name, fallback);
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

        assert_eq!(channel.metadata.name, fallback);
    }
}
