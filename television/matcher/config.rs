/// The configuration of the fuzzy matcher.
///
/// This contains the number of threads to use, whether to ignore case, whether
/// to prefer prefix matches, and whether to optimize for matching paths.
///
/// The default configuration uses the default configuration of the `Nucleo`
/// fuzzy matcher, e.g. case-insensitive matching, no preference for prefix
/// matches, and no optimization for matching paths as well as using the
/// default number of threads (which corresponds to the number of available logical
/// cores on the current machine).
#[derive(Debug, Clone)]
pub struct Config {
    /// Whether to prefer prefix matches.
    pub prefer_prefix: bool,
    /// The number of threads to use for matching.
    pub n_threads: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prefer_prefix: true,
            n_threads: Some(
                std::thread::available_parallelism()
                    .map(std::num::NonZeroUsize::get)
                    .unwrap_or(4)
                    .min(8),
            ),
        }
    }
}

impl Config {
    /// Create a new configuration with the default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to prefer prefix matches.
    pub fn prefer_prefix(mut self, prefer_prefix: bool) -> Self {
        self.prefer_prefix = prefer_prefix;
        self
    }

    /// Set the number of threads to use for matching.
    pub fn n_threads(mut self, n_threads: Option<usize>) -> Self {
        self.n_threads = n_threads;
        self
    }
}

impl From<&Config> for nucleo::Config {
    fn from(config: &Config) -> Self {
        let mut nucleo_config = nucleo::Config::DEFAULT;
        nucleo_config.prefer_prefix = config.prefer_prefix;
        nucleo_config
    }
}
