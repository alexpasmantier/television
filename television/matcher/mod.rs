use frizbee::Match;
use injector::Injector;
use matched_item::{MatchedItem, byte_indices_to_char_indices};
use parking_lot::{Mutex, RwLock};
use std::cmp::Ordering as CmpOrdering;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    },
    thread::available_parallelism,
    time::Duration,
};

pub mod injector;
pub mod matched_item;
mod worker;

use worker::{Snapshot, Store, Worker, WorkerMsg};

// pub use frizbee::SortStrategy;

/// Comparison function for custom sorting of match results.
pub type SortFn<I> = Box<
    dyn Fn(&Match, &I, &str, &Match, &I, &str) -> CmpOrdering + Send + Sync,
>;

/// Strategy for sorting match results.
#[derive(Default)]
pub enum SortStrategy<I: Sync + Send + 'static> {
    /// Sort by score (desc), then index (asc)
    #[default]
    Score,
    /// Sort items by index (asc)
    Index,
    /// Custom comparison function
    Custom(SortFn<I>),
}

impl<I: Sync + Send + 'static> std::fmt::Debug for SortStrategy<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortStrategy::Score => write!(f, "SortStrategy::Score"),
            SortStrategy::Index => write!(f, "SortStrategy::Index"),
            SortStrategy::Custom(_) => write!(f, "SortStrategy::Custom(...)"),
        }
    }
}

/// A callback the matcher invokes (on the worker thread) after it publishes
/// a fresh snapshot of results.
pub type Notify = Arc<dyn Fn() + Send + Sync>;

/// A fuzzy matcher that can be used to match items of type `I`.
///
/// This is a wrapper around the `frizbee` fuzzy matcher with matching on a
/// dedicated background thread. Items are pushed to the matcher via injectors.
///
/// [`Matcher::find`] updates the background thread's pattern
/// [`Matcher::results`] reads the latest matched results
/// [`Matcher::get_result`] reads a single result
#[allow(clippy::struct_field_names)]
pub struct Matcher<I>
where
    I: Sync + Send + Clone + 'static,
{
    /// Collection of (item, haystacks) pairs to be matched against.
    store: Arc<RwLock<Store<I>>>,
    /// Last snapshot of matches published by the background worker.
    snapshot: Arc<Mutex<Arc<Snapshot>>>,
    /// Channel used to notify the background worker of changes.
    worker_tx: mpsc::Sender<WorkerMsg<I>>,
    /// Whether the background worker is currently matching or has pending
    /// work.
    running: Arc<AtomicBool>,
    /// Bumped on every restart so that snapshots computed against a previous
    /// store can be detected and discarded.
    generation: u64,
    /// Live count of items pushed through injectors for the current store,
    /// swapped together with the store on restart. Kept separate from the
    /// store so the count stays current while batches are still in flight
    /// to the worker.
    count: Arc<AtomicUsize>,
    /// The last pattern passed to `find`, used to avoid notifying the worker
    /// when the pattern hasn't changed.
    last_pattern: String,
}

