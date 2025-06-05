use anyhow::Result;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::config::get_data_dir;

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
