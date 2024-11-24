use clap::Parser;

use crate::config::{get_config_dir, get_data_dir};
use television_channels::channels::CliTvChannel;

#[derive(Parser, Debug)]
#[command(author, version = version(), about)]
pub struct Cli {
    /// Which channel shall we watch?
    #[arg(value_enum, default_value = "files")]
    pub channel: CliTvChannel,

    /// Use a custom preview command (currently only supported by the stdin channel)
    #[arg(short, long, value_name = "STRING")]
    pub preview: Option<String>,

    /// Tick rate, i.e. number of ticks per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 50.0)]
    pub tick_rate: f64,

    /// Frame rate, i.e. number of frames per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 60.0)]
    pub frame_rate: f64,

    /// Passthrough keybindings (comma separated, e.g. "q,ctrl-w,ctrl-t") These keybindings will
    /// trigger selection of the current entry and be passed through to stdout along with the entry
    /// to be handled by the parent process.
    #[arg(long, value_name = "STRING")]
    pub passthrough_keybindings: Option<String>,
}

#[derive(Debug)]
pub struct PostProcessedCli {
    pub channel: CliTvChannel,
    pub preview: Option<String>,
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub passthrough_keybindings: Vec<String>,
}

impl From<Cli> for PostProcessedCli {
    fn from(cli: Cli) -> Self {
        let passthrough_keybindings = cli
            .passthrough_keybindings
            .unwrap_or_default()
            .split(',')
            .map(std::string::ToString::to_string)
            .collect();

        Self {
            channel: cli.channel,
            preview: cli.preview,
            tick_rate: cli.tick_rate,
            frame_rate: cli.frame_rate,
            passthrough_keybindings,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_from_cli() {
        let cli = Cli {
            channel: CliTvChannel::Files,
            preview: Some("bat -n --color=always {}".to_string()),
            tick_rate: 50.0,
            frame_rate: 60.0,
            passthrough_keybindings: Some("q,ctrl-w,ctrl-t".to_string()),
        };

        let post_processed_cli: PostProcessedCli = cli.into();

        assert_eq!(post_processed_cli.channel, CliTvChannel::Files);
        assert_eq!(
            post_processed_cli.preview,
            Some("bat -n --color=always {}".to_string())
        );
        assert_eq!(post_processed_cli.tick_rate, 50.0);
        assert_eq!(post_processed_cli.frame_rate, 60.0);
        assert_eq!(
            post_processed_cli.passthrough_keybindings,
            vec!["q".to_string(), "ctrl-w".to_string(), "ctrl-t".to_string()]
        );
    }
}
