use std::env;
use std::io::{BufWriter, IsTerminal, Write, stdout};
use std::path::Path;
use std::process::exit;

use anyhow::Result;
use clap::Parser;
use television::cable::load_cable;
use television::cli::post_process;
use television::gh::update_local_channels;
use television::{
    cable::Cable, channels::prototypes::ChannelPrototype,
    utils::clipboard::CLIPBOARD,
};
use tracing::{debug, error, info};

use television::app::{App, AppOptions};
use television::cli::{
    PostProcessedCli,
    args::{Cli, Command},
    guess_channel_from_prompt, list_channels,
};

use television::config::{Config, ConfigEnv, merge_keybindings};
use television::utils::shell::render_autocomplete_script_template;
use television::utils::{
    shell::{Shell, completion_script},
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

    let args = post_process(cli);
    debug!("PostProcessedCli: {:?}", args);

    // load the configuration file
    debug!("Loading configuration...");
    let mut config =
        Config::new(&ConfigEnv::init()?, args.config_file.as_deref())?;

    // override configuration values with provided CLI arguments
    debug!("Applying CLI overrides...");
    apply_cli_overrides(&args, &mut config);

    // handle subcommands
    debug!("Handling subcommands...");
    if let Some(subcommand) = &args.command {
        handle_subcommand(subcommand, &config)?;
    }

    debug!("Loading cable channels...");
    let cable =
        load_cable(&config.application.cable_dir).unwrap_or_else(|| exit(1));

    // optionally change the working directory
    args.working_directory.as_ref().map(set_current_dir);

    // determine the channel to use based on the CLI arguments and configuration
    debug!("Determining channel...");
    let channel_prototype =
        determine_channel(&args, &config, is_readable_stdin(), &cable);

    CLIPBOARD.with(<_>::default);

    debug!("Creating application...");
    let options = AppOptions::new(
        args.exact,
        args.select_1,
        args.no_remote,
        args.no_help,
        args.no_preview,
        args.preview_size,
        config.application.tick_rate,
    );
    let mut app =
        App::new(channel_prototype, config, args.input, options, cable);
    stdout().flush()?;
    debug!("Running application...");
    let output = app.run(stdout().is_terminal(), false).await?;
    info!("App output: {:?}", output);
    let stdout_handle = stdout().lock();
    let mut bufwriter = BufWriter::new(stdout_handle);
    if let Some(entries) = output.selected_entries {
        for entry in &entries {
            writeln!(
                bufwriter,
                "{}",
                entry.stdout_repr(
                    &app.television.channel.prototype.source.output
                )
            )?;
        }
    }
    bufwriter.flush()?;
    exit(0);
}

/// Apply overrides from the CLI arguments to the configuration.
///
/// This function mutates the configuration in place.
fn apply_cli_overrides(args: &PostProcessedCli, config: &mut Config) {
    if let Some(cable_dir) = &args.cable_dir {
        config.application.cable_dir.clone_from(cable_dir);
    }
    if let Some(tick_rate) = args.tick_rate {
        config.application.tick_rate = tick_rate;
    }
    if args.no_help {
        config.ui.show_help_bar = false;
        config.ui.no_help = true;
    }
    if args.no_preview {
        config.ui.show_preview_panel = false;
    }
    if let Some(ps) = args.preview_size {
        config.ui.preview_size = ps;
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

pub fn handle_subcommand(command: &Command, config: &Config) -> Result<()> {
    match command {
        Command::ListChannels => {
            list_channels(&config.application.cable_dir);
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
        Command::UpdateChannels => {
            update_local_channels()?;
            exit(0);
        }
    }
}

pub fn determine_channel(
    args: &PostProcessedCli,
    config: &Config,
    readable_stdin: bool,
    cable: &Cable,
) -> ChannelPrototype {
    if readable_stdin {
        debug!("Using stdin channel");
        ChannelPrototype::stdin(args.preview_spec.clone())
    } else if let Some(prompt) = &args.autocomplete_prompt {
        debug!("Using autocomplete prompt: {:?}", prompt);
        let channel_prototype = guess_channel_from_prompt(
            prompt,
            &config.shell_integration.commands,
            &config.shell_integration.fallback_channel,
            cable,
        );
        debug!("Using guessed channel: {:?}", channel_prototype);
        channel_prototype
    } else {
        let channel = args
            .channel
            .as_ref()
            .unwrap_or(&config.application.default_channel)
            .clone();

        let mut prototype = cable.get_channel(&channel);
        // use cli preview command if any
        if let Some(pc) = &args.preview_spec {
            prototype.preview = Some(pc.clone());
        }

        prototype
    }
}

#[cfg(test)]
mod tests {
    use rustc_hash::FxHashMap;
    use television::channels::prototypes::{
        ChannelPrototype, CommandSpec, PreviewSpec, Template,
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
            cable_channels.unwrap_or(Cable::from_prototypes(vec![
                ChannelPrototype::new("files", "fd -t f"),
                ChannelPrototype::new("dirs", "ls"),
                ChannelPrototype::new("git", "git status"),
            ]));
        let channel =
            determine_channel(args, config, readable_stdin, &channels);

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

    #[test]
    fn test_determine_channel_standard_case() {
        let channel = Some(String::from("dirs"));
        let args = PostProcessedCli {
            channel,
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
            channel: None,
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
        let preview_spec = PreviewSpec::new(
            CommandSpec::new(
                vec![Template::parse("echo hello").unwrap()],
                false,
                FxHashMap::default(),
            ),
            None,
        );

        let args = PostProcessedCli {
            channel: Some(String::from("dirs")),
            preview_spec: Some(preview_spec),
            ..Default::default()
        };
        let config = Config::default();

        let expected_prototype = ChannelPrototype::new("dirs", "ls")
            .with_preview(args.preview_spec.clone());

        assert_is_correct_channel(
            &args,
            &config,
            false,
            &expected_prototype,
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
