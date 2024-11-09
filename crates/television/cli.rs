use clap::Parser;

use crate::config::{get_config_dir, get_data_dir};
use television_channels::channels::CliTvChannel;

#[derive(Parser, Debug)]
#[command(author, version = version(), about)]
pub struct Cli {
    /// Which channel shall we watch?
    #[arg(value_enum, default_value = "files")]
    pub channel: CliTvChannel,

    /// Tick rate, i.e. number of ticks per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 50.0)]
    pub tick_rate: f64,

    /// Frame rate, i.e. number of frames per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 60.0)]
    pub frame_rate: f64,
}

const VERSION_MESSAGE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\ntarget triple: ",
    env!("VERGEN_CARGO_TARGET_TRIPLE"),
    "\nbuild: ",
    env!("VERGEN_RUSTC_SEMVER"),
    " (",
    env!("VERGEN_BUILD_DATE"),
    ")"
);

pub fn version() -> String {
    let author = clap::crate_authors!();

    // let current_exe_path = PathBuf::from(clap::crate_name!()).display().to_string();
    let config_dir_path = get_config_dir().display().to_string();
    let data_dir_path = get_data_dir().display().to_string();

    format!(
        "\
{VERSION_MESSAGE}

           _______________
          |,----------.  |\\
          ||           |=| |
          ||          || | |
          ||       . _o| | |
          |`-----------' |/
           ~~~~~~~~~~~~~~~
  __      __         _     _
 / /____ / /__ _  __(_)__ (_)__  ___ 
/ __/ -_) / -_) |/ / (_-</ / _ \\/ _ \\
\\__/\\__/_/\\__/|___/_/___/_/\\___/_//_/

Authors: {author}

Config directory: {config_dir_path}
Data directory: {data_dir_path}"
    )
}
