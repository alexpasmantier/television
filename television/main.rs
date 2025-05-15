use std::env;
use std::io::{stdout, BufWriter, IsTerminal, Write};
use std::path::Path;
use std::process::exit;

use anyhow::Result;
use clap::Parser;
use television::{
    cable,
    channels::prototypes::{Cable, ChannelPrototype},
    utils::clipboard::CLIPBOARD,
};
use tracing::{debug, error, info};

use television::app::{App, AppOptions};
use television::cli::{
    args::{Cli, Command},
    guess_channel_from_prompt, list_channels, PostProcessedCli,
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

    debug!("\n\n====  NEW SESSION  =====\n");

    // process the CLI arguments
    let cli = Cli::parse();
    debug!("CLI: {:?}", cli);
    let args: PostProcessedCli = cli.into();
    debug!("PostProcessedCli: {:?}", args);

    // load the configuration file
    debug!("Loading configuration...");
    let mut config = Config::new(&ConfigEnv::init()?)?;

    debug!("Loading cable channels...");
    let cable_channels = cable::load_cable().unwrap_or_default();

    // optionally handle subcommands
    debug!("Handling subcommands...");
    args.command
        .as_ref()
        .map(|x| handle_subcommands(x, &config));

    // optionally change the working directory
    args.working_directory.as_ref().map(set_current_dir);

    // optionally override configuration values with CLI arguments
    debug!("Applying CLI overrides...");
    apply_cli_overrides(&args, &mut config);

    // determine the channel to use based on the CLI arguments and configuration
    debug!("Determining channel...");
    let channel_prototype = determine_channel(
        args.clone(),
        &config,
        is_readable_stdin(),
        &cable_channels,
    )?;

    CLIPBOARD.with(<_>::default);

    debug!("Creating application...");
    let options = AppOptions::new(
        args.exact,
        args.select_1,
        args.no_remote,
        args.no_help,
        config.application.tick_rate,
    );
    let mut app = App::new(
        &channel_prototype,
        config,
        args.input,
        options,
        &cable_channels,
    );
    stdout().flush()?;
    debug!("Running application...");
    let output = app.run(stdout().is_terminal(), false).await?;
    info!("App output: {:?}", output);
    let stdout_handle = stdout().lock();
    let mut bufwriter = BufWriter::new(stdout_handle);
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
    if let Some(header) = &args.custom_header {
        config.ui.custom_header = Some(header.to_string());
    }
    config.ui.ui_scale = args.ui_scale;
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

pub fn handle_subcommands(command: &Command, config: &Config) -> Result<()> {
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
                &config.shell_integration,
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
    cable_channels: &Cable,
) -> Result<ChannelPrototype> {
    if readable_stdin {
        debug!("Using stdin channel");
        Ok(ChannelPrototype::stdin(args.preview_command))
    } else if let Some(prompt) = args.autocomplete_prompt {
        debug!("Using autocomplete prompt: {:?}", prompt);
        let channel_prototype = guess_channel_from_prompt(
            &prompt,
            &config.shell_integration.commands,
            &config.shell_integration.fallback_channel,
            cable_channels,
        )?;
        debug!("Using guessed channel: {:?}", channel_prototype);
        Ok(channel_prototype)
    } else {
        debug!("Using {:?} channel", args.channel);
        Ok(args.channel)
    }
}

#[cfg(test)]
mod tests {
    use rustc_hash::FxHashMap;
    use television::{
        cable::load_cable, channels::prototypes::ChannelPrototype,
    };

    use super::*;

    fn assert_is_correct_channel(
        args: &PostProcessedCli,
        config: &Config,
        readable_stdin: bool,
        expected_channel: &ChannelPrototype,
        cable_channels: Option<Cable>,
    ) {
        let channels: Cable =
            cable_channels.unwrap_or_else(|| load_cable().unwrap_or_default());
        let channel =
            determine_channel(args.clone(), config, readable_stdin, &channels)
                .unwrap();

        assert_eq!(
            channel.name, expected_channel.name,
            "Expected {:?} but got {:?}",
            expected_channel.name, channel.name
        );
    }

    #[tokio::test]
    /// Test that the channel is stdin when stdin is readable
    async fn test_determine_channel_readable_stdin() {
        let channel = ChannelPrototype::default();
        let args = PostProcessedCli {
            channel,
            ..Default::default()
        };
        let config = Config::default();
        assert_is_correct_channel(
            &args,
            &config,
            true,
            &ChannelPrototype::new("stdin", "cat", false, None),
            None,
        );
    }

    #[tokio::test]
    async fn test_determine_channel_autocomplete_prompt() {
        let autocomplete_prompt = Some("cd".to_string());
        let expected_channel =
            ChannelPrototype::new("dirs", "ls {}", false, None);
        let args = PostProcessedCli {
            autocomplete_prompt,
            ..Default::default()
        };
        let mut config = Config {
            shell_integration:
                television::config::shell_integration::ShellIntegrationConfig {
                    fallback_channel: "files".to_string(),
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

        assert_is_correct_channel(
            &args,
            &config,
            false,
            &expected_channel,
            None,
        );
    }

    #[tokio::test]
    async fn test_determine_channel_standard_case() {
        let channel = ChannelPrototype::new("dirs", "", false, None);
        let args = PostProcessedCli {
            channel,
            ..Default::default()
        };
        let config = Config::default();
        assert_is_correct_channel(
            &args,
            &config,
            false,
            &ChannelPrototype::new("dirs", "", false, None),
            None,
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
