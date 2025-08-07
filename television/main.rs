use anyhow::Result;
use clap::Parser;
use std::env;
use std::io::{BufWriter, IsTerminal, Write, stdout};
use std::path::PathBuf;
use std::process::exit;
use television::cli::ChannelCli;
use television::config::layers::LayeredConfig;
use television::config::shell_integration::ShellIntegrationConfig;
use television::{
    app::App,
    cable::{Cable, cable_empty_exit, load_cable},
    channels::prototypes::ChannelPrototype,
    cli::{
        args::{Cli, Command},
        guess_channel_from_prompt, list_channels, post_process,
    },
    config::{Config, ConfigEnv},
    errors::os_error_exit,
    gh::update_local_channels,
    television::Mode,
    utils::clipboard::CLIPBOARD,
    utils::{
        shell::{
            Shell, completion_script, render_autocomplete_script_template,
        },
        stdin::is_readable_stdin,
    },
};
use tracing::{debug, info};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    television::errors::init()?;
    television::logging::init()?;

    debug!("\n\n====  NEW SESSION  =====\n");

    let readable_stdin = is_readable_stdin();

    let cli = post_process(Cli::parse(), readable_stdin);
    debug!("PostProcessedCli: {:?}", cli);

    // load the configuration file
    debug!("Loading configuration...");
    let base_config =
        Config::new(&ConfigEnv::init()?, cli.global.config_file.as_deref())?;

    let cable_dir = cli
        .global
        .cable_dir
        .clone()
        .unwrap_or_else(|| base_config.application.cable_dir.clone());

    debug!("Loading cable channels...");
    let cable = load_cable(&cable_dir).unwrap_or_default();

    // handle subcommands
    debug!("Handling subcommands...");
    if let Some(subcommand) = &cli.global.command {
        handle_subcommand(subcommand, &cable, &base_config.shell_integration)?;
    }

    // optionally change the working directory
    if let Some(ref working_dir) = cli.global.workdir {
        set_current_dir(working_dir)
            .unwrap_or_else(|e| os_error_exit(&e.to_string()));
    }

    // determine the base channel prototype
    debug!("Determining base channel prototype...");
    let channel_prototype = determine_channel(
        &cli.channel,
        &base_config,
        readable_stdin,
        Some(&cable),
    );

    let layered_config =
        LayeredConfig::new(base_config, channel_prototype, cli.clone());

    CLIPBOARD.with(<_>::default);

    debug!("Creating application...");
    let mut app = App::new(layered_config, cable);

    // If the user requested to show the remote control on startup, switch the
    // television into Remote Control mode before the application event loop
    // begins. This mirrors the behaviour of toggling the remote control via
    // the corresponding keybinding after launch, ensuring the panel is
    // visible from the start.
    // TODO: This is a hack, preview is not initialised yet, find a better way to do it.
    if cli.channel.show_remote && app.television.remote_control.is_some() {
        app.television.mode = Mode::RemoteControl;
    }

    stdout().flush()?;
    debug!("Running application...");
    let output = app.run(stdout().is_terminal(), false).await?;
    info!("App output: {:?}", output);

    let stdout_handle = stdout().lock();
    let mut bufwriter = BufWriter::new(stdout_handle);
    if let Some(key) = output.expect_key {
        writeln!(bufwriter, "{}", key)?;
    }
    if let Some(entries) = output.selected_entries {
        for entry in &entries {
            writeln!(bufwriter, "{}", entry.output()?)?;
        }
    }
    bufwriter.flush()?;
    exit(0);
}

pub fn set_current_dir(path: &PathBuf) -> Result<()> {
    env::set_current_dir(path)?;
    Ok(())
}

pub fn handle_subcommand(
    command: &Command,
    cable: &Cable,
    shell_integration_config: &ShellIntegrationConfig,
) -> Result<()> {
    match command {
        Command::ListChannels => {
            list_channels(cable);
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
                shell_integration_config,
            )?;
            println!("{script}");
            exit(0);
        }
        Command::UpdateChannels { force } => {
            update_local_channels(force)?;
            exit(0);
        }
    }
}

/// Creates an ad-hoc channel prototype from CLI arguments
fn create_adhoc_channel(cli: &ChannelCli) -> ChannelPrototype {
    let p = ChannelPrototype::new(
        "Custom Channel",
        cli.source_command.as_ref().unwrap().raw(),
    );

    debug!("Creating ad-hoc channel prototype: {:?}", p);

    p
}

