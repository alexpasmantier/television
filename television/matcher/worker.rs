use super::{Notify, SortStrategy};
use frizbee::Match;
use parking_lot::{Mutex, RwLock};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

/// Number of items matched by the first chunk of a matching pass: small, so
/// that first results reach the screen quickly on large stores.
pub(super) const INITIAL_CHUNK_SIZE: usize = 512 * 1024;

/// Chunks double in size up to this cap, which bounds how long a pass can run
/// without publishing results or noticing new messages (pattern changes, new
/// items).
const MAX_CHUNK_SIZE: usize = 8 * 1024 * 1024;

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

/// The matches published in a [`Snapshot`].
pub(super) enum Matches {
    /// Every store item matches, in store order (empty pattern). Kept
    /// implicit: with millions of items, materializing (and re-merging) the
    /// full match list on every pushed batch gets expensive.
    All(u32),
    /// The matched items, ordered according to the sort strategy.
    Sorted(Vec<Match>),
}

impl Matches {
    pub(super) fn len(&self) -> usize {
        match self {
            Matches::All(count) => *count as usize,
            Matches::Sorted(matches) => matches.len(),
        }
    }

    pub(super) fn get(&self, index: u32) -> Option<Match> {
        match self {
            Matches::All(count) => {
                (index < *count).then(|| Match::from_index(index as usize))
            }
            Matches::Sorted(matches) => matches.get(index as usize).copied(),
        }
    }

    fn as_sorted(&self) -> Option<&[Match]> {
        match self {
            Matches::All(_) => None,
            Matches::Sorted(matches) => Some(matches),
        }
    }
}

/// The result of a matcher pass, published by the background worker.
///
/// A long pass over a large store is published incrementally: the snapshot
/// grows chunk by chunk until the whole store has been matched.
pub(super) struct Snapshot {
    /// The store generation this snapshot was computed against (see
    /// [`super::Matcher::restart`]).
    pub(super) generation: u64,
    /// The raw pattern the matches were computed with.
    pub(super) pattern: String,
    /// The matched items, ordered according to the sort strategy.
    pub(super) matches: Matches,
}

impl Snapshot {
    pub(super) fn empty(generation: u64) -> Self {
        Self {
            generation,
            pattern: String::new(),
            matches: Matches::Sorted(Vec::new()),
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
///
/// Passes over large stores are chunked: results are published after every
/// chunk and messages arriving mid-pass interrupt it (see
/// [`Worker::rematch`]), so a keystroke never waits on a full pass over
/// millions of items.
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
    /// Size of the first chunk of a matching pass.
    initial_chunk_size: usize,
}

impl<I> Worker<I>
where
    I: Sync + Send + 'static,
{
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        store: Arc<RwLock<Store<I>>>,
        snapshot: Arc<Mutex<Arc<Snapshot>>>,
        running: Arc<AtomicBool>,
        notify: Notify,
        rx: mpsc::Receiver<WorkerMsg<I>>,
        sort_strategy: SortStrategy<I>,
        n_threads: usize,
        initial_chunk_size: usize,
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
            initial_chunk_size,
        }
    }

