use super::{Notify, SortStrategy};
use frizbee::Match;
use parking_lot::{Mutex, RwLock};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

/// The items and haystacks that have been pushed into the matcher so far.
///
/// This is shared between the background worker (the sole writer, which
/// appends batches received from injectors and matches against the
/// haystacks) and the [`super::Matcher`] handle (which reads item data when
/// assembling results). Injectors never touch the store directly: they send
/// batches over the worker channel so that pushing items never blocks on a
/// matching pass.
///
/// The store is append-only: [`super::Matcher::restart`] swaps it for a fresh
/// one instead of clearing it, and batches sent by injectors created before
/// the restart are discarded by the worker (see [`WorkerMsg::Items`])
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

/// Messages sent from the Matcher and its injectors to the Worker.
pub(super) enum WorkerMsg<I> {
    Pattern(String),
    /// A batch of items pushed through an injector, tagged with the store
    /// generation the injector was created for so that batches in flight
    /// across a restart can be discarded.
    Items {
        generation: u64,
        batch: Vec<(I, String)>,
    },
    Restart {
        store: Arc<RwLock<Store<I>>>,
        generation: u64,
    },
    WaitForIdle(mpsc::Sender<()>),
}

/// The background worker that owns the inner [`frizbee::Matcher`].
///
/// The worker blocks on its message channel and re-matches the store against
/// the current pattern whenever items are added, the pattern changes, or the
/// matcher is restarted. Pending messages are drained before each pass so
/// that a burst of keystrokes or item batches results in a single pass over
/// the store with the latest state, which also acts as a natural debounce.
pub(super) struct Worker<I: Sync + Send + 'static> {
    store: Arc<RwLock<Store<I>>>,
    snapshot: Arc<Mutex<Arc<Snapshot>>>,
    running: Arc<AtomicBool>,
    /// Called after each published snapshot to wake the front-end
    notify: Notify,
    rx: mpsc::Receiver<WorkerMsg<I>>,
    matcher: frizbee::Matcher,
    pattern: String,
    sort_strategy: SortStrategy<I>,
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
        sort_strategy: SortStrategy<I>,
        n_threads: usize,
    ) -> Self {
        Self {
            store,
            snapshot,
            running,
            notify,
            rx,
            matcher: build_matcher("", &sort_strategy),
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
                self.matcher = build_matcher(&pattern, &self.sort_strategy);
                self.pattern = pattern;
                self.last_match_index = 0;
                true
            }
            WorkerMsg::Items { generation, batch } => {
                // Batches from injectors created before a restart land here
                // with a stale generation and are dropped along with the
                // store they were destined for
                if generation != self.generation {
                    return false;
                }
                let mut store = self.store.write();
                store.items.reserve(batch.len());
                store.haystacks.reserve(batch.len());
                for (item, haystack) in batch {
                    store.items.push(item);
                    store.haystacks.push(haystack);
                }
                true
            }
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
    ///
    /// Incremental passes (new items, same pattern) only match the store's
    /// tail and merge the new matches with the previously published ones:
    /// both runs are already in display order, so a linear merge replaces a
    /// full re-sort.
    #[allow(clippy::cast_possible_truncation)]
    fn rematch(&mut self) {
        let store = self.store.read();
        let offset = self.last_match_index;
        let is_incremental = offset != 0;
        let mut new_matches = self
            .matcher
            .match_list_parallel(&store.haystacks[offset..], self.n_threads);
        self.last_match_index = store.haystacks.len();
        drop(store);

        // Matches on the tail slice are indexed relative to it
        if is_incremental {
            for m in &mut new_matches {
                m.index += offset as u32;
            }
        }

        // The previous snapshot's matches are merged with (not mutated by)
        // incremental passes since the snapshot is shared with the front-end
        let prev = is_incremental
            .then(|| Arc::clone(&self.snapshot.lock()))
            .filter(|prev| prev.generation == self.generation);

        let matches = match &self.sort_strategy {
            SortStrategy::Score => match prev {
                Some(prev) => {
                    merge_matches(&prev.matches, &new_matches, |a, b| {
                        b.score.cmp(&a.score).then(a.index.cmp(&b.index))
                    })
                }
                None => new_matches,
            },
            SortStrategy::Index => match prev {
                // Tail indices are all greater than previous ones
                Some(prev) => {
                    let mut matches = Vec::with_capacity(
                        prev.matches.len() + new_matches.len(),
                    );
                    matches.extend_from_slice(&prev.matches);
                    matches.extend_from_slice(&new_matches);
                    matches
                }
                None => new_matches,
            },
            SortStrategy::Custom(sort_fn) => {
                let store = self.store.read();
                let cmp = |a: &Match, b: &Match| {
                    sort_fn(
                        a,
                        &store.items[a.index as usize],
                        &store.haystacks[a.index as usize],
                        b,
                        &store.items[b.index as usize],
                        &store.haystacks[b.index as usize],
                    )
                };
                new_matches.sort_by(cmp);
                match prev {
                    Some(prev) => {
                        merge_matches(&prev.matches, &new_matches, cmp)
                    }
                    None => new_matches,
                }
            }
        };

        *self.snapshot.lock() = Arc::new(Snapshot {
            generation: self.generation,
            pattern: self.pattern.clone(),
            matches,
        });
        // Wake the front-end so it can render the results
        (self.notify)();
    }
}

/// Merge two runs sorted by `cmp` into a freshly allocated `Vec`, keeping
/// `prev` elements first on ties.
fn merge_matches(
    prev: &[Match],
    new: &[Match],
    cmp: impl Fn(&Match, &Match) -> std::cmp::Ordering,
) -> Vec<Match> {
    let mut merged = Vec::with_capacity(prev.len() + new.len());
    let (mut i, mut j) = (0, 0);
    while i < prev.len() && j < new.len() {
        if cmp(&prev[i], &new[j]).is_le() {
            merged.push(prev[i]);
            i += 1;
        } else {
            merged.push(new[j]);
            j += 1;
        }
    }
    merged.extend_from_slice(&prev[i..]);
    merged.extend_from_slice(&new[j..]);
    merged
}

fn build_matcher<I: Sync + Send + 'static>(
    pattern: &str,
    sort_strategy: &SortStrategy<I>,
) -> frizbee::Matcher {
    let sort_strategy = match sort_strategy {
        SortStrategy::Score => frizbee::SortStrategy::ScoreThenIndexAsc,
        SortStrategy::Index | SortStrategy::Custom(_) => {
            frizbee::SortStrategy::IndexAsc
        }
    };

    frizbee::Matcher::from_query(
        pattern,
        &frizbee::Config::default()
            .sort(sort_strategy)
            .casing(frizbee::CaseMatching::Smart),
    )
}