/// Determines which channel prototype to use based on CLI arguments and configuration.
///
/// This function handles multiple modes of operation:
/// 1. **Stdin mode**: When stdin is readable, creates a stdin channel
/// 2. **Autocomplete mode**: When autocomplete prompt is provided, guesses channel from prompt
/// 3. **Ad-hoc mode**: When no channel is specified but source command is provided
/// 4. **Channel mode**: Uses a named channel from CLI args or config default
///
/// After determining the base channel, it applies all relevant CLI overrides.
pub fn determine_channel(
    cli: &ChannelCli,
    config: &Config,
    readable_stdin: bool,
    cable: Option<&Cable>,
) -> ChannelPrototype {
    // Determine the base channel prototype
    if readable_stdin {
        ChannelPrototype::stdin()
    } else if let Some(prompt) = &cli.autocomplete_prompt {
        if cable.is_none() {
            cable_empty_exit()
        }
        debug!("Using autocomplete prompt: {:?}", prompt);
        let prototype = guess_channel_from_prompt(
            prompt,
            &config.shell_integration.commands,
            &config.shell_integration.fallback_channel,
            cable.unwrap(),
        );
        debug!("Using guessed channel: {:?}", prototype);
        prototype
    } else if cli.channel.is_none() && cli.source_command.is_some() {
        create_adhoc_channel(cli)
    } else {
        if cable.is_none() {
            cable_empty_exit()
        }
        let channel_name = cli
            .channel
            .as_ref()
            .unwrap_or(&config.application.default_channel);
        debug!("Using channel: {:?}", channel_name);
        cable.unwrap().get_channel(channel_name)
    }
}

#[cfg(test)]
mod tests {
    use rustc_hash::FxHashMap;
    use television::{
        channels::prototypes::{
            ChannelPrototype, CommandSpec, PreviewSpec, Template,
        },
        cli::PostProcessedCli,
    };

    use super::*;

    fn assert_is_correct_channel(
        cli: &PostProcessedCli,
        config: &Config,
        readable_stdin: bool,
        expected_channel: &ChannelPrototype,
        cable_channels: Option<Cable>,
    ) {
        let channels: Cable =
            cable_channels.unwrap_or(Cable::from_prototypes(vec![
                ChannelPrototype::new("files", "fd -t f"),
                ChannelPrototype::new("dirs", "ls"),
                ChannelPrototype::new("git", "git status"),
            ]));
        let channel = determine_channel(
            &cli.channel,
            config,
            readable_stdin,
            Some(&channels),
        );

        assert_eq!(
            channel.metadata.name, expected_channel.metadata.name,
            "Expected {:?} but got {:?}",
            expected_channel.metadata.name, channel.metadata.name
        );
    }

    #[test]
    /// Test that the channel is stdin when stdin is readable
    fn test_determine_channel_readable_stdin() {
        let args = PostProcessedCli::default();
        let config = Config::default();
        assert_is_correct_channel(
            &args,
            &config,
            true,
            &ChannelPrototype::new("stdin", "cat"),
            None,
        );
    }

    #[test]
    fn test_determine_channel_autocomplete_prompt() {
        let autocomplete_prompt = Some("cd".to_string());
        let expected_channel = ChannelPrototype::new("dirs", "ls {}");
        let args = PostProcessedCli {
            channel: ChannelCli {
                autocomplete_prompt,
                ..Default::default()
            },
            ..Default::default()
        };
        let config = Config {
            shell_integration:
                television::config::shell_integration::ShellIntegrationConfig {
                    fallback_channel: "files".to_string(),
                    commands: {
                        let mut m = FxHashMap::default();
                        m.insert("cd".to_string(), "dirs".to_string());
                        m
                    },
                    keybindings: FxHashMap::default(),
                },
            ..Default::default()
        };

        assert_is_correct_channel(
            &args,
            &config,
            false,
            &expected_channel,
            None,
        );
    }

    #[test]
    fn test_determine_channel_standard_case() {
        let channel = Some(String::from("dirs"));
        let args = PostProcessedCli {
            channel: ChannelCli {
                channel,
                ..Default::default()
            },
            ..Default::default()
        };
        let config = Config::default();
        assert_is_correct_channel(
            &args,
            &config,
            false,
            &ChannelPrototype::new("dirs", "ls {}"),
            None,
        );
    }

    #[test]
    fn test_determine_channel_config_fallback() {
        let args = PostProcessedCli {
            channel: ChannelCli {
                channel: None,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut config = Config::default();
        config.application.default_channel = String::from("dirs");
        assert_is_correct_channel(
            &args,
            &config,
            false,
            &ChannelPrototype::new("dirs", "ls"),
            None,
        );
    }

    #[test]
    fn test_determine_channel_with_cli_preview() {
        let preview_command = Template::parse("echo hello").unwrap();
        let preview_spec = PreviewSpec::new(
            CommandSpec::new(
                vec![preview_command.clone()],
                false,
                FxHashMap::default(),
            ),
            None,
        );

        let args = PostProcessedCli {
            channel: ChannelCli {
                channel: Some(String::from("dirs")),
                preview_command: Some(preview_command),
                ..Default::default()
            },
            ..Default::default()
        };
        let config = Config::default();

        let expected_prototype = ChannelPrototype::new("dirs", "ls")
            .with_preview(Some(preview_spec));

        assert_is_correct_channel(
            &args,
            &config,
            false,
            &expected_prototype,
            None,
        );
    }

    #[test]
    fn test_determine_channel_adhoc_with_source_command() {
        let cli = PostProcessedCli {
            channel: ChannelCli {
                channel: None,
                source_command: Some(Template::parse("fd -t f -H").unwrap()),
                ..Default::default()
            },
            ..Default::default()
        };
        let config = Config::default();

        let channel = determine_channel(
            &cli.channel,
            &config,
            false,
            Some(&Cable::default()),
        );

        assert_eq!(channel.metadata.name, "Custom Channel");
        assert_eq!(channel.source.command.inner[0].raw(), "fd -t f -H");
    }
}
