use super::{MatcherConfig, Notify, PromoteTable, SortStrategy};
use frizbee::Match;
use parking_lot::{Mutex, RwLock};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

pub(super) const INITIAL_CHUNK_SIZE: usize = 512 * 1024;

/// This caps how long a pass can run without publishing results or noticing new messages (pattern
/// changes, new items).
const MAX_CHUNK_SIZE: usize = 8 * 1024 * 1024;

/// The items and corresponding haystacks that have been pushed into the matcher so far.
///
/// The two are kept separate so that the haystacks can be passed as a contiguous slice to the
/// matcher.
///
/// This is shared between the background worker (writes new items and matches against the
/// haystacks) and the [`super::Matcher`] handle (which reads item data when assembling results).
///
/// The store is append-only.
pub(super) struct Store<I> {
    /// Bumped on every restart (see [`super::Matcher::restart`]) so that snapshots and injector
    /// batches computed for a previous store can be detected and discarded.
    pub(super) generation: u64,
    pub(super) items: Vec<I>,
    pub(super) haystacks: Vec<Box<str>>,
}

impl<I> Store<I> {
    pub(super) fn new(generation: u64) -> Self {
        Self {
            generation,
            items: Vec::new(),
            haystacks: Vec::new(),
        }
    }
}

/// The matches published in a [`Snapshot`].
pub(super) enum Matches {
    /// Every item in store, in the order they were ingested (and how many).
    All(u32),
    /// Same as [`Matches::All`], but with promoted entries materialized and sorted at the front of
    /// the list.
    AllWithPromoted {
        /// Total number of matched items, promoted entries included.
        total: u32,
        /// The promoted matches, in display order.
        promoted: Vec<Match>,
        /// The promoted store indices in ascending order, used to skip over
        /// promoted entries when indexing into the implicit remainder.
        by_index: Vec<u32>,
    },
    /// The matched items, ordered according to the sort strategy. The first
    /// `promoted` entries are the promoted prefix (see [`SortStrategy::Promoted`]).
    Sorted { matches: Vec<Match>, promoted: u32 },
}

impl Matches {
    pub(super) fn len(&self) -> usize {
        match self {
            Matches::All(count) => *count as usize,
            Matches::AllWithPromoted { total, .. } => *total as usize,
            Matches::Sorted { matches, .. } => matches.len(),
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub(super) fn get(&self, index: u32) -> Option<Match> {
        match self {
            Matches::All(count) => {
                (index < *count).then(|| Match::from_index(index as usize))
            }
            Matches::AllWithPromoted {
                total,
                promoted,
                by_index,
            } => {
                if let Some(m) = promoted.get(index as usize) {
                    return Some(*m);
                }
                if index >= *total {
                    return None;
                }
                // The remainder is every store index in order minus the
                // promoted ones: walk the promoted indices to translate the
                // rank into a store index.
                let mut store_index = index - promoted.len() as u32;
                for &promoted_index in by_index {
                    if promoted_index <= store_index {
                        store_index += 1;
                    } else {
                        break;
                    }
                }
                Some(Match::from_index(store_index as usize))
            }
            Matches::Sorted { matches, .. } => {
                matches.get(index as usize).copied()
            }
        }
    }

    /// The materialized matches and the length of their promoted prefix, if
    /// this snapshot holds any.
    fn as_sorted(&self) -> Option<(&[Match], u32)> {
        match self {
            Matches::All(_) | Matches::AllWithPromoted { .. } => None,
            Matches::Sorted { matches, promoted } => {
                Some((matches, *promoted))
            }
        }
    }
}

/// The result of a matcher pass, published by the background worker.
///
/// A long pass over a large store is published incrementally: the snapshot grows chunk by chunk
/// until the whole store has been matched.
pub(super) struct Snapshot {
    /// The store generation this snapshot was computed against (see [`Store::generation`]).
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
            matches: Matches::Sorted {
                matches: Vec::new(),
                promoted: 0,
            },
        }
    }
}

