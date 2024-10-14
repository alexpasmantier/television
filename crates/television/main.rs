use std::io::{stdout, IsTerminal, Write};

use clap::Parser;
use color_eyre::Result;
use tracing::{debug, info};

use crate::app::App;
use crate::channels::CliTvChannel;
use crate::cli::Cli;

mod action;
mod app;
mod channels;
mod cli;
mod config;
mod entry;
mod errors;
mod event;
mod fuzzy;
mod logging;
mod previewers;
mod render;
pub mod television;
mod tui;
mod ui;
mod utils;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    crate::errors::init()?;
    crate::logging::init()?;

    let args = Cli::parse();

    let mut app: App = App::new(
        {
            if is_readable_stdin() {
                debug!("Using stdin channel");
                CliTvChannel::Stdin
            } else {
                debug!("Using {:?} channel", args.channel);
                args.channel
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

pub fn is_readable_stdin() -> bool {
    use std::io::IsTerminal;

    #[cfg(unix)]
    fn imp() -> bool {
        use std::{
            fs::File,
            os::{fd::AsFd, unix::fs::FileTypeExt},
        };

        let stdin = std::io::stdin();
        let Ok(fd) = stdin.as_fd().try_clone_to_owned() else {
            return false;
        };
        let file = File::from(fd);
        let Ok(md) = file.metadata() else {
            return false;
        };
        let ft = md.file_type();
        let is_file = ft.is_file();
        let is_fifo = ft.is_fifo();
        let is_socket = ft.is_socket();
        is_file || is_fifo || is_socket
    }

    #[cfg(windows)]
    fn imp() -> bool {
        let stdin = winapi_util::HandleRef::stdin();
        let typ = match winapi_util::file::typ(stdin) {
            Ok(typ) => typ,
            Err(err) => {
                log::debug!(
                    "for heuristic stdin detection on Windows, \
                     could not get file type of stdin \
                     (thus assuming stdin is not readable): {err}",
                );
                return false;
            }
        };
        let is_disk = typ.is_disk();
        let is_pipe = typ.is_pipe();
        let is_readable = is_disk || is_pipe;
        log::debug!(
            "for heuristic stdin detection on Windows, \
             found that is_disk={is_disk} and is_pipe={is_pipe}, \
             and thus concluded that is_stdin_readable={is_readable}",
        );
        is_readable
    }

    #[cfg(not(any(unix, windows)))]
    fn imp() -> bool {
        log::debug!("on non-{{Unix,Windows}}, assuming stdin is not readable");
        false
    }

    !std::io::stdin().is_terminal() && imp()
}
