use rustc_hash::FxHashMap;
use std::sync::Arc;

use ratatui::widgets::Paragraph;
use television_utils::cache::RingSet;

const DEFAULT_RENDERED_PREVIEW_CACHE_SIZE: usize = 25;

#[derive(Debug)]
pub struct RenderedPreviewCache<'a> {
    previews: FxHashMap<String, Arc<Paragraph<'a>>>,
    ring_set: RingSet<String>,
}

impl<'a> RenderedPreviewCache<'a> {
    pub fn new(capacity: usize) -> Self {
        RenderedPreviewCache {
            previews: FxHashMap::default(),
            ring_set: RingSet::with_capacity(capacity),
        }
    }

    pub fn get(&self, key: &str) -> Option<Arc<Paragraph<'a>>> {
        self.previews.get(key).cloned()
    }

    pub fn insert(&mut self, key: String, preview: &Arc<Paragraph<'a>>) {
        self.previews.insert(key.clone(), preview.clone());
        if let Some(oldest_key) = self.ring_set.push(key) {
            self.previews.remove(&oldest_key);
        }
    }
}

impl Default for RenderedPreviewCache<'_> {
    fn default() -> Self {
        RenderedPreviewCache::new(DEFAULT_RENDERED_PREVIEW_CACHE_SIZE)
    }
}
