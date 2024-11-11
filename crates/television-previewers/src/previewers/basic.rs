use std::sync::Arc;

use crate::previewers::{Preview, PreviewContent};
use television_channels::entry::Entry;

#[derive(Debug, Default)]
pub struct BasicPreviewer {
    _config: BasicPreviewerConfig,
}

#[derive(Debug, Default)]
pub struct BasicPreviewerConfig {}

impl BasicPreviewer {
    pub fn new(config: Option<BasicPreviewerConfig>) -> Self {
        BasicPreviewer {
            _config: config.unwrap_or_default(),
        }
    }

    pub fn preview(&self, entry: &Entry) -> Arc<Preview> {
        Arc::new(Preview {
            title: entry.name.clone(),
            content: PreviewContent::PlainTextWrapped(entry.name.clone()),
        })
    }
}
