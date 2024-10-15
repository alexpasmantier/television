use std::collections::HashMap;
use std::sync::Arc;

use crate::entry;
use crate::previewers::{Preview, PreviewContent};

pub(crate) struct EnvVarPreviewer {
    cache: HashMap<entry::Entry, Arc<Preview>>,
}

impl EnvVarPreviewer {
    pub(crate) fn new() -> Self {
        EnvVarPreviewer {
            cache: HashMap::new(),
        }
    }

    pub(crate) fn preview(&mut self, entry: &entry::Entry) -> Arc<Preview> {
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