impl<I> Matcher<I>
where
    I: Sync + Send + Clone + 'static,
{
    /// Create a new fuzzy matcher with the given sort strategy and number of
    /// threads.
    ///
    /// Use [`Matcher::with_notify`] to be woken as soon as fresh results are
    /// available.
    pub fn new(sort_strategy: SortStrategy<I>, n_threads: usize) -> Self {
        Self::with_notify(sort_strategy, n_threads, Arc::new(|| {}))
    }

    /// Create a new fuzzy matcher that calls `notify` every time the background
    /// worker publishes fresh results.
    pub fn with_notify(
        sort_strategy: SortStrategy<I>,
        n_threads: usize,
        notify: Notify,
    ) -> Self {
        Self::build(
            sort_strategy,
            n_threads,
            notify,
            worker::INITIAL_CHUNK_SIZE,
        )
    }

    /// Create a matcher with a custom initial chunk size, small enough to
    /// force multi-chunk passes on small stores.
    #[cfg(test)]
    fn with_chunk_size(
        sort_strategy: SortStrategy<I>,
        n_threads: usize,
        chunk_size: usize,
    ) -> Self {
        Self::build(sort_strategy, n_threads, Arc::new(|| {}), chunk_size)
    }

    fn build(
        sort_strategy: SortStrategy<I>,
        n_threads: usize,
        notify: Notify,
        initial_chunk_size: usize,
    ) -> Self {
        let store = Arc::new(RwLock::new(Store::default()));
        let snapshot = Arc::new(Mutex::new(Arc::new(Snapshot::empty(0))));
        let running = Arc::new(AtomicBool::new(false));
        let (worker_tx, worker_rx) = mpsc::channel();

        let worker = Worker::new(
            Arc::clone(&store),
            Arc::clone(&snapshot),
            Arc::clone(&running),
            notify,
            worker_rx,
            sort_strategy,
            n_threads,
            initial_chunk_size,
        );
        std::thread::Builder::new()
            .name("matcher-worker".to_string())
            .spawn(move || worker.run())
            .expect("failed to spawn the matcher worker thread");

        Self {
            store,
            snapshot,
            worker_tx,
            running,
            generation: 0,
            count: Arc::new(AtomicUsize::new(0)),
            last_pattern: String::new(),
        }
    }

    /// Get an injector that can be used to push items into the fuzzy matcher.
    ///
    /// This can be used at any time to push items into the fuzzy matcher.
    ///
    /// # Example
    /// ```
    /// use television::matcher::{Matcher, SortStrategy};
    ///
    /// let matcher: Matcher<()> = Matcher::new(SortStrategy::Score, 2);
    /// let injector = matcher.injector();
    ///
    /// injector.push_batch(vec![
    ///     ((), "some string to match against".to_string()),
    ///     ((), "some other string".to_string()),
    /// ]);
    /// ```
    pub fn injector(&self) -> Injector<I> {
        Injector::new(
            self.worker_tx.clone(),
            Arc::clone(&self.running),
            self.generation,
            Arc::clone(&self.count),
        )
    }

    /// Find items that match the given pattern.
    ///
    /// This should be called whenever the pattern changes. It only notifies
    /// the background worker and returns immediately. Results become available
    /// when `notify` is called or after awaiting `wait_for_idle`.
    pub fn find(&mut self, pattern: &str) {
        if pattern == self.last_pattern {
            return;
        }
        self.last_pattern = pattern.to_string();
        self.running.store(true, Ordering::Relaxed);
        let _ = self.worker_tx.send(WorkerMsg::Pattern(pattern.to_string()));
    }

    /// Get the matched items.
    ///
    /// This reads the latest snapshot published by the background worker.
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
    /// use television::matcher::{Matcher, SortStrategy};
    ///
    /// let mut matcher: Matcher<String> = Matcher::new(SortStrategy::Score, 2);
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
    #[allow(clippy::cast_possible_truncation)]
    pub fn results(
        &mut self,
        num_entries: u32,
        offset: u32,
    ) -> Vec<matched_item::MatchedItem<I>> {
        let snapshot = self.snapshot.lock().clone();
        let mut indices_matcher = frizbee::Matcher::from_query(
            &snapshot.pattern,
            &frizbee::Config::default().casing(frizbee::CaseMatching::Smart),
        );

        // Discard snapshots computed against a previous store (i.e. published
        // by the worker right before a restart)
        if snapshot.generation != self.generation {
            return Vec::new();
        }

        let match_count = snapshot.matches.len() as u32;
        // If the offset is greater than the number of matched items, return an empty Vec
        if offset >= match_count {
            return Vec::new();
        }
        // Limit to available entries
        let num_entries = num_entries.min(match_count - offset);

        // Clone the store handle so the read guard borrows a local instead of
        // `self` (the indices matcher needs `&mut self` below)
        let store = Arc::clone(&self.store);
        // NOTE: `read_recursive` so reads never queue behind a writer that's
        // waiting on the worker's long-held read lock during a matcher pass
        let store = store.read_recursive();

        // PERF: Pre-allocate the results Vec so we avoid repeated reallocations
        let mut results = Vec::with_capacity(num_entries as usize);
        for i in offset..offset + num_entries {
            if let Some(m) = snapshot.matches.get(i) {
                results.push(Self::matched_item(
                    &store,
                    &mut indices_matcher,
                    m.index,
                ));
            }
        }

        results
    }

    /// Get a single matched item.
    ///
    /// # Example
    /// ```ignore
    /// use television::matcher::{Matcher, SortStrategy};
    ///
    /// let mut matcher: Matcher<String> = Matcher::new(SortStrategy::Score, 2);
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
        let snapshot = self.snapshot.lock().clone();
        if snapshot.generation != self.generation {
            return None;
        }
        let m = snapshot.matches.get(index)?;
        let mut indices_matcher = frizbee::Matcher::from_query(
            &snapshot.pattern,
            &frizbee::Config::default().casing(frizbee::CaseMatching::Smart),
        );

        let store = Arc::clone(&self.store);
        let store = store.read_recursive();
        Some(Self::matched_item(&store, &mut indices_matcher, m.index))
    }

    /// The number of items matching the current pattern.
    #[allow(clippy::cast_possible_truncation)]
    pub fn matched_item_count(&self) -> u32 {
        let snapshot = self.snapshot.lock();
        if snapshot.generation == self.generation {
            snapshot.matches.len() as u32
        } else {
            0
        }
    }

    /// The total number of items that have been pushed into the matcher,
    /// including batches still in flight to the worker.
    #[allow(clippy::cast_possible_truncation)]
    pub fn total_item_count(&self) -> u32 {
        self.count.load(Ordering::Relaxed) as u32
    }

    /// Whether the background worker is currently matching or has pending
    /// work.
    pub fn running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Restart the matcher.
    ///
    /// This will reset the matcher to its initial state, clearing all items
    /// and matches (the current pattern is preserved).
    ///
    /// In-flight injectors keep sending batches tagged with the previous
    /// generation, which the worker silently discards; call `injector` again
    /// to get an injector for the fresh store.
    pub fn restart(&mut self) {
        self.generation += 1;
        self.store = Arc::new(RwLock::new(Store::default()));
        self.count = Arc::new(AtomicUsize::new(0));
        // Clear the published snapshot right away so stale results don't
        // linger while the worker processes the restart
        *self.snapshot.lock() = Arc::new(Snapshot::empty(self.generation));
        self.running.store(true, Ordering::Relaxed);
        let _ = self.worker_tx.send(WorkerMsg::Restart {
            store: Arc::clone(&self.store),
            generation: self.generation,
        });
    }

    /// Block until the background worker has processed all previously sent
    /// messages and finished the resulting matcher pass.
    pub fn wait_for_idle(&self) {
        let (ack_tx, ack_rx) = mpsc::channel();
        if self.worker_tx.send(WorkerMsg::WaitForIdle(ack_tx)).is_ok() {
            let _ = ack_rx.recv();
        }
    }

    pub fn wait_for_idle_timeout(&self, timeout: Duration) {
        let (ack_tx, ack_rx) = mpsc::channel();
        if self.worker_tx.send(WorkerMsg::WaitForIdle(ack_tx)).is_ok() {
            let _ = ack_rx.recv_timeout(timeout);
        }
    }

    /// Assemble a `MatchedItem` for the store entry at `index`, computing the
    /// matched character indices on the fly
    fn matched_item(
        store: &Store<I>,
        indices_matcher: &mut frizbee::Matcher,
        index: u32,
    ) -> MatchedItem<I> {
        let haystack = &store.haystacks[index as usize];

        let mut match_indices = indices_matcher
            .match_one_indices(haystack, index)
            .map(|m| m.indices)
            .unwrap_or_default();
        // Frizbee returns UTF-8 byte offsets in reverse order
        match_indices.reverse();

        MatchedItem::new(
            store.items[index as usize].clone(),
            haystack.clone(),
            // Convert UTF-8 byte offsets to UTF-32 character indices
            byte_indices_to_char_indices(haystack, match_indices),
        )
    }
}

