use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand};

use crate::utils::resolve_paths;

#[derive(Parser, Debug)]
#[command(name = "television")]
#[command(bin_name = "tv")]
#[command(version, about = "", long_about = None, arg_required_else_help=true)]
pub struct Cli {
    /// a regex pattern to search for
    #[arg(num_args = 1)]
    pub pattern: Option<String>,

    /// you can specify multiple patterns using -e "pattern1" -e "pattern2" etc.
    #[arg(
        short = 'e',
        long,
        action = ArgAction::Append
    )]
    patterns: Vec<String>,

    /// path in which to search recursively
    #[arg(num_args = 0..)]
    pub paths: Vec<PathBuf>,

    /// paths to ignore when recursively walking target directory
    #[clap(short = 'I', long)]
    pub ignore_paths: Vec<PathBuf>,

    /// disregard .gitignore rules when recursively walking directory (defaults to false)
    #[clap(short = 'G', long, default_value_t = false)]
    pub disregard_gitignore: bool,

    /// number of threads to use
    #[clap(short = 'T', long, default_value_t = 4)]
    pub n_threads: usize,

    /// enable multiline matching
    #[clap(short = 'U', long, default_value_t = false)]
    pub multiline: bool,

    /// output in JSON format
    #[clap(long, default_value_t = false)]
    pub json: bool,

    /// output file paths only
    #[clap(short = 'f', long, default_value_t = false)]
    pub file_paths_only: bool,

    /// output absolute paths (defaults to relative)
    #[clap(short = 'A', long, default_value_t = false)]
    pub absolute_paths: bool,

    /// disable colored output (colored by default)
    #[clap(short = 'C', long, default_value_t = false)]
    pub disable_colored_output: bool,

    /// filter on filetype (defaults to all filetypes)
    #[clap(short = 't', long)]
    pub filter_filetypes: Vec<String>,

    /// disable hyperlinks in output (defaults to false)
    #[clap(short = 'H', long, default_value_t = false)]
    pub disable_hyperlinks: bool,

    /// Subcommands
    #[clap(subcommand)]
    pub sub_command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone, Copy)]
pub enum Commands {
    /// Upgrade the crate to its latest version
    Upgrade {
        /// Optional flag for force upgrade
        #[arg(short, long, default_value_t = false)]
        force: bool,
    },

    /// Run gg in interactive mode (`gg int`)
    #[clap(name = "int")]
    Interactive,
}

pub const DEFAULT_PATH: &str = ".";

impl Cli {
    pub fn validate(&mut self) {
        if self.sub_command.is_some() {
            return;
        }
        if self.patterns.is_empty() {
            // If no patterns are provided using -e, the positional argument should be treated as a
            // pattern
            if self.pattern.is_none() {
                eprintln!("error: the following required arguments were not provided: <PATTERN>");
                std::process::exit(1);
            }
        } else if self.pattern.is_some() {
            // If patterns are provided using -e and we have what seems to be an additional positional pattern,
            // it should be interpreted as a path.
            self.paths
                .push(PathBuf::from(self.pattern.clone().unwrap()));
            self.pattern = None;
        } else {
            // If patterns are provided using -e and no positional arguments are provided, use
            // default path
            self.paths = vec![PathBuf::from(DEFAULT_PATH)];
        }
    }
}

#[derive(Debug, Clone)]
pub struct PostProcessedCli {
    pub patterns: Vec<String>,
    pub paths: Vec<PathBuf>,
    pub ignored_paths: Vec<PathBuf>,
    pub n_threads: usize,
    pub disregard_gitignore: bool,
    pub multiline: bool,
    pub absolute_paths: bool,
    pub colored_output: bool,
    pub filter_filetypes: Vec<String>,
    pub disable_hyperlinks: bool,
    pub sub_command: Option<Commands>,
}

impl Default for PostProcessedCli {
    fn default() -> Self {
        PostProcessedCli {
            patterns: Vec::new(),
            paths: Vec::new(),
            ignored_paths: Vec::new(),
            n_threads: 1,
            disregard_gitignore: false,
            multiline: false,
            absolute_paths: false,
            colored_output: true,
            filter_filetypes: Vec::new(),
            disable_hyperlinks: false,
            sub_command: None,
        }
    }
}

pub fn process_cli_args(mut cli: Cli) -> anyhow::Result<PostProcessedCli> {
    cli.validate();

    if cli.paths.is_empty() {
        cli.paths.push(PathBuf::from(DEFAULT_PATH));
    }

    if cli.sub_command.is_some() {
        return Ok(PostProcessedCli {
            sub_command: cli.sub_command,
            ..Default::default()
        });
    }
    Ok(PostProcessedCli {
        patterns: if !cli.patterns.is_empty() {
            cli.patterns
        } else {
            vec![cli.pattern.unwrap()]
        },
        paths: resolve_paths(cli.paths),
        ignored_paths: resolve_paths(cli.ignore_paths),
        n_threads: cli.n_threads,
        disregard_gitignore: cli.disregard_gitignore,
        multiline: cli.multiline,
        absolute_paths: cli.absolute_paths,
        colored_output: !cli.disable_colored_output,
        filter_filetypes: cli.filter_filetypes,
        disable_hyperlinks: cli.disable_hyperlinks,
        sub_command: cli.sub_command,
    })
}
