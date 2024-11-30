use std::num::NonZero;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::available_parallelism;
use tracing::debug;

use crossbeam_channel::{unbounded, Receiver, Sender};
use frizbee::{match_list, match_list_for_matched_indices, Options};
use parking_lot::Mutex;
use rayon::prelude::ParallelSliceMut;
use threadpool::ThreadPool;

pub struct Injector<I>
where
    I: Sync + Send + Clone + 'static,
{
    injection_channel_tx: Sender<I>,
}

impl<I> Injector<I>
where
    I: Sync + Send + Clone + 'static,
{
    pub fn clone(&self) -> Injector<I> {
        Injector {
            injection_channel_tx: self.injection_channel_tx.clone(),
        }
    }
}

impl<I> Injector<I>
where
    I: Sync + Send + Clone + 'static,
{
    pub fn push(&self, item: I) {
        self.injection_channel_tx.send(item).unwrap();
    }
}

struct WorkerPool {
    thread_pool: ThreadPool,
    num_injected_items: usize,
    running: bool,
}

impl WorkerPool {
    pub fn new(thread_pool: ThreadPool) -> Self {
        Self {
            thread_pool,
            num_injected_items: 0,
            running: false,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.thread_pool.execute(f);
    }
}

struct MatchResult {
    index_in_haystack: usize,
    score: u16,
}

impl MatchResult {
    pub fn new(index_in_haystack: usize, score: u16) -> Self {
        Self {
            index_in_haystack,
            score,
        }
    }
}

struct Status {
    pool_busy: AtomicBool,
    injector_running: AtomicBool,
    results_need_sorting: AtomicBool,
}

impl Default for Status {
    fn default() -> Self {
        Self {
            pool_busy: AtomicBool::new(false),
            injector_running: AtomicBool::new(false),
            results_need_sorting: AtomicBool::new(false),
        }
    }
}

pub struct MatcherStatus {
    pub pool_busy: bool,
    pub injector_running: bool,
}

impl From<Arc<Status>> for MatcherStatus {
    fn from(status: Arc<Status>) -> Self {
        Self {
            pool_busy: status
                .pool_busy
                .load(std::sync::atomic::Ordering::Relaxed),
            injector_running: status
                .injector_running
                .load(std::sync::atomic::Ordering::Relaxed),
        }
    }
}

type IntoHaystackFn<I> = fn(&I) -> String;

pub struct Matcher<I>
where
    I: Sync + Send + Clone + 'static,
{
    pattern: String,
    items: Arc<Mutex<Vec<I>>>,
    into_haystack: IntoHaystackFn<I>,
    worker_pool: WorkerPool,
    injection_channel_rx: Receiver<I>,
    injection_channel_tx: Sender<I>,
    /// The indices of the matched items.
    results: Arc<Mutex<Vec<MatchResult>>>,
    status: Arc<Status>,
}

const DEFAULT_ITEMS_CAPACITY: usize = 1024 * 1024;
/// The maximum number of items that can be acquired per tick.
///
/// This is used to prevent item acquisition from holding onto the lock on `self.items` for too long.
const MAX_ACQUIRED_ITEMS_PER_TICK: usize = 1024 * 1024;

const JOB_CHUNK_SIZE: usize = 1024 * 64;
const SMITH_WATERMAN_OPTS: Options = Options {
    indices: false,
    prefilter: true,
    stable_sort: false,
    unstable_sort: false,
    min_score: 0,
};

impl<I> Matcher<I>
where
    I: Sync + Send + Clone + 'static,
{
    pub fn new(f: IntoHaystackFn<I>) -> Self {
        debug!("Creating threadpool");
        let thread_pool = ThreadPool::new(usize::from(
            available_parallelism().unwrap_or(NonZero::new(8).unwrap()),
        ));
        let worker_pool = WorkerPool::new(thread_pool);
        let (sender, receiver) = unbounded();
        debug!("finished initializing matcher");
        Self {
            pattern: String::new(),
            items: Arc::new(Mutex::new(Vec::with_capacity(
                DEFAULT_ITEMS_CAPACITY,
            ))),
            into_haystack: f,
            worker_pool,
            injection_channel_rx: receiver,
            injection_channel_tx: sender,
            results: Arc::new(Mutex::new(Vec::new())),
            status: Arc::new(Status::default()),
        }
    }

    // update nucleo_pool status
    pub fn tick(&mut self) -> MatcherStatus {
        self.acquire_new_items();
        self.match_items();
        // sort the results if needed
        // this could be done in parallel
        if self
            .status
            .results_need_sorting
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            let mut results = self.results.lock_arc();
            // results.par_sort_unstable_by_key(|r| std::cmp::Reverse(r.score));
            self.status
                .results_need_sorting
                .store(false, std::sync::atomic::Ordering::Relaxed);
        }
        // update the pool status
        self.status.pool_busy.store(
            self.worker_pool.thread_pool.active_count() > 0
                || self.worker_pool.thread_pool.queued_count() > 0,
            std::sync::atomic::Ordering::Relaxed,
        );
        MatcherStatus::from(Arc::clone(&self.status))
    }

