use std::io::{stdout, IsTerminal, Write};
use std::process::exit;

use clap::Parser;
use cli::{list_channels, ParsedCliChannel, PostProcessedCli};
use color_eyre::Result;
use television_channels::channels::TelevisionChannel;
use television_channels::entry::PreviewType;
use tracing::{debug, info};

use crate::app::App;
use crate::cli::Cli;
use television_channels::channels::stdin::Channel as StdinChannel;
use television_utils::stdin::is_readable_stdin;

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
        }
    }

    match App::new(
        {
            if is_readable_stdin() {
                debug!("Using stdin channel");
                TelevisionChannel::Stdin(StdinChannel::new(
                    args.preview_command.map(PreviewType::Command),
                ))
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
        args.tick_rate,
        args.frame_rate,
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
