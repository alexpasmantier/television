use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

use tracing::debug;

use crate::previewers::Preview;

/// TODO: add unit tests
/// A ring buffer that also keeps track of the keys it contains to avoid duplicates.
///
/// I'm planning on using this as a backend LRU-cache for the preview cache.
/// Basic idea:
/// - When a new key is pushed, if it's already in the buffer, do nothing.
/// - If the buffer is full, remove the oldest key and push the new key.
struct RingSet<T> {
    ring_buffer: VecDeque<T>,
    known_keys: HashSet<T>,
    capacity: usize,
}

impl<T> RingSet<T>
where
    T: Eq + std::hash::Hash + Clone + std::fmt::Debug,
{
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

/// Default size of the preview cache.
/// This does seem kind of arbitrary for now, will need to play around with it.
const DEFAULT_PREVIEW_CACHE_SIZE: usize = 100;

/// A cache for previews.
/// The cache is implemented as a LRU cache with a fixed size.
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
}

impl Default for PreviewCache {
    fn default() -> Self {
        PreviewCache::new(DEFAULT_PREVIEW_CACHE_SIZE)
    }
}
