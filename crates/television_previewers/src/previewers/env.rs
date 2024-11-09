use std::collections::HashMap;
use std::sync::Arc;

use crate::previewers::{Preview, PreviewContent};
use television_channels::entry;

#[derive(Debug, Default)]
pub struct EnvVarPreviewer {
    cache: HashMap<entry::Entry, Arc<Preview>>,
    config: EnvVarPreviewerConfig,
}

#[derive(Debug, Default)]
pub struct EnvVarPreviewerConfig {}

impl EnvVarPreviewer {
    pub fn new(config: Option<EnvVarPreviewerConfig>) -> Self {
        EnvVarPreviewer {
            cache: HashMap::new(),
            config: config.unwrap_or_default(),
        }
    }

    pub fn preview(&mut self, entry: &entry::Entry) -> Arc<Preview> {
        // check if we have that preview in the cache
        if let Some(preview) = self.cache.get(entry) {
            return preview.clone();
        }
        let preview = Arc::new(Preview {
            title: entry.name.clone(),
            content: if let Some(preview) = &entry.value {
                PreviewContent::PlainTextWrapped(
                    maybe_add_newline_after_colon(preview, &entry.name),
                )
            } else {
                PreviewContent::Empty
            },
        });
        self.cache.insert(entry.clone(), preview.clone());
        preview
    }
}

const PATH: &str = "PATH";

fn maybe_add_newline_after_colon(s: &str, name: &str) -> String {
    if name.contains(PATH) {
        return s.replace(":", "\n");
    }
    s.to_string()
}
