use std::{collections::HashMap, sync::Arc};

use crate::previewers::Preview;
use television_utils::cache::RingSet;
use tracing::debug;

/// Default size of the preview cache: 100 entries.
///
/// This does seem kind of arbitrary for now, will need to play around with it.
/// At the moment, files over 4 MB are not previewed, so the cache size
/// should never exceed 400 MB.
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
    pub fn insert(&mut self, key: String, preview: &Arc<Preview>) {
        debug!("Inserting preview into cache: {}", key);
        self.entries.insert(key.clone(), Arc::clone(preview));
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
            self.insert(key, &preview);
            preview
        }
    }
}

impl Default for PreviewCache {
    fn default() -> Self {
        PreviewCache::new(DEFAULT_PREVIEW_CACHE_SIZE)
    }
}
