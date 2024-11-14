use std::io::{stdout, IsTerminal, Write};

use clap::Parser;
use color_eyre::Result;
use television_channels::channels::TelevisionChannel;
use tracing::{debug, info};

use crate::app::App;
use crate::cli::Cli;
use television_channels::channels::stdin::Channel as StdinChannel;
use television_utils::stdin::is_readable_stdin;

pub mod action;
pub mod app;
pub mod cli;
pub mod config;
pub mod errors;
pub mod event;
pub mod logging;
pub mod picker;
pub mod render;
pub mod television;
pub mod tui;
pub mod ui;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    errors::init()?;
    logging::init()?;

    let args = Cli::parse();

    match App::new(
        {
            if is_readable_stdin() {
                debug!("Using stdin channel");
                TelevisionChannel::Stdin(StdinChannel::default())
            } else {
                debug!("Using {:?} channel", args.channel);
                args.channel.to_channel()
            }
        },
        args.tick_rate,
        args.frame_rate,
    ) {
        Ok(mut app) => {
            if let Some(entry) = app.run(stdout().is_terminal()).await? {
                // print entry to stdout
                stdout().flush()?;
                info!("{:?}", entry);
                writeln!(stdout(), "{}", entry.stdout_repr())?;
            }
            Ok(())
        }
        Err(err) => {
            println!("{err:?}");
            return Ok(());
        }
    }
}
