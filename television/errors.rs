use std::panic;

use anyhow::Result;
use tracing::error;

/// Sets up a panic hook to provide consistent error handling.
///
/// The hook will:
/// - Clean up the terminal before exiting to avoid leaving it in an
///     inconsistent state for the user.
///
/// - In release builds, use human-panic to generate a friendly crash report.
///     More information can be found at <https://crates.io/crates/human-panic>
///
/// - In debug builds, use better-panic for a more detailed dev stacktrace.
///     More information can be found at <https://crates.io/crates/better-panic>
///
/// - Finally, exit the process with a status code of 1.
pub fn init() -> Result<()> {
    panic::set_hook(Box::new(move |panic_info| {
        // Clean up the terminal
        if let Ok(mut t) = crate::tui::Tui::new(std::io::stderr()) {
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
