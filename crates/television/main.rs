use std::io::{stdout, IsTerminal, Write};

use channels::TelevisionChannel;
use clap::Parser;
use color_eyre::Result;
use tracing::{debug, info};

use crate::app::App;
use crate::channels::stdin::Channel as StdinChannel;
use crate::cli::Cli;
use crate::utils::is_readable_stdin;

pub mod action;
pub mod app;
pub mod channels;
pub mod cli;
pub mod config;
pub mod entry;
pub mod errors;
pub mod event;
pub mod logging;
pub mod picker;
pub mod previewers;
pub mod render;
pub mod television;
pub mod tui;
pub mod ui;
pub mod utils;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    errors::init()?;
    logging::init()?;

    let args = Cli::parse();

    let mut app: App = App::new(
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
    )?;

    if let Some(entry) = app.run(stdout().is_terminal()).await? {
        // print entry to stdout
        stdout().flush()?;
        info!("{:?}", entry);
        writeln!(stdout(), "{}", entry.stdout_repr())?;
    }
    Ok(())
}
