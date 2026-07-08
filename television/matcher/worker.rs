use super::{Notify, SortStrategy, build_matcher};
use parking_lot::{Mutex, RwLock};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

/// The items and haystacks that have been pushed into the matcher so far.
///
/// This is shared between the injectors (writers), the background worker
/// (which matches against the haystacks), and the [`super::Matcher`] handle
/// (which reads item data when assembling results).
///
/// The store is append-only: [`super::Matcher::restart`] swaps it for a fresh
/// one instead of clearing it, which conveniently invalidates any outstanding
/// injectors (their pushes land in the old, orphaned store).
pub(super) struct Store<I> {
    /// Items that have been added to the matcher.
    pub(super) items: Vec<I>,
    /// The strings the items are matched against.
    pub(super) haystacks: Vec<String>,
}

impl<I> Default for Store<I> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            haystacks: Vec::new(),
        }
    }
}

/// The result of a matcher pass, published by the background worker.
///
/// [`super::Matcher::results`] and friends read the latest snapshot instead of
/// waiting on the worker, so the UI thread never blocks on matching.
pub(super) struct Snapshot {
    /// The store generation this snapshot was computed against (see
    /// [`super::Matcher::restart`]).
    pub(super) generation: u64,
    /// The raw pattern the matches were computed with.
    pub(super) pattern: String,
    /// The matched items, ordered according to the sort strategy.
    pub(super) matches: Vec<frizbee::Match>,
}

impl Snapshot {
    pub(super) fn empty(generation: u64) -> Self {
        Self {
            generation,
            pattern: String::new(),
            matches: Vec::new(),
        }
    }
}

/// Messages sent from the Matcher to the Worker.
pub(super) enum WorkerMsg<I> {
    Pattern(String),
    ItemsAdded,
    Restart {
        store: Arc<RwLock<Store<I>>>,
        generation: u64,
    },
    WaitForIdle(mpsc::Sender<()>),
}

/// The background worker that owns the inner frizbee matcher.
///
/// The worker blocks on its message channel and re-matches the store against
/// the current pattern whenever items are added, the pattern changes, or the
/// matcher is restarted. Pending messages are drained before each pass so
/// that a burst of keystrokes or item batches results in a single pass over
/// the store with the latest state, which also acts as a natural debounce.
pub(super) struct Worker<I> {
    store: Arc<RwLock<Store<I>>>,
    snapshot: Arc<Mutex<Arc<Snapshot>>>,
    running: Arc<AtomicBool>,
    /// Called after each published snapshot to wake the front-end
    notify: Notify,
    rx: mpsc::Receiver<WorkerMsg<I>>,
    matcher: frizbee::Matcher,
    pattern: String,
    sort_strategy: SortStrategy,
    /// The generation of the store currently being matched against.
    generation: u64,
    /// Last item that was matched
    last_match_index: usize,
    /// Number of threads to use when matching.
    n_threads: usize,
}

impl<I> Worker<I>
where
    I: Sync + Send + 'static,
{
    pub(super) fn new(
        store: Arc<RwLock<Store<I>>>,
        snapshot: Arc<Mutex<Arc<Snapshot>>>,
        running: Arc<AtomicBool>,
        notify: Notify,
        rx: mpsc::Receiver<WorkerMsg<I>>,
        sort_strategy: SortStrategy,
        n_threads: usize,
    ) -> Self {
        Self {
            store,
            snapshot,
            running,
            notify,
            rx,
            matcher: build_matcher("", sort_strategy),
            pattern: String::new(),
            sort_strategy,
            generation: 0,
            last_match_index: 0,
            n_threads,
        }
    }

    pub(super) fn run(mut self) {
        // Exits once the matcher handle and all of its injectors are dropped
        while let Ok(msg) = self.rx.recv() {
            self.running.store(true, Ordering::Relaxed);

            let mut waiters = Vec::new();
            let mut dirty = self.handle_message(msg, &mut waiters);
            // Gather all pending messages into a single matcher pass
            while let Ok(msg) = self.rx.try_recv() {
                dirty |= self.handle_message(msg, &mut waiters);
            }

            if dirty {
                self.rematch();
            }
            self.running.store(false, Ordering::Relaxed);

            for waiter in waiters {
                let _ = waiter.send(());
            }
        }
    }

    /// Apply a message to the worker state, returning whether a new matcher
    /// pass is needed.
    fn handle_message(
        &mut self,
        msg: WorkerMsg<I>,
        waiters: &mut Vec<mpsc::Sender<()>>,
    ) -> bool {
        match msg {
            WorkerMsg::Pattern(pattern) => {
                if pattern == self.pattern {
                    return false;
                }
                self.matcher = build_matcher(&pattern, self.sort_strategy);
                self.pattern = pattern;
                self.last_match_index = 0;
                true
            }
            WorkerMsg::ItemsAdded => true,
            WorkerMsg::Restart { store, generation } => {
                self.store = store;
                self.generation = generation;
                self.last_match_index = 0;
                true
            }
            WorkerMsg::WaitForIdle(ack) => {
                waiters.push(ack);
                false
            }
        }
    }

    /// Match the store against the current pattern and publish the resulting
    /// snapshot.
    fn rematch(&mut self) {
        let store = self.store.read();
        let is_incremental = self.last_match_index != 0;
        let mut matches = self.matcher.match_list_parallel(
            &store.haystacks[self.last_match_index..],
            self.n_threads,
        );
        self.last_match_index = store.haystacks.len();
        drop(store);

        // If we're incrementally matching, append the new matches to the
        // existing ones and sort them
        if is_incremental {
            let mut existing_matches = self.snapshot.lock().matches.clone();
            existing_matches.extend(matches);
            matches = existing_matches;
            if self.sort_strategy == SortStrategy::Score {
                frizbee::radix_sort_matches(&mut matches);
            }
        }

        *self.snapshot.lock() = Arc::new(Snapshot {
            generation: self.generation,
            pattern: self.pattern.clone(),
            matches,
        });
        // Wake the front-end so it can render the fresh results immediately
        // instead of waiting for the next periodic render tick
        (self.notify)();
    }
}
