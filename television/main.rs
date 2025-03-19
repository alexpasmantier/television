use std::env;
use std::io::{stdout, BufWriter, IsTerminal, Write};
use std::path::Path;
use std::process::exit;

use anyhow::Result;
use clap::Parser;
use television::channels::cable::PreviewKind;
use television::utils::clipboard::CLIPBOARD;
use tracing::{debug, error, info};

use television::app::App;
use television::channels::{
    entry::PreviewType, stdin::Channel as StdinChannel, TelevisionChannel,
};
use television::cli::{
    args::{Cli, Command},
    guess_channel_from_prompt, list_channels, ParsedCliChannel,
    PostProcessedCli,
};

use television::config::{merge_keybindings, Config, ConfigEnv};
use television::utils::shell::render_autocomplete_script_template;
use television::utils::{
    shell::{completion_script, Shell},
    stdin::is_readable_stdin,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    television::errors::init()?;
    television::logging::init()?;

    // post-process the CLI arguments
    let args: PostProcessedCli = Cli::parse().into();
    debug!("{:?}", args);

    // load the configuration file
    let mut config = Config::new(&ConfigEnv::init()?)?;

    // optionally handle subcommands
    args.command.as_ref().map(handle_subcommands);

    // optionally change the working directory
    args.working_directory.as_ref().map(set_current_dir);

    // optionally override configuration values with CLI arguments
    apply_cli_overrides(&args, &mut config);

    // determine the channel to use based on the CLI arguments and configuration
    let channel =
        determine_channel(args.clone(), &config, is_readable_stdin())?;

    CLIPBOARD.with(<_>::default);

    let mut app =
        App::new(channel, config, &args.passthrough_keybindings, args.input);
    stdout().flush()?;
    let output = app.run(stdout().is_terminal(), false).await?;
    info!("{:?}", output);
    let stdout_handle = stdout().lock();
    let mut bufwriter = BufWriter::new(stdout_handle);
    if let Some(passthrough) = output.passthrough {
        writeln!(bufwriter, "{passthrough}")?;
    }
    if let Some(entries) = output.selected_entries {
        for entry in &entries {
            writeln!(bufwriter, "{}", entry.stdout_repr())?;
        }
    }
    bufwriter.flush()?;
    exit(0);
}

/// Apply overrides from the CLI arguments to the configuration.
///
/// This function mutates the configuration in place.
fn apply_cli_overrides(args: &PostProcessedCli, config: &mut Config) {
    if let Some(tick_rate) = args.tick_rate {
        config.application.tick_rate = tick_rate;
    }
    if args.no_preview {
        config.ui.show_preview_panel = false;
    }
    if let Some(keybindings) = &args.keybindings {
        config.keybindings =
            merge_keybindings(config.keybindings.clone(), keybindings);
    }
}

pub fn set_current_dir(path: &String) -> Result<()> {
    let path = Path::new(path);
    if !path.exists() {
        error!("Working directory \"{}\" does not exist", path.display());
        println!(
            "Error: Working directory \"{}\" does not exist",
            path.display()
        );
        exit(1);
    }
    env::set_current_dir(path)?;
    Ok(())
}

pub fn handle_subcommands(command: &Command) -> Result<()> {
    match command {
        Command::ListChannels => {
            list_channels();
            exit(0);
        }
        Command::InitShell { shell } => {
            let target_shell = Shell::from(shell);
            // the completion scripts for the various shells are templated
            // so that it's possible to override the keybindings triggering
            // shell autocomplete and command history in tv
            let script = render_autocomplete_script_template(
                target_shell,
                completion_script(target_shell)?,
                &Config::default().shell_integration,
            )?;
            println!("{script}");
            exit(0);
        }
    }
}

pub fn determine_channel(
    args: PostProcessedCli,
    config: &Config,
    readable_stdin: bool,
) -> Result<TelevisionChannel> {
    if readable_stdin {
        debug!("Using stdin channel");
        Ok(TelevisionChannel::Stdin(StdinChannel::new(
            match &args.preview_kind {
                PreviewKind::Command(ref preview_command) => {
                    PreviewType::Command(preview_command.clone())
                }
                PreviewKind::Builtin(preview_type) => preview_type.clone(),
                PreviewKind::None => PreviewType::None,
            },
        )))
    } else if let Some(prompt) = args.autocomplete_prompt {
        let channel = guess_channel_from_prompt(
            &prompt,
            &config.shell_integration.commands,
        )?;
        debug!("Using guessed channel: {:?}", channel);
        match channel {
            ParsedCliChannel::Builtin(c) => Ok(c.to_channel()),
            ParsedCliChannel::Cable(c) => {
                Ok(TelevisionChannel::Cable(c.into()))
            }
        }
    } else {
        debug!("Using {:?} channel", args.channel);
        match args.channel {
            ParsedCliChannel::Builtin(c) => Ok(c.to_channel()),
            ParsedCliChannel::Cable(c) => {
                Ok(TelevisionChannel::Cable(c.into()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rustc_hash::FxHashMap;

    use super::*;

    fn assert_is_correct_channel(
        args: &PostProcessedCli,
        config: &Config,
        readable_stdin: bool,
        expected_channel: &TelevisionChannel,
    ) {
        let channel =
            determine_channel(args.clone(), config, readable_stdin).unwrap();

        assert!(
            channel.name() == expected_channel.name(),
            "Expected {:?} but got {:?}",
            expected_channel.name(),
            channel.name()
        );
    }

    #[tokio::test]
    async fn test_determine_channel_readable_stdin() {
        let channel = television::cli::ParsedCliChannel::Builtin(
            television::channels::CliTvChannel::Env,
        );
        let args = PostProcessedCli {
            channel,
            ..Default::default()
        };
        let config = Config::default();
        assert_is_correct_channel(
            &args,
            &config,
            true,
            &TelevisionChannel::Stdin(StdinChannel::new(PreviewType::None)),
        );
    }

    #[tokio::test]
    async fn test_determine_channel_autocomplete_prompt() {
        let autocomplete_prompt = Some("cd".to_string());
        let expected_channel = television::channels::TelevisionChannel::Dirs(
            television::channels::dirs::Channel::default(),
        );
        let args = PostProcessedCli {
            autocomplete_prompt,
            ..Default::default()
        };
        let mut config = Config {
            shell_integration:
                television::config::shell_integration::ShellIntegrationConfig {
                    commands: FxHashMap::default(),
                    channel_triggers: {
                        let mut m = FxHashMap::default();
                        m.insert("dirs".to_string(), vec!["cd".to_string()]);
                        m
                    },
                    keybindings: FxHashMap::default(),
                },
            ..Default::default()
        };
        config.shell_integration.merge_triggers();

        assert_is_correct_channel(&args, &config, false, &expected_channel);
    }

    #[tokio::test]
    async fn test_determine_channel_standard_case() {
        let channel = television::cli::ParsedCliChannel::Builtin(
            television::channels::CliTvChannel::Dirs,
        );
        let args = PostProcessedCli {
            channel,
            ..Default::default()
        };
        let config = Config::default();
        assert_is_correct_channel(
            &args,
            &config,
            false,
            &TelevisionChannel::Dirs(
                television::channels::dirs::Channel::default(),
            ),
        );
    }

    #[test]
    fn test_apply_cli_overrides() {
        let mut config = Config::default();
        let args = PostProcessedCli {
            tick_rate: Some(100_f64),
            no_preview: true,
            ..Default::default()
        };
        apply_cli_overrides(&args, &mut config);

        assert_eq!(config.application.tick_rate, 100_f64);
        assert!(!config.ui.show_preview_panel);
    }
}
