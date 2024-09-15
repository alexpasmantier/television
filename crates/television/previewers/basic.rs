use std::sync::Arc;

use crate::entry::Entry;
use crate::previewers::{Preview, PreviewContent};

pub struct BasicPreviewer {}

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
