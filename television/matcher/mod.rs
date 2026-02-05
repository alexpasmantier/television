use injector::Injector;
use nucleo::Utf32Str;
use std::sync::Arc;

pub mod config;
pub mod injector;
pub mod lazy;
pub mod matched_item;

const MATCHER_TICK_TIMEOUT: u64 = 2;

/// Convert a `Utf32String` to a `String` efficiently.
///
/// For ASCII content (the common case), this performs a direct byte copy
/// instead of going through the char-by-char `Display` implementation.
#[inline]
fn utf32_to_string(s: &nucleo::Utf32String) -> String {
    match s.slice(..) {
        Utf32Str::Ascii(bytes) => {
            // SAFETY: Utf32Str::Ascii invariant guarantees valid ASCII, which is valid UTF-8
            unsafe { std::str::from_utf8_unchecked(bytes) }.to_owned()
        }
        Utf32Str::Unicode(chars) => chars.iter().collect(),
    }
}

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
    /// A pre-allocated buffer used to collect match indices when fetching the results
    /// from the matcher. This avoids having to re-allocate on each pass.
    col_indices_buffer: Vec<u32>,
}

impl<I> Matcher<I>
where
    I: Sync + Send + Clone + 'static,
{
    /// Create a new fuzzy matcher with the given configuration and sort strategy.
    pub fn new(
        config: &config::Config,
        sort_strategy: config::SortStrategy<I>,
    ) -> Self {
        let mut inner = nucleo::Nucleo::new(
            config.into(),
            Arc::new(|| {}),
            config.n_threads,
            1,
        );
        inner.set_sort_strategy(sort_strategy);
        Self {
            inner,
            total_item_count: 0,
            matched_item_count: 0,
            status: Status::default(),
            last_pattern: String::new(),
            col_indices_buffer: Vec::with_capacity(128), // Pre-allocate for performance
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
    /// use television::matcher::{config::{Config, SortStrategy}, Matcher};
    ///
    /// let config = Config::default();
    /// let matcher = Matcher::new(&config, SortStrategy::Score);
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
    /// use television::matcher::{config::{Config, SortStrategy}, Matcher};
    ///
    /// let config = Config::default();
    /// let mut matcher: Matcher<String> = Matcher::new(&config, SortStrategy::Score);
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

        // If the offset is greater than the number of matched items, return an empty Vec
        if offset >= self.matched_item_count {
            return Vec::new();
        }
        // If we're asking for more entries than available, adjust the count
        let num_entries = if offset + num_entries > self.matched_item_count {
            self.matched_item_count - offset
        } else {
            num_entries
        };

        // Clear the pre-allocated match indices buffer for safety
        self.col_indices_buffer.clear();
        let mut matcher = lazy::MATCHER.lock();

        // PERF: Pre-allocate the results Vec so we avoid repeated reallocations
        let mut results = Vec::with_capacity(num_entries as usize);

        for item in snapshot.matched_items(
            offset..(num_entries + offset).min(self.matched_item_count),
        ) {
            snapshot.pattern().column_pattern(0).indices(
                item.matcher_columns[0].slice(..),
                &mut matcher,
                &mut self.col_indices_buffer,
            );

            // PERF: Avoid unnecessary sorting
            if self.col_indices_buffer.len() > 1 {
                self.col_indices_buffer.sort_unstable();
                self.col_indices_buffer.dedup();
            }

            let indices: Vec<u32> =
                self.col_indices_buffer.drain(..).collect();
            let matched_string = utf32_to_string(&item.matcher_columns[0]);

            results.push(matched_item::MatchedItem {
                inner: item.data.clone(),
                matched_string,
                match_indices: indices,
            });
        }

        results
    }

    /// Get a single matched item.
    ///
    /// # Example
    /// ```ignore
    /// use television::matcher::{config::{Config, SortStrategy}, Matcher};
    ///
    /// let config = Config::default();
    /// let mut matcher: Matcher<String> = Matcher::new(&config, SortStrategy::Score);
    /// matcher.find("some pattern");
    ///
    /// if let Some(item) = matcher.get_result(0) {
    ///     // Do something with the matched item
    ///     // ...
    /// }
    /// ```
    pub fn get_result(
        &mut self,
        index: u32,
    ) -> Option<matched_item::MatchedItem<I>> {
        let snapshot = self.inner.snapshot();
        let mut matcher = lazy::MATCHER.lock();
        self.col_indices_buffer.clear();

        snapshot.get_matched_item(index).map(|item| {
            snapshot.pattern().column_pattern(0).indices(
                item.matcher_columns[0].slice(..),
                &mut matcher,
                &mut self.col_indices_buffer,
            );
            if self.col_indices_buffer.len() > 1 {
                self.col_indices_buffer.sort_unstable();
                self.col_indices_buffer.dedup();
            }

            let indices = self.col_indices_buffer.drain(..);
            let matched_string = utf32_to_string(&item.matcher_columns[0]);

            matched_item::MatchedItem {
                inner: item.data.clone(),
                matched_string,
                match_indices: indices.collect(),
            }
        })
    }

    /// Restart the matcher.
    ///
    /// This will reset the matcher to its initial state, clearing all
    /// matched items and the last pattern.
    pub fn restart(&mut self) {
        self.inner.restart(true);
        self.total_item_count = 0;
        self.matched_item_count = 0;
        self.status = Status::default();
        self.last_pattern.clear();
        self.col_indices_buffer.clear();
    }

    /// Set whether to reverse the input order.
    ///
    /// When enabled, items with higher indices (added later) will appear first
    /// when scores are equal. This is useful for channels like shell history
    /// where more recent entries should appear at the top.
    ///
    /// Defaults to `false`.
    pub fn reverse_items(&mut self, reverse: bool) {
        self.inner.reverse_items(reverse);
    }
}
