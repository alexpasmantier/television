use crate::tui::{Tui, TuiMode};
use anyhow::Result;
use colored::Colorize;
use std::panic;
use tracing::error;

pub fn init() -> Result<()> {
    panic::set_hook(Box::new(move |panic_info| {
        // Clean up the terminal
        if let Ok(mut t) = Tui::new(std::io::stderr(), &TuiMode::Fullscreen) {
            if let Err(err) = t.exit() {
                error!("Unable to exit terminal: {:?}", err);
            }
        }

        // In release builds, use human-panic to generate a friendly crash report:
        #[cfg(not(debug_assertions))]
        {
            use human_panic::{handle_dump, metadata, print_msg};
            let meta = metadata!();
            let file_path = handle_dump(&meta, panic_info);
            print_msg(file_path, &meta).expect(
                "human-panic: printing error message to console failed",
            );
        }

        // In debug builds, use better-panic for a more detailed dev stacktrace:
        #[cfg(debug_assertions)]
        {
            better_panic::Settings::auto()
                .most_recent_first(false)
                .lineno_suffix(true)
                .verbosity(better_panic::Verbosity::Full)
                .create_panic_handler()(panic_info);
        }

        std::process::exit(1);
    }));

    Ok(())
}

pub fn cli_parsing_error_exit(message: &str) -> ! {
    eprintln!("Error parsing CLI arguments: {message}\n");
    std::process::exit(1);
}

pub fn unknown_channel_exit(channel: &str) -> ! {
    eprintln!(
        "Channel not found: {}\n\nTry running {} to update the channel list.\n\nSee {} for more information.",
        channel.red(),
        "tv update-channels [--force]".blue(),
        "tv --help".blue()
    );
    std::process::exit(1);
}

pub fn os_error_exit(message: &str) -> ! {
    eprintln!("OS error: {message}\n");
    std::process::exit(1);
}
