use injector::Injector;
use matched_item::MatchedItem;
use parking_lot::{Mutex, RwLock};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::available_parallelism,
    time::Duration,
};

pub mod injector;
pub mod matched_item;
mod worker;

use worker::{Snapshot, Store, Worker, WorkerMsg};

pub use frizbee::SortStrategy;

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
///
/// TODO: support custom sort strategies (for frecency)
/// TODO: support cancellation on new needle (try_* in frizbee)
/// TODO: investigate "incorrect" indices for `matcher::new(Snapshot::)` on `tv text`
#[allow(clippy::struct_field_names)]
pub struct Matcher<I>
where
    I: Sync + Send + Clone + 'static,
{
    /// The store shared with the injectors and the background worker.
    store: Arc<RwLock<Store<I>>>,
    /// The latest snapshot published by the background worker.
    snapshot: Arc<Mutex<Arc<Snapshot>>>,
    /// Channel used to notify the background worker of changes.
    worker_tx: mpsc::Sender<WorkerMsg<I>>,
    /// Whether the background worker is currently matching or has pending
    /// work.
    running: Arc<AtomicBool>,
    /// The sort strategy items are ordered by.
    sort_strategy: SortStrategy,
    /// Bumped on every restart so that snapshots computed against a previous
    /// store can be detected and discarded.
    generation: u64,
    /// The last pattern passed to `find`, used to avoid notifying the worker
    /// when the pattern hasn't changed.
    last_pattern: String,
}

impl<I> Matcher<I>
where
    I: Sync + Send + Clone + 'static,
{
    /// Create a new fuzzy matcher with the given sort strategy and number of
    /// threads, spawning the background worker thread.
    ///
    /// The matcher won't notify anyone when results change; use
    /// [`Matcher::with_notify`] to be woken as soon as fresh results are
    /// available.
    pub fn new(sort_strategy: SortStrategy, n_threads: usize) -> Self {
        Self::with_notify(sort_strategy, n_threads, Arc::new(|| {}))
    }

    /// Create a new fuzzy matcher that calls `notify` every time the background
    /// worker publishes fresh results.
    ///
    /// The callback runs on the worker thread and should be cheap (e.g. sending
    /// on a channel to wake the UI). It lets the front-end render as soon as
    /// results are available instead of polling on a timer.
    pub fn with_notify(
        sort_strategy: SortStrategy,
        n_threads: usize,
        notify: Notify,
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
            sort_strategy,
            generation: 0,
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
            Arc::clone(&self.store),
            self.worker_tx.clone(),
            Arc::clone(&self.running),
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
        let mut indices_matcher =
            build_matcher(&snapshot.pattern, self.sort_strategy);

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
        for m in snapshot
            .matches
            .iter()
            .skip(offset as usize)
            .take(num_entries as usize)
        {
            results.push(Self::matched_item(
                &store,
                &mut indices_matcher,
                m.index,
            ));
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
        let m = snapshot.matches.get(index as usize)?;
        let mut indices_matcher =
            build_matcher(&snapshot.pattern, self.sort_strategy);

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

    /// The total number of items that have been pushed into the matcher.
    #[allow(clippy::cast_possible_truncation)]
    pub fn total_item_count(&self) -> u32 {
        self.store.read_recursive().items.len() as u32
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
    /// In-flight injectors keep writing to the old store, which is silently
    /// dropped once they're done with it; call `injector` again to get an
    /// injector for the fresh store.
    pub fn restart(&mut self) {
        self.generation += 1;
        self.store = Arc::new(RwLock::new(Store::default()));
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
        // frizbee returns UTF-8 byte offsets in reverse order, while the UI
        // renders against `.chars()` indices.
        match_indices.reverse();

        MatchedItem::new(
            store.items[index as usize].clone(),
            haystack.clone(),
            match_indices,
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

fn build_matcher(
    pattern: &str,
    sort_strategy: SortStrategy,
) -> frizbee::Matcher {
    frizbee::Matcher::from_query(
        pattern,
        &frizbee::Config::default().sort(sort_strategy),
    )
}