    // match any new items and update `self.results`
    pub fn match_items(&mut self) {
        // debug!("items.len(): {}, injected: {}", self.items.lock_arc().len(), self.worker_pool.num_injected_items);
        // if all items have already been fed to the worker pool, simply return
        if self.items.lock_arc().len() == self.worker_pool.num_injected_items {
            return;
        }
        let n_injected_items = self.worker_pool.num_injected_items;
        let items = self.items.lock_arc();
        let new_item_chunks: Vec<&[I]> =
            items[n_injected_items..].chunks(JOB_CHUNK_SIZE).collect();
        let into_haystack = self.into_haystack;
        let mut item_offset = n_injected_items;
        for chunk in new_item_chunks {
            let chunk = chunk.to_vec();
            let chunk_size = chunk.len();
            let pattern = self.pattern.clone();
            let results = Arc::clone(&self.results);
            let status = Arc::clone(&self.status);
            self.worker_pool.execute(move || {
                let strings: Vec<String> =
                    chunk.iter().map(|item| (into_haystack)(item)).collect();
                let matches =
                    match_list(&pattern, &strings.iter().map(|s| s.as_str()).collect::<Vec<_>>()[..], SMITH_WATERMAN_OPTS);
                // debug!("matches: {:?}", matches);
                if matches.is_empty() {
                    return;
                }
                let mut results = results.lock_arc();
                results.extend(matches.into_iter().map(|m| {
                    MatchResult::new(
                        m.index_in_haystack + item_offset,
                        m.score,
                    )
                }));
                status
                    .results_need_sorting
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            });
            self.worker_pool.num_injected_items += chunk_size;
            item_offset += chunk_size;
        }
    }

    /// reads from the injection channel and puts new items into the items vec
    fn acquire_new_items(&self) {
        let items = Arc::clone(&self.items);
        let injection_channel_rx = self.injection_channel_rx.clone();
        let status = Arc::clone(&self.status);
        self.worker_pool.execute(move || {
            let injection_channel_rx = injection_channel_rx;
            status
                .injector_running
                .store(true, std::sync::atomic::Ordering::Relaxed);
            items.lock_arc().extend(
                injection_channel_rx
                    .try_iter()
                    .take(MAX_ACQUIRED_ITEMS_PER_TICK),
            );
            status
                .injector_running
                .store(false, std::sync::atomic::Ordering::Relaxed);
        });
    }

    pub fn injector(&self) -> Injector<I> {
        Injector {
            injection_channel_tx: self.injection_channel_tx.clone(),
        }
    }

    /// this will update the pattern, clear results, and start a new search
    pub fn find(&mut self, pattern: &str) {
        if self.pattern == pattern {
            return;
        }
        self.pattern = pattern.to_string();
        self.results.lock_arc().clear();
        self.worker_pool.num_injected_items = 0;
    }

    pub fn results(
        &self,
        num_entries: u32,
        offset: u32,
    ) -> Vec<MatchedItem<I>> {
        let global_results = self.results.lock_arc();
        let mut indices = Vec::new();
        let items = self.items.lock_arc();
        global_results
            .iter()
            .skip(offset as usize)
            .take(num_entries as usize)
            .for_each(|r| {
                indices.push(r.index_in_haystack);
            });
        let matched_inner: Vec<_> =
            indices.iter().map(|i| items[*i].clone()).collect();
        let matched_strings = matched_inner
            .iter()
            .map(|s| (self.into_haystack)(s))
            .collect::<Vec<_>>();
        let matched_indices = match_list_for_matched_indices(
            &self.pattern,
            &matched_strings
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()[..],
        );
        let mut matched_items = Vec::new();
        for (inner, indices) in
            matched_inner.iter().zip(matched_indices.iter())
        {
            matched_items.push(MatchedItem {
                inner: inner.clone(),
                matched_string: (self.into_haystack)(inner),
                match_indices: indices
                    .iter()
                    .map(|i| (*i as u32, *i as u32 + 1))
                    .collect(),
            });
        }
        matched_items
    }

    pub fn get_result(&self, index: usize) -> Option<I> {
        let results = self.results.lock_arc();
        if index >= results.len() {
            return None;
        }
        let result = &results[index];
        let items = self.items.lock_arc();
        Some(items[result.index_in_haystack].clone())
    }

    pub fn result_count(&self) -> usize {
        self.results.lock_arc().len()
    }

    pub fn total_count(&self) -> usize {
        self.items.lock_arc().len()
    }

    pub fn running(&self) -> bool {
        self.status
            .injector_running
            .load(std::sync::atomic::Ordering::Relaxed)
            || self
                .status
                .pool_busy
                .load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// A matched item.
///
/// This contains the matched item, the dimension against which it was matched,
/// represented as a string, and the indices of the matched characters.
///
/// The indices are pairs of `(start, end)` where `start` is the index of the
/// first character in the match, and `end` is the index of the character after
/// the last character in the match.
#[derive(Debug, Clone)]
pub struct MatchedItem<I>
where
    I: Sync + Send + Clone + 'static,
{
    /// The matched item.
    pub inner: I,
    /// The dimension against which the item was matched (as a string).
    pub matched_string: String,
    /// The indices of the matched characters.
    pub match_indices: Vec<(u32, u32)>,
}
