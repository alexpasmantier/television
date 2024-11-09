use std::sync::Arc;

use crate::entry::Entry;
use crate::previewers::{Preview, PreviewContent};

#[derive(Debug, Default)]
pub struct BasicPreviewer {
    config: BasicPreviewerConfig,
}

#[derive(Debug, Default)]
pub struct BasicPreviewerConfig {}

impl BasicPreviewer {
    pub fn new(config: Option<BasicPreviewerConfig>) -> Self {
        BasicPreviewer {
            config: config.unwrap_or_default(),
        }
    }

    pub fn preview(&self, entry: &Entry) -> Arc<Preview> {
        Arc::new(Preview {
            title: entry.name.clone(),
            content: PreviewContent::PlainTextWrapped(entry.name.clone()),
        })
    }
}
