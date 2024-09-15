use std::path::PathBuf;

use clap::{
    builder::{styling::AnsiColor, Styles},
    Parser,
};
use serde::Serialize;
use serde_with::{serde_as, skip_serializing_none, NoneAsEmptyString};
use tracing::level_filters::LevelFilter;

pub fn version() -> String {
    let version_message = format!(
        "{} ({})",
        env!("CARGO_PKG_VERSION"),
        env!("VERGEN_BUILD_DATE"),
    );
    let author = clap::crate_authors!();

    format!(
        "\
{version_message}

Authors: {author}"
    )
}

const HELP_STYLES: Styles = Styles::styled()
    .header(AnsiColor::Blue.on_default().bold())
    .usage(AnsiColor::Blue.on_default().bold())
    .literal(AnsiColor::White.on_default())
    .placeholder(AnsiColor::Green.on_default());

/// Command line arguments.
///
/// Implements Serialize so that we can use it as a source for Figment
/// configuration.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Default, Parser, Serialize)]
#[command(author, version = version(), about, long_about = None, styles = HELP_STYLES)]
pub struct Cli {
    /// Print default configuration
    #[arg(long)]
    pub print_default_config: bool,

    /// A path to a crates-tui configuration file.
    #[arg(
        short,
        long,
        value_name = "FILE",
        default_value = get_default_config_path()
    )]
    pub config_file: Option<PathBuf>,

    /// A path to a base16 color file.
    #[arg(long, value_name = "FILE", default_value = get_default_color_file())]
    pub color_file: Option<PathBuf>,

    /// Frame rate, i.e. number of frames per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 15.0)]
    pub frame_rate: f64,

    /// The directory to use for storing application data.
    #[arg(long, value_name = "DIR", default_value = get_default_data_dir())]
    pub data_dir: Option<PathBuf>,

    /// The log level to use. Valid values are: error, warn, info, debug, trace, off.
    ///
    /// [default: info]
    #[arg(long, value_name = "LEVEL", alias = "log")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub log_level: Option<LevelFilter>,
}

fn get_default_config_path() -> String {
    crate::config::default_config_file()
        .to_string_lossy()
        .into_owned()
}

fn get_default_color_file() -> String {
    crate::config::default_color_file()
        .to_string_lossy()
        .into_owned()
}

fn get_default_data_dir() -> String {
    crate::config::default_data_dir()
        .to_string_lossy()
        .into_owned()
}

pub fn parse() -> Cli {
    Cli::parse()
}