    pub(super) fn run(mut self) {
        // A message that interrupted a matching pass, to be processed before
        // the pass resumes.
        let mut next_msg: Option<WorkerMsg<I>> = None;
        // Whether an interrupted pass still has items left to match.
        let mut pass_pending = false;
        let mut waiters: Vec<mpsc::Sender<()>> = Vec::new();

        loop {
            let msg = match next_msg.take() {
                Some(msg) => msg,
                // Exits once the matcher handle and all of its injectors
                // are dropped
                None => match self.rx.recv() {
                    Ok(msg) => msg,
                    Err(_) => return,
                },
            };
            self.running.store(true, Ordering::Relaxed);

            let mut dirty = self.handle_message(msg, &mut waiters);
            // Gather all pending messages into a single matcher pass
            while let Ok(msg) = self.rx.try_recv() {
                dirty |= self.handle_message(msg, &mut waiters);
            }

            if dirty || pass_pending {
                next_msg = self.rematch();
                pass_pending = next_msg.is_some();
            }

            // Only report idle (and ack waiters) once the pass ran to
            // completion without being interrupted
            if next_msg.is_none() {
                self.running.store(false, Ordering::Relaxed);
                for waiter in waiters.drain(..) {
                    let _ = waiter.send(());
                }
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

    /// Match the store against the current pattern, publishing the results
    /// progressively.
    ///
    /// The store is matched in chunks of doubling size, and a snapshot is
    /// published after every chunk so that first results reach the screen
    /// quickly on large stores. Between chunks the worker checks its message
    /// channel: an incoming message (keystroke, new items, ...) interrupts
    /// the pass and is returned to [`Worker::run`], which processes it and
    /// re-enters this function. A pass with an unchanged pattern resumes from
    /// `last_match_index`, so already-matched items are never re-matched:
    /// each chunk's matches are merged with the previously published ones
    /// (both runs are already in display order, so a linear merge replaces a
    /// full re-sort).
    #[allow(clippy::cast_possible_truncation)]
    fn rematch(&mut self) -> Option<WorkerMsg<I>> {
        // Clone the store handle so the read guard borrows a local instead
        // of `self` (publishing needs `&mut self` below). The worker is the
        // only writer, so holding the read lock across the pass blocks no
        // one: front-end reads use `read_recursive`.
        let store = Arc::clone(&self.store);
        let store = store.read();
        let total = store.haystacks.len();

        // With an empty pattern everything matches in store order: keep the
        // match list implicit instead of materializing (and re-merging)
        // millions of `Match`es on every pushed batch. Custom sorting still
        // needs the materialized path below.
        if !matches!(self.sort_strategy, SortStrategy::Custom(_))
            && self.matcher.patterns().iter().all(|p| p.needle.is_empty())
        {
            self.last_match_index = total;
            self.publish(Matches::All(total as u32));
            return None;
        }

        let mut chunk_size = self.initial_chunk_size;
        loop {
            let offset = self.last_match_index;
            let end = (offset + chunk_size).min(total);
            let mut new_matches = self.matcher.match_list_parallel(
                &store.haystacks[offset..end],
                self.n_threads,
            );
            self.last_match_index = end;

            // Matches on a chunk are indexed relative to it
            if offset != 0 {
                for m in &mut new_matches {
                    m.index += offset as u32;
                }
            }

            // The previously published matches are merged with (not mutated
            // by) this chunk's since the snapshot is shared with the
            // front-end. A pattern change resets `last_match_index`, so a
            // snapshot from a previous pattern is never merged with (offset
            // is 0), and neither is one from a previous store generation.
            let prev = (offset != 0)
                .then(|| Arc::clone(&self.snapshot.lock()))
                .filter(|prev| prev.generation == self.generation);
            let prev_matches = prev
                .as_ref()
                .and_then(|prev| prev.matches.as_sorted())
                .unwrap_or(&[]);

            let matches = match &self.sort_strategy {
                SortStrategy::Score => {
                    merge_matches(prev_matches, &new_matches, |a, b| {
                        b.score.cmp(&a.score).then(a.index.cmp(&b.index))
                    })
                }
                // Chunk indices are all greater than previous ones
                SortStrategy::Index => {
                    let mut matches = Vec::with_capacity(
                        prev_matches.len() + new_matches.len(),
                    );
                    matches.extend_from_slice(prev_matches);
                    matches.append(&mut new_matches);
                    matches
                }
                SortStrategy::Custom(sort_fn) => {
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
                    merge_matches(prev_matches, &new_matches, cmp)
                }
            };
            self.publish(Matches::Sorted(matches));

            if self.last_match_index >= total {
                return None;
            }
            // Interrupt the pass as soon as a new message arrives
            if let Ok(msg) = self.rx.try_recv() {
                return Some(msg);
            }
            chunk_size = (chunk_size * 2).min(MAX_CHUNK_SIZE);
        }
    }

    fn publish(&mut self, matches: Matches) {
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
