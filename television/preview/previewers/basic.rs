use std::sync::Arc;

use crate::channels::entry::Entry;
use crate::preview::{Preview, PreviewContent};

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
            icon: entry.icon,
            partial_offset: None,
            total_lines: 1,
        })
    }
}
