use anyhow::Result;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::config::get_data_dir;

/// Initializes the logging system.
///
/// This function sets up the logging system with a file subscriber that writes
/// logs to a file in the data directory.
///
/// The data directory is determined in `television::config::get_data_dir` and
/// is created if it does not exist.
///
/// The log file can be found at: `<data_dir>/television.log`.
///
/// Log messages are filtered based on the `RUST_LOG` environment variable which
/// can be set to one of the following values:
/// - `error`
/// - `warn`
/// - `info`
/// - `debug`
/// - `trace`
pub fn init() -> Result<()> {
    let directory = get_data_dir();
    std::fs::create_dir_all(directory.clone())?;
    let log_path = directory.join(format!("{}.log", env!("CARGO_PKG_NAME")));
    let log_file = std::fs::File::create(log_path)?;
    let file_subscriber = fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(log_file)
        .with_target(false)
        .with_ansi(false)
        .with_filter(EnvFilter::from_default_env());

    tracing_subscriber::registry()
        .with(file_subscriber)
        .try_init()?;
    Ok(())
}
