use std::env;
use std::io::{stdout, IsTerminal, Write};
use std::path::Path;
use std::process::exit;

use clap::Parser;
use color_eyre::Result;
use tracing::{debug, error, info};

use crate::app::App;
use crate::cli::{
    guess_channel_from_prompt, list_channels, Cli, ParsedCliChannel,
    PostProcessedCli,
};
use crate::config::Config;
use television_channels::{
    channels::{stdin::Channel as StdinChannel, TelevisionChannel},
    entry::PreviewType,
};
use television_utils::{
    shell::{completion_script, Shell},
    stdin::is_readable_stdin,
};

pub mod action;
pub mod app;
pub mod cable;
pub mod cli;
pub mod config;
pub mod errors;
pub mod event;
pub mod input;
pub mod keymap;
pub mod logging;
pub mod picker;
pub mod render;
pub mod television;
pub mod tui;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    errors::init()?;
    logging::init()?;

    let args: PostProcessedCli = Cli::parse().into();
    debug!("{:?}", args);

    if let Some(command) = args.command {
        match command {
            cli::Command::ListChannels => {
                list_channels();
                exit(0);
            }
            cli::Command::InitShell { shell } => {
                let script = completion_script(Shell::from(shell))?;
                println!("{script}");
                exit(0);
            }
        }
    }

    let mut config = Config::new()?;
    config.config.tick_rate =
        args.tick_rate.unwrap_or(config.config.tick_rate);
    config.config.frame_rate =
        args.frame_rate.unwrap_or(config.config.frame_rate);

    if let Some(working_directory) = args.working_directory {
        let path = Path::new(&working_directory);
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

    match App::new(
        {
            if is_readable_stdin() {
                debug!("Using stdin channel");
                TelevisionChannel::Stdin(StdinChannel::new(
                    args.preview_command.map(PreviewType::Command),
                ))
            } else if let Some(prompt) = args.autocomplete_prompt {
                let channel = guess_channel_from_prompt(
                    &prompt,
                    &config.shell_integration.commands,
                )?;
                debug!("Using guessed channel: {:?}", channel);
                match channel {
                    ParsedCliChannel::Builtin(c) => c.to_channel(),
                    ParsedCliChannel::Cable(c) => {
                        TelevisionChannel::Cable(c.into())
                    }
                }
            } else {
                debug!("Using {:?} channel", args.channel);
                match args.channel {
                    ParsedCliChannel::Builtin(c) => c.to_channel(),
                    ParsedCliChannel::Cable(c) => {
                        TelevisionChannel::Cable(c.into())
                    }
                }
            }
        },
        config,
        &args.passthrough_keybindings,
    ) {
        Ok(mut app) => {
            stdout().flush()?;
            let output = app.run(stdout().is_terminal()).await?;
            info!("{:?}", output);
            if let Some(passthrough) = output.passthrough {
                writeln!(stdout(), "{passthrough}")?;
            }
            if let Some(entry) = output.selected_entry {
                writeln!(stdout(), "{}", entry.stdout_repr())?;
            }
            exit(0);
        }
        Err(err) => {
            println!("{err:?}");
            exit(1);
        }
    }
}
