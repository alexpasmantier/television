use rustc_hash::FxHashMap;

use crate::channels::entry::Entry;
use crate::previewer::Preview;
use crate::utils::cache::RingSet;
use tracing::debug;

/// Default size of the preview cache: 50 entries.
///
/// This does seem kind of arbitrary for now, will need to play around with it.
/// Assuming a worst case scenario where files are an average of 1 MB this means
/// the cache will never exceed 50 MB which sounds safe enough.
const DEFAULT_CACHE_SIZE: usize = 50;

/// A cache for previews.
/// The cache is implemented as an LRU cache with a fixed size.
#[derive(Debug)]
pub struct Cache {
    entries: FxHashMap<Entry, Preview>,
    ring_set: RingSet<Entry>,
}

impl Cache {
    /// Create a new preview cache with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Cache {
            entries: FxHashMap::default(),
            ring_set: RingSet::with_capacity(capacity),
        }
    }

    pub fn get(&self, key: &Entry) -> Option<Preview> {
        self.entries.get(key).cloned()
    }

    /// Insert a new preview into the cache.
    /// If the cache is full, the oldest entry will be removed.
    /// If the key is already in the cache, the preview will be updated.
    pub fn insert(&mut self, key: &Entry, preview: &Preview) {
        debug!("Inserting preview into cache for key: {:?}", key);
        self.entries.insert(key.clone(), preview.clone());
        if let Some(oldest_key) = self.ring_set.push(key.clone()) {
            debug!("Cache full, removing oldest entry: {:?}", oldest_key);
            self.entries.remove(&oldest_key);
        }
    }

    /// In this context, the size represents the number of occupied slots within the cache.
    /// Not to be mistaken with the cache's capacity which is its max size.
    pub fn size(&self) -> usize {
        self.ring_set.size()
    }

    pub fn clear(&mut self) {
        debug!("Clearing preview cache");
        self.entries.clear();
        self.ring_set.clear();
    }
}

impl Default for Cache {
    fn default() -> Self {
        Cache::new(DEFAULT_CACHE_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use ratatui::text::Text;

    use super::*;
    use crate::channels::entry::Entry;
    use crate::previewer::Preview;

    #[test]
    fn test_preview_cache_ops() {
        let mut cache = Cache::new(2);
        let entry = Entry::new("test".to_string());
        let preview = Preview::default();

        cache.insert(&entry, &preview);
        assert_eq!(cache.get(&entry).unwrap(), preview);
        assert_eq!(cache.size(), 1);

        // override cache content for the same key
        let mut other_preview = preview.clone();
        other_preview.content = Text::raw("some content");
        cache.insert(&entry, &other_preview);
        assert_eq!(cache.get(&entry).unwrap(), other_preview);
        assert_eq!(cache.size(), 1);

        // insert new entries to trigger eviction
        let new_entry = Entry::new("new_test".to_string());
        let new_preview = Preview::default();
        cache.insert(&new_entry, &new_preview);
        // the two previews should still be available
        assert_eq!(cache.size(), 2);
        assert_eq!(cache.get(&new_entry).unwrap(), new_preview);
        assert_eq!(cache.get(&entry).unwrap(), other_preview);
        // this one should trigger eviction
        let another_entry = Entry::new("another_test".to_string());
        cache.insert(&another_entry, &Preview::default());

        assert_eq!(cache.size(), 2);
        assert!(cache.get(&entry).is_none());
        assert!(cache.get(&new_entry).is_some());
        assert!(cache.get(&another_entry).is_some());
        assert_eq!(cache.get(&new_entry).unwrap(), Preview::default());
        assert_eq!(cache.get(&another_entry).unwrap(), Preview::default());
    }
}
