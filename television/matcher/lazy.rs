use nucleo::Matcher;
use parking_lot::Mutex;
use std::ops::DerefMut;

/// A lazy-initialized mutex.
///
/// This is used to lazily initialize a nucleo matcher (which pre-allocates
/// quite a bit of memory upfront which can be expensive during initialization).
///
/// # Example
/// ```rust
/// use television::matcher::lazy::LazyMutex;
///
/// struct Thing {
///     // ...
/// }
///
/// impl Thing {
///     fn new() -> Self {
///         // something expensive
///         Thing { }
///     }
/// }
///
/// static THING_TO_LAZY_INIT: LazyMutex<Thing> = LazyMutex::new(|| {
///    Thing::new()
/// });
///
/// let _thing = THING_TO_LAZY_INIT.lock();
/// ```
pub struct LazyMutex<T> {
    /// The inner value, wrapped in a mutex.
    inner: Mutex<Option<T>>,
    /// The initialization function.
    init: fn() -> T,
}

impl<T> LazyMutex<T> {
    pub const fn new(init: fn() -> T) -> Self {
        Self {
            inner: Mutex::new(None),
            init,
        }
    }

    /// Locks the mutex and returns a guard that allows mutable access to the
    /// inner value.
    pub fn lock(&self) -> impl DerefMut<Target = T> + '_ {
        parking_lot::MutexGuard::map(self.inner.lock(), |val| {
            val.get_or_insert_with(self.init)
        })
    }
}

/// A lazy-initialized nucleo matcher used for conveniently computing match indices.
///
/// This is used to lazily initialize a nucleo matcher (which pre-allocates quite a bit of memory
/// upfront which can be expensive at initialization).
///
/// This matcher is used as a convenience for computing match indices on a subset of matched items.
///
/// # Example
/// ```ignore
/// use television::matcher::{lazy::MATCHER, matched_item::MatchedItem};
///
/// let snapshot = channel_matcher.snapshot();
///
/// let mut col_indices = vec![];
/// let mut matcher = MATCHER.lock();
///
/// snapshot
///     .matched_items(..)
///     .map(move |item| {
///         snapshot.pattern().column_pattern(0).indices(
///             item.matcher_columns[0].slice(..),
///             &mut matcher,
///             &mut col_indices,
///         );
///         col_indices.sort_unstable();
///         col_indices.dedup();
///
///         let indices = col_indices.drain(..);
///
///         let matched_string = item.matcher_columns[0].to_string();
///         MatchedItem {
///             inner: item.data.clone(),
///             matched_string,
///             match_indices: indices.map(|i| (i, i + 1)).collect(),
///         }
///     })
///     .collect();
/// ```
pub static MATCHER: LazyMutex<Matcher> = LazyMutex::new(Matcher::default);
