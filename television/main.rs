use std::env;
use std::io::{stdout, BufWriter, IsTerminal, Write};
use std::path::Path;
use std::process::exit;

use anyhow::Result;
use clap::Parser;
use television::utils::clipboard::CLIPBOARD;
use tracing::{debug, error, info};

use television::app::App;
use television::channels::{
    entry::PreviewType, stdin::Channel as StdinChannel, TelevisionChannel,
};
use television::cli::{
    guess_channel_from_prompt, list_channels, Cli, ParsedCliChannel,
    PostProcessedCli,
};

use television::config::{Config, ConfigEnv};
use television::utils::shell::render_autocomplete_script_template;
use television::utils::{
    shell::{completion_script, Shell},
    stdin::is_readable_stdin,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    television::errors::init()?;
    television::logging::init()?;

    let args: PostProcessedCli = Cli::parse().into();
    debug!("{:?}", args);

    let mut config = Config::new(&ConfigEnv::init()?)?;

    if let Some(command) = args.command {
        match command {
            television::cli::Command::ListChannels => {
                list_channels();
                exit(0);
            }
            television::cli::Command::InitShell { shell } => {
                let target_shell = Shell::from(shell);
                // the completion scripts for the various shells are templated
                // so that it's possible to override the keybindings triggering
                // shell autocomplete and command history in tv
                let script = render_autocomplete_script_template(
                    target_shell,
                    completion_script(target_shell)?,
                    &config.shell_integration,
                )?;
                println!("{script}");
                exit(0);
            }
        }
    }

    config.config.tick_rate =
        args.tick_rate.unwrap_or(config.config.tick_rate);
    if args.no_preview {
        config.ui.show_preview_panel = false;
    }

    if let Some(working_directory) = &args.working_directory {
        let path = Path::new(working_directory);
        if !path.exists() {
            error!(
                "Working directory \"{}\" does not exist",
                &working_directory
            );
            println!(
                "Error: Working directory \"{}\" does not exist",
                &working_directory
            );
            exit(1);
        }
        env::set_current_dir(path)?;
    }

    CLIPBOARD.with(<_>::default);

    let channel =
        determine_channel(args.clone(), &config, is_readable_stdin())?;

    let mut app =
        App::new(channel, config, &args.passthrough_keybindings, args.input);

    stdout().flush()?;
    let output = app.run(stdout().is_terminal(), false).await?;
    info!("{:?}", output);
    // lock stdout
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

pub fn determine_channel(
    args: PostProcessedCli,
    config: &Config,
    readable_stdin: bool,
) -> Result<TelevisionChannel> {
    if readable_stdin {
        debug!("Using stdin channel");
        Ok(TelevisionChannel::Stdin(StdinChannel::new(
            args.preview_command.map(PreviewType::Command),
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
            &TelevisionChannel::Stdin(StdinChannel::new(None)),
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
}
