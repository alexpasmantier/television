use std::sync::Arc;

use crate::entry::Entry;
use crate::previewers::{Preview, PreviewContent};

pub struct BasicPreviewer {}

impl Default for BasicPreviewer {
    fn default() -> Self {
        BasicPreviewer::new()
    }
}

impl BasicPreviewer {
    pub fn new() -> Self {
        BasicPreviewer {}
    }

    pub fn preview(&self, entry: &Entry) -> Arc<Preview> {
        Arc::new(Preview {
            title: entry.name.clone(),
            content: PreviewContent::PlainTextWrapped(entry.name.clone()),
        })
    }
}
