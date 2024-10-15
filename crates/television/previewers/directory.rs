use std::path::Path;
use std::sync::Arc;

use devicons::FileIcon;

use crate::entry::Entry;

use crate::previewers::cache::PreviewCache;
use crate::previewers::{Preview, PreviewContent};

pub(crate) struct DirectoryPreviewer {
    cache: PreviewCache,
}

impl DirectoryPreviewer {
    pub(crate) fn new() -> Self {
        DirectoryPreviewer {
            cache: PreviewCache::default(),
        }
    }

    pub(crate) fn preview(&mut self, entry: &Entry) -> Arc<Preview> {
        if let Some(preview) = self.cache.get(&entry.name) {
            return preview;
        }
        let preview = Arc::new(build_preview(entry));
        self.cache.insert(entry.name.clone(), preview.clone());
        preview
    }
}

fn build_preview(entry: &Entry) -> Preview {
    let dir_path = Path::new(&entry.name);
    // get the list of files in the directory
    let mut lines = vec![];
    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                lines.push(format!(
                    "{} {}",
                    FileIcon::from(&file_name),
                    &file_name
                ));
            }
        }
    }

    Preview {
        title: entry.name.clone(),
        content: PreviewContent::PlainText(lines),
    }
}