/// Get the number of threads to use for the matcher.
///
/// This uses the number of available threads on the system, minus 3, to avoid
/// saturating the system.
///
/// The number is capped to 32 threads to avoid impacting startup time and memory usage.
///
/// Defaults to 4 if the number of available threads cannot be determined.
pub fn matcher_threads() -> usize {
    available_parallelism()
        .map(|n| n.get().saturating_sub(3).clamp(1, 32))
        .unwrap_or(4)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect_ids(matcher: &mut Matcher<usize>) -> Vec<usize> {
        let count = matcher.matched_item_count();
        matcher.results(count, 0).iter().map(|m| m.inner).collect()
    }

    /// Incremental passes (items pushed across several batches) must produce
    /// the same results, in the same order, as matching everything in one go.
    #[test]
    fn incremental_matching_equals_full_rematch() {
        let items: Vec<(usize, String)> = (0..300)
            .map(|i| {
                let haystack = match i % 3 {
                    0 => format!("abc_{i}"),
                    1 => format!("xx_abc_{i}"),
                    _ => format!("a{i}b{i}c{i}"),
                };
                (i, haystack)
            })
            .collect();

        let mut incremental: Matcher<usize> =
            Matcher::new(SortStrategy::Score, 2);
        let injector = incremental.injector();
        incremental.find("abc");
        for chunk in items.chunks(30) {
            injector.push_batch(chunk.to_vec());
            incremental.wait_for_idle();
        }

        let mut full: Matcher<usize> = Matcher::new(SortStrategy::Score, 2);
        full.injector().push_batch(items);
        full.find("abc");
        full.wait_for_idle();

        let full_ids = collect_ids(&mut full);
        assert!(!full_ids.is_empty());
        assert_eq!(collect_ids(&mut incremental), full_ids);
    }

    /// A chunked pass (small chunks over a larger store) must produce the
    /// same results, in the same order, as matching everything in one chunk.
    #[test]
    fn chunked_matching_equals_single_chunk() {
        let items: Vec<(usize, String)> = (0..500)
            .map(|i| {
                let haystack = match i % 3 {
                    0 => format!("abc_{i}"),
                    1 => format!("xx_abc_{i}"),
                    _ => format!("a{i}b{i}c{i}"),
                };
                (i, haystack)
            })
            .collect();

        let mut chunked: Matcher<usize> =
            Matcher::with_chunk_size(SortStrategy::Score, 2, 16);
        chunked.injector().push_batch(items.clone());
        chunked.find("abc");
        chunked.wait_for_idle();

        let mut single: Matcher<usize> = Matcher::new(SortStrategy::Score, 2);
        single.injector().push_batch(items);
        single.find("abc");
        single.wait_for_idle();

        let single_ids = collect_ids(&mut single);
        assert!(!single_ids.is_empty());
        assert_eq!(collect_ids(&mut chunked), single_ids);
    }

    /// With no pattern, items pushed across several batches must come out in
    /// insertion order.
    #[test]
    fn incremental_matching_preserves_index_order_on_empty_pattern() {
        let mut matcher: Matcher<usize> = Matcher::new(SortStrategy::Score, 2);
        let injector = matcher.injector();
        for chunk in (0..15).collect::<Vec<_>>().chunks(5) {
            injector.push_batch(
                chunk.iter().map(|&i| (i, format!("item_{i}"))).collect(),
            );
            matcher.wait_for_idle();
        }

        assert_eq!(collect_ids(&mut matcher), (0..15).collect::<Vec<_>>());
    }
}
