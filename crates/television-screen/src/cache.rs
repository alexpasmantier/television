use devicons::FileIcon;
use rustc_hash::FxHashMap;
use std::sync::Arc;

use ratatui::widgets::Paragraph;
use television_utils::cache::RingSet;

const DEFAULT_RENDERED_PREVIEW_CACHE_SIZE: usize = 10;

#[derive(Clone, Debug)]
pub struct CachedPreview<'a> {
    pub key: String,
    pub icon: Option<FileIcon>,
    pub title: String,
    pub paragraph: Arc<Paragraph<'a>>,
}

impl<'a> CachedPreview<'a> {
    pub fn new(
        key: String,
        icon: Option<FileIcon>,
        title: String,
        paragraph: Arc<Paragraph<'a>>,
    ) -> Self {
        CachedPreview {
            key,
            icon,
            title,
            paragraph,
        }
    }
}

#[derive(Debug)]
pub struct RenderedPreviewCache<'a> {
    previews: FxHashMap<String, CachedPreview<'a>>,
    ring_set: RingSet<String>,
    pub last_preview: Option<CachedPreview<'a>>,
}

impl<'a> RenderedPreviewCache<'a> {
    pub fn new(capacity: usize) -> Self {
        RenderedPreviewCache {
            previews: FxHashMap::default(),
            ring_set: RingSet::with_capacity(capacity),
            last_preview: None,
        }
    }

    pub fn get(&self, key: &str) -> Option<CachedPreview<'a>> {
        self.previews.get(key).cloned()
    }

    pub fn insert(
        &mut self,
        key: String,
        icon: Option<FileIcon>,
        title: String,
        paragraph: &Arc<Paragraph<'a>>,
    ) {
        let cached_preview = CachedPreview::new(
            key.clone(),
            icon,
            title.clone(),
            paragraph.clone(),
        );
        self.last_preview = Some(cached_preview.clone());
        self.previews.insert(key.clone(), cached_preview);
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
