use super::worker::{Store, WorkerMsg};
use parking_lot::RwLock;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

/// An injector that can be used to push items of type `I` into the fuzzy matcher.
///
/// Items are pushed along with the haystack string they should be matched
/// against. Injectors write directly into the matcher's shared store and
/// notify the background worker once per batch, so pushing many items at once
/// is much cheaper than pushing them one by one.
///
/// Injectors remain valid after the matcher is restarted, but their pushes
/// land in the store the matcher was using when the injector was created and
/// are silently discarded along with it.
#[derive(Clone)]
pub struct Injector<I>
where
    I: Sync + Send + Clone + 'static,
{
    /// The store shared with the matcher and its background worker.
    store: Arc<RwLock<Store<I>>>,
    /// Channel used to notify the background worker of newly pushed items.
    worker_tx: mpsc::Sender<WorkerMsg<I>>,
    /// The matcher's running flag, set when new items are pushed so the
    /// front-end can display a loading indicator right away.
    running: Arc<AtomicBool>,
}

impl<I> Injector<I>
where
    I: Sync + Send + Clone + 'static,
{
    pub(super) fn new(
        store: Arc<RwLock<Store<I>>>,
        worker_tx: mpsc::Sender<WorkerMsg<I>>,
        running: Arc<AtomicBool>,
    ) -> Self {
        Self {
            store,
            worker_tx,
            running,
        }
    }

    /// Push a single item into the fuzzy matcher.
    ///
    /// The item will be matched against the given `haystack` string.
    ///
    /// Prefer `push_batch` when pushing more than one item at a time.
    pub fn push(&self, item: I, haystack: String) {
        self.push_batch(vec![(item, haystack)]);
    }

    /// Push a batch of `(item, haystack)` pairs into the fuzzy matcher.
    ///
    /// The whole batch is appended under a single store lock and the
    /// background worker is only notified once. Any per-item work (e.g.
    /// stripping ANSI codes) should be done before calling this to keep the
    /// store lock held as briefly as possible.
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
    pub fn push_batch(&self, batch: Vec<(I, String)>) {
        if batch.is_empty() {
            return;
        }

        {
            let mut store = self.store.write();
            store.items.reserve(batch.len());
            store.haystacks.reserve(batch.len());
            for (item, haystack) in batch {
                store.items.push(item);
                store.haystacks.push(haystack);
            }
        }

        self.running.store(true, Ordering::Relaxed);
        let _ = self.worker_tx.send(WorkerMsg::ItemsAdded);
    }
}
