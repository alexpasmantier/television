use injector::Injector;
use std::sync::Arc;

pub mod config;
pub mod injector;
pub mod lazy;
pub mod matched_item;

const MATCHER_TICK_TIMEOUT: u64 = 2;

/// The status of the fuzzy matcher.
///
/// This currently only contains a boolean indicating whether the matcher is
/// running in the background.
/// This mostly serves as a way to communicate the status of the matcher to the
/// front-end and display a loading indicator.
#[derive(Default, Debug, Clone, Copy)]
pub struct Status {
    /// Whether the matcher is currently running.
    pub running: bool,
}

impl From<nucleo::Status> for Status {
    fn from(status: nucleo::Status) -> Self {
        Self {
            running: status.running,
        }
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
    /// The inner `Nucleo` fuzzy matcher.
    inner: nucleo::Nucleo<I>,
    /// The current total number of items in the matcher.
    pub total_item_count: u32,
    /// The current number of matched items in the matcher.
    pub matched_item_count: u32,
    /// The current status of the matcher.
    pub status: Status,
    /// The last pattern that was matched against.
    pub last_pattern: String,
}

impl<I> Matcher<I>
where
    I: Sync + Send + Clone + 'static,
{
    /// Create a new fuzzy matcher with the given configuration.
    pub fn new(config: config::Config) -> Self {
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
    /// ```ignore
    /// use television::matcher::{config::Config, Matcher};
    ///
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
    /// ```ignore
    /// use television::matcher::{config::Config, Matcher};
    ///
    /// let config = Config::default();
    /// let mut matcher: Matcher<String> = Matcher::new(config);
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
    ) -> Vec<matched_item::MatchedItem<I>> {
        let snapshot = self.inner.snapshot();
        self.total_item_count = snapshot.item_count();
        self.matched_item_count = snapshot.matched_item_count();

        let mut col_indices = Vec::new();
        let mut matcher = lazy::MATCHER.lock();

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
                matched_item::MatchedItem {
                    inner: item.data.clone(),
                    matched_string,
                    match_indices: indices.collect(),
                }
            })
            .collect()
    }

    /// Get a single matched item.
    ///
    /// # Example
    /// ```ignore
    /// use television::matcher::{config::Config, Matcher};
    ///
    /// let config = Config::default();
    /// let mut matcher: Matcher<String> = Matcher::new(config);
    /// matcher.find("some pattern");
    ///
    /// if let Some(item) = matcher.get_result(0) {
    ///     // Do something with the matched item
    ///     // ...
    /// }
    /// ```
    pub fn get_result(
        &self,
        index: u32,
    ) -> Option<matched_item::MatchedItem<I>> {
        let snapshot = self.inner.snapshot();
        snapshot.get_matched_item(index).map(|item| {
            let matched_string = item.matcher_columns[0].to_string();
            matched_item::MatchedItem {
                inner: item.data.clone(),
                matched_string,
                match_indices: Vec::new(),
            }
        })
    }
}
