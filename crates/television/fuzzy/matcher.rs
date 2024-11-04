use std::sync::Arc;

use super::MATCHER;

const MATCHER_TICK_TIMEOUT: u64 = 2;

/// A matched item.
///
/// This contains the matched item, the dimension against which it was matched,
/// represented as a string, and the indices of the matched characters.
///
/// The indices are pairs of `(start, end)` where `start` is the index of the
/// first character in the match, and `end` is the index of the character after
/// the last character in the match.
pub struct MatchedItem<I>
where
    I: Sync + Send + Clone + 'static,
{
    pub inner: I,
    pub matched_string: String,
    pub match_indices: Vec<(u32, u32)>,
}

/// The status of the fuzzy matcher.
///
/// This currently only contains a boolean indicating whether the matcher is
/// running in the background.
/// This mostly serves as a way to communicate the status of the matcher to the
/// front-end and display a loading indicator.
#[derive(Default)]
pub struct Status {
    pub running: bool,
}

impl From<nucleo::Status> for Status {
    fn from(status: nucleo::Status) -> Self {
        Self {
            running: status.running,
        }
    }
}

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
#[derive(Copy, Clone)]
pub struct Config {
    pub n_threads: Option<usize>,
    pub ignore_case: bool,
    pub prefer_prefix: bool,
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

/// An injector that can be used to push items of type `I` into the fuzzy matcher.
///
/// This is a wrapper around the `Injector` type from the `Nucleo` fuzzy matcher.
///
/// The `push` method takes an item of type `I` and a closure that produces the
/// string to match against based on the item.
#[derive(Clone)]
pub struct Injector<I>
where
    I: Sync + Send + Clone + 'static,
{
    inner: nucleo::Injector<I>,
}

impl<I> Injector<I>
where
    I: Sync + Send + Clone + 'static,
{
    pub fn new(inner: nucleo::Injector<I>) -> Self {
        Self { inner }
    }

    /// Push an item into the fuzzy matcher.
    ///
    /// The closure `f` should produce the string to match against based on the
    /// item.
    ///
    /// # Example
    /// ```
    /// let config = Config::default();
    /// let matcher = Matcher::new(config);
    ///
    /// let injector = matcher.injector();
    /// injector.push(
    ///     ("some string", 3, "some other string"),
    ///     // Say we want to match against the third element of the tuple
    ///     |s, cols| cols[0] = s.2.into()
    /// );
    /// ```
    pub fn push<F>(&self, item: I, f: F)
    where
        F: FnOnce(&I, &mut [nucleo::Utf32String]),
    {
        self.inner.push(item, f);
    }
}

/// A fuzzy matcher that can be used to match items of type `I`.
///
/// `I` should be `Sync`, `Send`, `Clone`, and `'static`.
/// This is a wrapper around the `Nucleo` fuzzy matcher that only matches
/// on a single dimension.
///
/// The matcher can be used to find items that match a given pattern and to
/// retrieve the matched items as well as the indices of the matched characters.
pub struct Matcher<I>
where
    I: Sync + Send + Clone + 'static,
{
    inner: nucleo::Nucleo<I>,
    pub total_item_count: u32,
    pub matched_item_count: u32,
    pub status: Status,
    pub last_pattern: String,
}

impl<I> Matcher<I>
where
    I: Sync + Send + Clone + 'static,
{
    /// Create a new fuzzy matcher with the given configuration.
    pub fn new(config: Config) -> Self {
        Self {
            inner: nucleo::Nucleo::new(
                (&config).into(),
                Arc::new(|| {}),
                config.n_threads,
                1,
            ),
            total_item_count: 0,
            matched_item_count: 0,
            status: Status::default(),
            last_pattern: String::new(),
        }
    }

    /// Tick the fuzzy matcher.
    ///
    /// This should be called periodically to update the state of the matcher.
    pub fn tick(&mut self) {
        self.status = self.inner.tick(MATCHER_TICK_TIMEOUT).into();
    }

    /// Get an injector that can be used to push items into the fuzzy matcher.
    ///
    /// This can be used at any time to push items into the fuzzy matcher.
    ///
    /// # Example
    /// ```
    /// let config = Config::default();
    /// let matcher = Matcher::new(config);
    /// let injector = matcher.injector();
    ///
    /// injector.push(
    ///     ("some string", 3, "some other string"),
    ///     // Say we want to match against the third element of the tuple
    ///     |s, cols| cols[0] = s.2.into()
    /// );
    /// ```
    pub fn injector(&self) -> Injector<I> {
        Injector::new(self.inner.injector())
    }

    /// Find items that match the given pattern.
    ///
    /// This should be called whenever the pattern changes.
    /// The `Matcher` will keep track of the last pattern and only reparse the
    /// pattern if it has changed, allowing for more efficient matching when
    /// `self.last_pattern` is a prefix of the new `pattern`.
    pub fn find(&mut self, pattern: &str) {
        if pattern != self.last_pattern {
            self.inner.pattern.reparse(
                0,
                pattern,
                nucleo::pattern::CaseMatching::Smart,
                nucleo::pattern::Normalization::Smart,
                pattern.starts_with(&self.last_pattern),
            );
            self.last_pattern = pattern.to_string();
        }
    }

    /// Get the matched items.
    ///
    /// This should be called to retrieve the matched items after calling
    /// `find`.
    ///
    /// The `num_entries` parameter specifies the number of entries to return,
    /// and the `offset` parameter specifies the offset of the first entry to
    /// return.
    ///
    /// The returned items are `MatchedItem`s that contain the matched item, the
    /// dimension against which it was matched, represented as a string, and the
    /// indices of the matched characters.
    ///
    /// # Example
    /// ```
    /// let config = Config::default();
    /// let matcher = Matcher::new(config);
    /// matcher.find("some pattern");
    ///
    /// let results = matcher.results(10, 0);
    /// for item in results {
    ///     println!("{:?}", item);
    ///     // Do something with the matched item
    ///     // ...
    ///     // Do something with the matched indices
    ///     // ...
    /// }
    /// ```
    pub fn results(
        &mut self,
        num_entries: u32,
        offset: u32,
    ) -> Vec<MatchedItem<I>> {
        let snapshot = self.inner.snapshot();
        self.total_item_count = snapshot.item_count();
        self.matched_item_count = snapshot.matched_item_count();

        let mut col_indices = Vec::new();
        let mut matcher = MATCHER.lock();

        snapshot
            .matched_items(
                offset..(num_entries + offset).min(self.matched_item_count),
            )
            .map(move |item| {
                snapshot.pattern().column_pattern(0).indices(
                    item.matcher_columns[0].slice(..),
                    &mut matcher,
                    &mut col_indices,
                );
                col_indices.sort_unstable();
                col_indices.dedup();

                let indices = col_indices.drain(..);

                let matched_string = item.matcher_columns[0].to_string();
                MatchedItem {
                    inner: item.data.clone(),
                    matched_string,
                    match_indices: indices.map(|i| (i, i + 1)).collect(),
                }
            })
            .collect()
    }

    /// Get a single matched item.
    ///
    /// # Example
    /// ```
    /// let config = Config::default();
    /// let matcher = Matcher::new(config);
    /// matcher.find("some pattern");
    ///
    /// if let Some(item) = matcher.get_result(0) {
    ///     println!("{:?}", item);
    ///     // Do something with the matched item
    ///     // ...
    /// }
    /// ```
    pub fn get_result(&self, index: u32) -> Option<MatchedItem<I>> {
        let snapshot = self.inner.snapshot();
        snapshot.get_matched_item(index).map(|item| {
            let matched_string = item.matcher_columns[0].to_string();
            MatchedItem {
                inner: item.data.clone(),
                matched_string,
                match_indices: Vec::new(),
            }
        })
    }
}
