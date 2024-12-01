use std::fmt::Debug;
use std::num::NonZero;
use std::ops::Deref;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::{available_parallelism, spawn};

use crossbeam_channel::{unbounded, Receiver, Sender};
use frizbee::{match_list, match_list_for_matched_indices, Options};
use parking_lot::Mutex;
use rayon::prelude::ParallelSliceMut;
use threadpool::ThreadPool;
use tracing::debug;

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

type IntoHaystackFn<I> = fn(&I) -> &str;

pub struct Matcher<I>
where
    I: Sync + Send + Clone + 'static + Debug,
{
    pattern: String,
    items: Arc<boxcar::Vec<I>>,
    into_haystack: IntoHaystackFn<I>,
    worker_pool: WorkerPool,
    injection_channel_rx: Receiver<I>,
    injection_channel_tx: Sender<I>,
    results: Arc<boxcar::Vec<MatchResult>>,
    status: Arc<Status>,
}

const DEFAULT_ITEMS_CAPACITY: usize = 1024 * 1024;
/// The maximum number of items that can be acquired per tick.
///
/// This is used to prevent item acquisition from holding onto the lock on `self.items` for too long.
const MAX_ACQUIRED_ITEMS_PER_TICK: usize = 1024 * 1024 * 4;

/// Number of items to match in a single simd job.
const JOB_CHUNK_SIZE: usize = 1024 * 1024;
const SMITH_WATERMAN_OPTS: Options = Options {
    indices: false,
    prefilter: true,
    stable_sort: false,
    unstable_sort: false,
    min_score: 0,
};

impl<I> Matcher<I>
where
    I: Sync + Send + Clone + 'static + Debug,
{
    pub fn new(f: IntoHaystackFn<I>) -> Self {
        let thread_pool = ThreadPool::with_name(
            "SimdMatcher".to_string(),
            usize::from(
                available_parallelism().unwrap_or(NonZero::new(8).unwrap()),
            ),
        );
        let worker_pool = WorkerPool::new(thread_pool);
        let (sender, receiver) = unbounded();
        Self {
            pattern: String::new(),
            items: Arc::new(boxcar::Vec::with_capacity(
                DEFAULT_ITEMS_CAPACITY,
            )),
            into_haystack: f,
            worker_pool,
            injection_channel_rx: receiver,
            injection_channel_tx: sender,
            results: Arc::new(boxcar::Vec::new()),
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
            debug!("Sorting results");
            // let mut results = self.results.clone();
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
        let item_count = self.items.count();
        if item_count == self.worker_pool.num_injected_items {
            return;
        }
        let n_injected_items = self.worker_pool.num_injected_items;
        let pattern = self.pattern.clone();

        let mut chunks = Vec::new();
        let mut offsets = Vec::new();
        let mut current_offset = n_injected_items;
        let items = Arc::clone(&self.items);
        loop {
            if current_offset >= item_count {
                break;
            }
            let chunk_size = (item_count - current_offset).min(JOB_CHUNK_SIZE);
            chunks.push(
                items
                    .iter()
                    .skip(current_offset)
                    .take(chunk_size)
                    .map(|(_, v)| (self.into_haystack)(v)),
            );
            offsets.push(current_offset);
            current_offset += chunk_size;
        }

        let offsets_c = offsets.clone();

        for (i, chunk) in chunks.into_iter().enumerate() {
            let pattern = pattern.clone();
            let results = Arc::clone(&self.results);
            let status = Arc::clone(&self.status);
            let cur_offset = offsets_c[i];
            self.worker_pool.execute(move || {
                let matches = match_list(
                    &pattern,
                    &chunk.collect::<Vec<&str>>(),
                    SMITH_WATERMAN_OPTS,
                );
                if matches.is_empty() {
                    return;
                }
                for m in &matches {
                    results.push(MatchResult::new(
                        m.index_in_haystack + cur_offset,
                        m.score,
                    ));
                }
                status
                    .results_need_sorting
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            });
        }
        self.worker_pool.num_injected_items = item_count;
    }

    /// reads from the injection channel and puts new items into the items vec
    fn acquire_new_items(&self) {
        let items = Arc::clone(&self.items);
        if self.injection_channel_rx.is_empty() {
            return;
        }
        debug!("Acquiring new items");
        let injection_channel_rx = self.injection_channel_rx.clone();
        let status = Arc::clone(&self.status);
        spawn(move || {
            status
                .injector_running
                .store(true, std::sync::atomic::Ordering::Relaxed);
            for item in injection_channel_rx
                .try_iter()
                .take(MAX_ACQUIRED_ITEMS_PER_TICK)
            {
                items.push(item);
            }
            status
                .injector_running
                .store(false, std::sync::atomic::Ordering::Relaxed);
        });
        //self.worker_pool.execute(move || {
        //    let injection_channel_rx = injection_channel_rx;
        //    status
        //        .injector_running
        //        .store(true, std::sync::atomic::Ordering::Relaxed);
        //    items.lock_arc().extend(
        //        injection_channel_rx
        //            .try_iter()
        //            .take(MAX_ACQUIRED_ITEMS_PER_TICK),
        //    );
        //    status
        //        .injector_running
        //        .store(false, std::sync::atomic::Ordering::Relaxed);
        //});
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
        self.results = Arc::new(boxcar::Vec::new());
        self.worker_pool.num_injected_items = 0;
    }

    pub fn results(
        &self,
        num_entries: u32,
        offset: u32,
    ) -> Vec<MatchedItem<I>> {
        let mut indices = Vec::new();
        self.results
            .iter()
            .skip(offset as usize)
            .map(|(_, v)| v)
            .take(num_entries as usize)
            .for_each(|r| {
                indices.push(r.index_in_haystack);
            });
        let matched_inner: Vec<_> =
            indices.iter().map(|i| self.items[*i].clone()).collect();
        let matched_indices = match_list_for_matched_indices(
            &self.pattern,
            &matched_inner
                .iter()
                .map(|item| (self.into_haystack)(item))
                .collect::<Vec<_>>(),
        );
        let mut matched_items = Vec::new();
        for (inner, indices) in
            matched_inner.iter().zip(matched_indices.iter())
        {
            matched_items.push(MatchedItem {
                inner: inner.clone(),
                matched_string: (self.into_haystack)(inner).to_string(),
                match_indices: indices
                    .iter()
                    .map(|i| (*i as u32, *i as u32 + 1))
                    .collect(),
            });
        }
        matched_items
    }

    pub fn get_result(&self, index: usize) -> Option<MatchedItem<I>> {
        if index >= self.results.count() {
            return None;
        }
        let result = &self.results[index];
        Some(MatchedItem {
            inner: self.items[result.index_in_haystack].clone(),
            matched_string: (self.into_haystack)(
                &self.items[result.index_in_haystack],
            )
            .to_string(),
            match_indices: vec![],
        })
    }

    pub fn result_count(&self) -> usize {
        self.results.count()
    }

    pub fn total_count(&self) -> usize {
        self.items.count()
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
