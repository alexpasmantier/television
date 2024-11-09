use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

use tracing::debug;

use crate::previewers::Preview;

/// A ring buffer that also keeps track of the keys it contains to avoid duplicates.
///
/// This serves as a backend for the preview cache.
/// Basic idea:
/// - When a new key is pushed, if it's already in the buffer, do nothing.
/// - If the buffer is full, remove the oldest key and push the new key.
///
/// # Example
/// ```rust
/// let mut ring_set = RingSet::with_capacity(3);
/// // push 3 values into the ringset
/// assert_eq!(ring_set.push(1), None);
/// assert_eq!(ring_set.push(2), None);
/// assert_eq!(ring_set.push(3), None);
///
/// // check that the values are in the buffer
/// assert!(ring_set.contains(&1));
/// assert!(ring_set.contains(&2));
/// assert!(ring_set.contains(&3));
///
/// // push an existing value (should do nothing)
/// assert_eq!(ring_set.push(1), None);
///
/// // entries should still be there
/// assert!(ring_set.contains(&1));
/// assert!(ring_set.contains(&2));
/// assert!(ring_set.contains(&3));
///
/// // push a new value, should remove the oldest value (1)
/// assert_eq!(ring_set.push(4), Some(1));
///
/// // 1 is no longer there but 2 and 3 remain
/// assert!(!ring_set.contains(&1));
/// assert!(ring_set.contains(&2));
/// assert!(ring_set.contains(&3));
/// assert!(ring_set.contains(&4));
/// ```
#[derive(Debug)]
struct RingSet<T> {
    ring_buffer: VecDeque<T>,
    known_keys: HashSet<T>,
    capacity: usize,
}

impl<T> RingSet<T>
where
    T: Eq + std::hash::Hash + Clone + std::fmt::Debug,
{
    /// Create a new `RingSet` with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        RingSet {
            ring_buffer: VecDeque::with_capacity(capacity),
            known_keys: HashSet::with_capacity(capacity),
            capacity,
        }
    }

    /// Push a new item to the back of the buffer, removing the oldest item if the buffer is full.
    /// Returns the item that was removed, if any.
    /// If the item is already in the buffer, do nothing and return None.
    pub fn push(&mut self, item: T) -> Option<T> {
        // If the key is already in the buffer, do nothing
        if self.contains(&item) {
            debug!("Key already in ring buffer: {:?}", item);
            return None;
        }
        let mut popped_key = None;
        // If the buffer is full, remove the oldest key (e.g. pop from the front of the buffer)
        if self.ring_buffer.len() >= self.capacity {
            popped_key = self.pop();
        }
        // finally, push the new key to the back of the buffer
        self.ring_buffer.push_back(item.clone());
        self.known_keys.insert(item);
        popped_key
    }

    fn pop(&mut self) -> Option<T> {
        if let Some(item) = self.ring_buffer.pop_front() {
            debug!("Removing key from ring buffer: {:?}", item);
            self.known_keys.remove(&item);
            Some(item)
        } else {
            None
        }
    }

    fn contains(&self, key: &T) -> bool {
        self.known_keys.contains(key)
    }
}

/// Default size of the preview cache: 100 entries.
///
/// This does seem kind of arbitrary for now, will need to play around with it.
/// At the moment, files over 4 MB are not previewed, so the cache size
/// shouldn't exceed 400 MB.
const DEFAULT_PREVIEW_CACHE_SIZE: usize = 100;

/// A cache for previews.
/// The cache is implemented as an LRU cache with a fixed size.
#[derive(Debug)]
pub struct PreviewCache {
    entries: HashMap<String, Arc<Preview>>,
    ring_set: RingSet<String>,
}

impl PreviewCache {
    /// Create a new preview cache with the given capacity.
    pub fn new(capacity: usize) -> Self {
        PreviewCache {
            entries: HashMap::new(),
            ring_set: RingSet::with_capacity(capacity),
        }
    }

    pub fn get(&self, key: &str) -> Option<Arc<Preview>> {
        self.entries.get(key).cloned()
    }

    /// Insert a new preview into the cache.
    /// If the cache is full, the oldest entry will be removed.
    /// If the key is already in the cache, the preview will be updated.
    pub fn insert(&mut self, key: String, preview: Arc<Preview>) {
        debug!("Inserting preview into cache: {}", key);
        self.entries.insert(key.clone(), preview.clone());
        if let Some(oldest_key) = self.ring_set.push(key) {
            debug!("Cache full, removing oldest entry: {}", oldest_key);
            self.entries.remove(&oldest_key);
        }
    }

    /// Get the preview for the given key, or insert a new preview if it doesn't exist.
    #[allow(dead_code)]
    pub fn get_or_insert<F>(&mut self, key: String, f: F) -> Arc<Preview>
    where
        F: FnOnce() -> Preview,
    {
        if let Some(preview) = self.get(&key) {
            preview
        } else {
            let preview = Arc::new(f());
            self.insert(key, preview.clone());
            preview
        }
    }
}

impl Default for PreviewCache {
    fn default() -> Self {
        PreviewCache::new(DEFAULT_PREVIEW_CACHE_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_set() {
        let mut ring_set = RingSet::with_capacity(3);
        // push 3 values into the ringset
        assert_eq!(ring_set.push(1), None);
        assert_eq!(ring_set.push(2), None);
        assert_eq!(ring_set.push(3), None);

        // check that the values are in the buffer
        assert!(ring_set.contains(&1));
        assert!(ring_set.contains(&2));
        assert!(ring_set.contains(&3));

        // push an existing value (should do nothing)
        assert_eq!(ring_set.push(1), None);

        // entries should still be there
        assert!(ring_set.contains(&1));
        assert!(ring_set.contains(&2));
        assert!(ring_set.contains(&3));

        // push a new value, should remove the oldest value (1)
        assert_eq!(ring_set.push(4), Some(1));

        // 1 is no longer there but 2 and 3 remain
        assert!(!ring_set.contains(&1));
        assert!(ring_set.contains(&2));
        assert!(ring_set.contains(&3));
        assert!(ring_set.contains(&4));

        // push two new values, should remove 2 and 3
        assert_eq!(ring_set.push(5), Some(2));
        assert_eq!(ring_set.push(6), Some(3));

        // 2 and 3 are no longer there but 4, 5 and 6 remain
        assert!(!ring_set.contains(&2));
        assert!(!ring_set.contains(&3));
        assert!(ring_set.contains(&4));
        assert!(ring_set.contains(&5));
        assert!(ring_set.contains(&6));
    }
}