/// Messages sent from the Matcher and its injectors to the Worker.
pub(super) enum WorkerMessage<I> {
    NewPattern(String),
    /// A batch of items pushed through an injector, tagged with the store
    /// generation the injector was created for so that batches in flight
    /// across a restart can be discarded.
    NewItems {
        generation: u64,
        batch: Vec<(I, String)>,
    },
    /// A new store has been created (see [`super::Matcher::restart`]).
    Restart(Arc<RwLock<Store<I>>>),
    /// Wait for the worker to finish its current pass and report idle over the channel.
    WaitForIdle(mpsc::Sender<()>),
}

/// The background worker that owns the inner [`frizbee::Matcher`].
///
/// The worker blocks on its message channel and re-matches the store against the current pattern
/// whenever items are added, the pattern changes, or the matcher is restarted. Pending messages are
/// drained before each pass so that a burst of keystrokes or item batches results in a single pass
/// over the store with the latest state, which also acts as a natural debounce.
///
/// Passes over large stores are chunked: results are published after every chunk and messages
/// arriving mid-pass interrupt it (see [`Worker::match`]).
pub(super) struct Worker<I: Sync + Send + 'static> {
    store: Arc<RwLock<Store<I>>>,
    snapshot: Arc<Mutex<Arc<Snapshot>>>,
    running: Arc<AtomicBool>,
    /// Called after each published snapshot to wake the front-end
    notify: Notify,
    rx: mpsc::Receiver<WorkerMessage<I>>,
    matcher: frizbee::Matcher,
    pattern: String,
    sort_strategy: SortStrategy<I>,
    /// The matching behavior, needed to rebuild the matcher on pattern
    /// changes.
    config: MatcherConfig,
    /// Last item that was matched
    last_match_index: usize,
    /// Number of threads to use when matching.
    n_threads: usize,
    /// Size of the first chunk of a matching pass.
    initial_chunk_size: usize,
    /// Promote table sampled at the start of the current pass.
    promote_table: Option<PromoteTable>,
    /// Promoted matches accumulated by the current pass, tagged with their
    /// promote score and kept in display order.
    promoted_matches: Vec<(u64, Match)>,
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
        rx: mpsc::Receiver<WorkerMessage<I>>,
        sort_strategy: SortStrategy<I>,
        config: MatcherConfig,
        n_threads: usize,
        initial_chunk_size: usize,
    ) -> Self {
        Self {
            store,
            snapshot,
            running,
            notify,
            rx,
            matcher: build_matcher("", config, &sort_strategy),
            pattern: String::new(),
            sort_strategy,
            config,
            last_match_index: 0,
            n_threads,
            initial_chunk_size,
            promote_table: None,
            promoted_matches: Vec::new(),
        }
    }

    pub(super) fn run(mut self) {
        // A message that interrupted a chunked matching pass (see `self.rematch`), to be processed
        // before the pass resumes.
        let mut next_message: Option<WorkerMessage<I>> = None;
        // Whether an interrupted pass still has items left to match.
        let mut pass_pending = false;
        let mut waiters: Vec<mpsc::Sender<()>> = Vec::new();

        loop {
            let message = match next_message.take() {
                Some(msg) => msg,
                None => match self.rx.recv() {
                    Ok(msg) => msg,
                    // The matcher handle and all of its injectors have been dropped, so
                    // the worker can exit.
                    Err(_) => return,
                },
            };
            self.running.store(true, Ordering::Relaxed);

            let mut dirty = self.handle_message(message, &mut waiters);
            // Gather all pending messages into a single matcher pass
            while let Ok(msg) = self.rx.try_recv() {
                dirty |= self.handle_message(msg, &mut waiters);
            }

            if dirty || pass_pending {
                next_message = self.r#match();
                pass_pending = next_message.is_some();
            }

            // Only report idle (and ack waiters) once the pass ran to
            // completion without being interrupted
            if next_message.is_none() {
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
        msg: WorkerMessage<I>,
        waiters: &mut Vec<mpsc::Sender<()>>,
    ) -> bool {
        match msg {
            WorkerMessage::NewPattern(pattern) => {
                if pattern == self.pattern {
                    return false;
                }
                self.matcher =
                    build_matcher(&pattern, self.config, &self.sort_strategy);
                self.pattern = pattern;
                self.last_match_index = 0;
                true
            }
            WorkerMessage::NewItems { generation, batch } => {
                // Batches from injectors created before a restart land here
                // with a stale generation and are dropped along with the
                // store they were destined for
                let mut store = self.store.write();
                if generation != store.generation {
                    return false;
                }
                store.items.reserve(batch.len());
                store.haystacks.reserve(batch.len());
                for (item, haystack) in batch {
                    store.items.push(item);
                    store.haystacks.push(haystack.into_boxed_str());
                }
                true
            }
            WorkerMessage::Restart(store) => {
                self.store = store;
                self.last_match_index = 0;
                true
            }
            WorkerMessage::WaitForIdle(ack) => {
                waiters.push(ack);
                false
            }
        }
    }

    /// Match the store against the current pattern.
    ///
    /// The store is matched progressively in chunks of doubling size (publishing incremental
    /// snapshots), and this will always attempt to resume from `last_match_index` if it got
    /// interrupted by an incoming message (keystroke, new items, ...).
    #[allow(clippy::cast_possible_truncation)]
    fn r#match(&mut self) -> Option<WorkerMessage<I>> {
        let store = Arc::clone(&self.store);
        let store = store.read();
        let total = store.haystacks.len();

        // Sample the promote table once per pass; promoting depends on it, so
        // a new table invalidates any matched progress.
        if let SortStrategy::Promoted { table, .. } = &self.sort_strategy {
            let table = table();
            if self
                .promote_table
                .as_ref()
                .is_none_or(|current| !Arc::ptr_eq(current, &table))
            {
                self.last_match_index = 0;
                self.promote_table = Some(table);
            }
        }

        let empty_pattern =
            self.matcher.patterns().iter().all(|p| p.needle.is_empty());
        // With an empty pattern and nothing to promote, everything matches in store order.
        if empty_pattern
            && self.promote_table.as_ref().is_none_or(|t| t.is_empty())
        {
            self.last_match_index = total;
            self.publish(store.generation, Matches::All(total as u32));
            return None;
        }

        let mut chunk_size = self.initial_chunk_size;
        loop {
            let offset = self.last_match_index;
            if offset == 0 {
                // A fresh pass starts with a clean promoted accumulator
                self.promoted_matches.clear();
            }
            let end = (offset + chunk_size).min(total);

            let matches = if empty_pattern {
                self.find_promoted_in_chunk(&store, offset, end)
            } else {
                self.match_chunk(&store, offset, end)
            };
            self.last_match_index = end;
            self.publish(store.generation, matches);

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

    /// Find the promoted entries in a chunk of the store and merge them with the
    /// previously published promoted entries.
    #[allow(clippy::cast_possible_truncation)]
    fn find_promoted_in_chunk(
        &mut self,
        store: &Store<I>,
        offset: usize,
        end: usize,
    ) -> Matches {
        let SortStrategy::Promoted { key, .. } = &self.sort_strategy else {
            unreachable!("promote_chunk requires a promote table");
        };
        let table = self
            .promote_table
            .as_ref()
            .expect("promote_chunk requires a promote table");

        let mut new_promoted = Vec::new();
        for index in offset..end {
            let promote_key =
                key(&store.items[index], &store.haystacks[index]);
            if let Some(&score) = table.get(promote_key.as_ref()) {
                new_promoted.push((score, Match::from_index(index)));
            }
        }

        self.promoted_matches.append(&mut new_promoted);
        self.promoted_matches
            .sort_by(|a, b| b.0.cmp(&a.0).then(a.1.index.cmp(&b.1.index)));

        let promoted: Vec<Match> =
            self.promoted_matches.iter().map(|(_, m)| *m).collect();
        let mut by_index: Vec<u32> =
            promoted.iter().map(|m| m.index).collect();
        by_index.sort_unstable();
        Matches::AllWithPromoted {
            total: end as u32,
            promoted,
            by_index,
        }
    }

    /// Match a chunk of the store against the current pattern and merge the
    /// result with the previously published matches.
    #[allow(clippy::cast_possible_truncation)]
    fn match_chunk(
        &mut self,
        store: &Store<I>,
        offset: usize,
        end: usize,
    ) -> Matches {
        let mut new_matches = self.matcher.match_list_parallel(
            &store.haystacks[offset..end],
            self.n_threads,
        );

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
            .filter(|prev| prev.generation == store.generation);
        let (prev_matches, prev_num_promoted) = prev
            .as_ref()
            .and_then(|prev| prev.matches.as_sorted())
            .unwrap_or((&[], 0));

        match &self.sort_strategy {
            SortStrategy::Score => Matches::Sorted {
                matches: merge_matches(
                    prev_matches,
                    &new_matches,
                    score_then_index,
                ),
                promoted: 0,
            },
            // Chunk matches are already in store order
            SortStrategy::Index => {
                let mut matches =
                    Vec::with_capacity(prev_matches.len() + new_matches.len());
                matches.extend_from_slice(prev_matches);
                matches.append(&mut new_matches);
                Matches::Sorted {
                    matches,
                    promoted: 0,
                }
            }
            SortStrategy::Promoted { key, .. } => {
                let table = self
                    .promote_table
                    .as_ref()
                    .expect("sampled at the start of the pass");
                // Split this chunk's matches into promoted entries and the
                // rest, preserving their score order.
                let mut rest = Vec::with_capacity(new_matches.len());
                let mut new_promoted = Vec::new();
                if table.is_empty() {
                    rest = new_matches;
                } else {
                    for m in new_matches {
                        let index = m.index as usize;
                        let promote_key =
                            key(&store.items[index], &store.haystacks[index]);
                        // Only promote entries that are exact substring matches
                        match table.get(promote_key.as_ref()) {
                            Some(&score)
                                if is_substring_match(
                                    self.matcher.patterns(),
                                    &store.haystacks[index],
                                ) =>
                            {
                                new_promoted.push((score, m));
                            }
                            _ => rest.push(m),
                        }
                    }
                }
                // The promoted prefix is tiny (bounded by the table size),
                // so a full re-sort per chunk is cheaper than bookkeeping
                self.promoted_matches.append(&mut new_promoted);
                self.promoted_matches.sort_by(|a, b| {
                    b.0.cmp(&a.0)
                        .then(b.1.score.cmp(&a.1.score))
                        .then(a.1.index.cmp(&b.1.index))
                });

                let prev_rest = &prev_matches[prev_num_promoted as usize..];
                let rest = merge_matches(prev_rest, &rest, score_then_index);
                let mut matches = Vec::with_capacity(
                    self.promoted_matches.len() + rest.len(),
                );
                matches.extend(self.promoted_matches.iter().map(|(_, m)| *m));
                matches.extend(rest);
                Matches::Sorted {
                    matches,
                    promoted: self.promoted_matches.len() as u32,
                }
            }
        }
    }

    fn publish(&mut self, generation: u64, matches: Matches) {
        *self.snapshot.lock() = Arc::new(Snapshot {
            generation,
            pattern: self.pattern.clone(),
            matches,
        });
        // Wake the front-end so it can render the results
        (self.notify)();
    }
}

/// Whether every "positive" needle appears contiguously in `haystack`.
fn is_substring_match(patterns: &[frizbee::Pattern], haystack: &str) -> bool {
    patterns.iter().filter(|p| !p.negated).all(|p| {
        let needle = p.needle.as_str();
        if needle.chars().any(char::is_uppercase) {
            haystack.contains(needle)
        } else {
            haystack.to_lowercase().contains(needle)
        }
    })
}

/// Score (desc) then index (asc): the display order of non-promoted matches.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn score_then_index(a: &Match, b: &Match) -> std::cmp::Ordering {
    b.score.cmp(&a.score).then(a.index.cmp(&b.index))
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
    config: MatcherConfig,
    sort_strategy: &SortStrategy<I>,
) -> frizbee::Matcher {
    let sort_strategy = match sort_strategy {
        SortStrategy::Score | SortStrategy::Promoted { .. } => {
            frizbee::SortStrategy::ScoreThenIndexAsc
        }
        SortStrategy::Index => frizbee::SortStrategy::IndexAsc,
    };

    frizbee::Matcher::from_patterns(
        &super::parse_patterns(pattern, config),
        &frizbee::Config::default()
            .sort(sort_strategy)
            .matching(config.matching_mode)
            .casing(frizbee::CaseMatching::Smart),
    )
}
