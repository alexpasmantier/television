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
#[derive(Copy, Clone, Debug)]
pub struct Config {
    /// The number of threads to use for the fuzzy matcher.
    pub n_threads: Option<usize>,
    /// Whether to ignore case when matching.
    pub ignore_case: bool,
    /// Whether to prefer prefix matches.
    pub prefer_prefix: bool,
    /// Whether to optimize for matching paths.
    pub match_paths: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            n_threads: None,
            ignore_case: true,
            prefer_prefix: false,
            match_paths: false,
        }
    }
}

impl Config {
    /// Set the number of threads to use.
    pub fn n_threads(mut self, n_threads: usize) -> Self {
        self.n_threads = Some(n_threads);
        self
    }

    /// Set whether to ignore case.
    pub fn ignore_case(mut self, ignore_case: bool) -> Self {
        self.ignore_case = ignore_case;
        self
    }

    /// Set whether to prefer prefix matches.
    pub fn prefer_prefix(mut self, prefer_prefix: bool) -> Self {
        self.prefer_prefix = prefer_prefix;
        self
    }

    /// Set whether to optimize for matching paths.
    pub fn match_paths(mut self, match_paths: bool) -> Self {
        self.match_paths = match_paths;
        self
    }
}

impl From<&Config> for nucleo::Config {
    fn from(config: &Config) -> Self {
        let mut matcher_config = nucleo::Config::DEFAULT;
        matcher_config.ignore_case = config.ignore_case;
        matcher_config.prefer_prefix = config.prefer_prefix;
        if config.match_paths {
            matcher_config = matcher_config.match_paths();
        }
        matcher_config
    }
}
