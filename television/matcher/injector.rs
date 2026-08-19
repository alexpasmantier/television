use super::worker::WorkerMsg;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc,
};

/// An injector that can be used to push items of type `I` into the fuzzy matcher.
///
/// Items are pushed along with the haystack string they should be matched
/// against. Batches are sent over a channel to the background worker, which
/// owns the store: pushing never blocks, even while a matching pass is
/// running. Prefer pushing many items per batch since every push incurs a
/// channel send and wakes the worker.
///
/// Injectors remain valid after the matcher is restarted, but their batches
/// are tagged with the store generation the injector was created for and are
/// silently discarded by the worker after a restart.
#[derive(Clone)]
pub struct Injector<I>
where
    I: Sync + Send + Clone + 'static,
{
    /// Channel used to send batches to the background worker.
    worker_tx: mpsc::Sender<WorkerMsg<I>>,
    /// The matcher's running flag, set when new items are pushed so the
    /// front-end can display a loading indicator right away.
    running: Arc<AtomicBool>,
    /// The store generation this injector was created for (see
    /// [`super::Matcher::restart`]).
    generation: u64,
    /// Live count of pushed items, read by
    /// [`super::Matcher::total_item_count`].
    count: Arc<AtomicUsize>,
}

impl<I> Injector<I>
where
    I: Sync + Send + Clone + 'static,
{
    pub(super) fn new(
        worker_tx: mpsc::Sender<WorkerMsg<I>>,
        running: Arc<AtomicBool>,
        generation: u64,
        count: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            worker_tx,
            running,
            generation,
            count,
        }
    }

    /// Push a single item into the fuzzy matcher.
    ///
    /// Prefer `push_batch` when pushing more than one item at a time.
    pub fn push(&self, item: I, haystack: String) {
        self.push_batch(vec![(item, haystack)]);
    }

    /// Push a batch of `(item, haystack)` pairs into the fuzzy matcher.
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

        self.count.fetch_add(batch.len(), Ordering::Relaxed);
        self.running.store(true, Ordering::Relaxed);
        let _ = self.worker_tx.send(WorkerMsg::Items {
            generation: self.generation,
            batch,
        });
    }
}
