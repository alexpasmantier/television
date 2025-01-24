use rustc_hash::FxHashMap;
use std::sync::Arc;

use crate::channels::entry;
use crate::preview::{Preview, PreviewContent};

#[derive(Debug, Default)]
pub struct EnvVarPreviewer {
    cache: FxHashMap<entry::Entry, Arc<Preview>>,
    _config: EnvVarPreviewerConfig,
}

#[derive(Debug, Default)]
pub struct EnvVarPreviewerConfig {}

impl EnvVarPreviewer {
    pub fn new(config: Option<EnvVarPreviewerConfig>) -> Self {
        EnvVarPreviewer {
            cache: FxHashMap::default(),
            _config: config.unwrap_or_default(),
        }
    }

    pub fn preview(&mut self, entry: &entry::Entry) -> Arc<Preview> {
        // check if we have that preview in the cache
        if let Some(preview) = self.cache.get(entry) {
            return preview.clone();
        }
        let content = entry.value.as_ref().map(|preview| {
            maybe_add_newline_after_colon(preview, &entry.name)
        });
        let total_lines = content.as_ref().map_or_else(
            || 1,
            |c| u16::try_from(c.lines().count()).unwrap_or(u16::MAX),
        );
        let preview = Arc::new(Preview {
            title: entry.name.clone(),
            content: match content {
                Some(content) => PreviewContent::PlainTextWrapped(content),
                None => PreviewContent::Empty,
            },
            icon: entry.icon,
            partial_offset: None,
            total_lines,
        });
        self.cache.insert(entry.clone(), preview.clone());
        preview
    }
}

const PATH: &str = "PATH";

fn maybe_add_newline_after_colon(s: &str, name: &str) -> String {
    if name.contains(PATH) {
        return s.replace(':', "\n");
    }
    s.to_string()
}
