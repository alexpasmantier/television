use std::path::Path;
use std::sync::Arc;

use devicons::FileIcon;
use parking_lot::Mutex;
use termtree::Tree;

use television_channels::entry::Entry;

use crate::previewers::cache::PreviewCache;
use crate::previewers::{meta, Preview, PreviewContent};
use television_utils::files::walk_builder;

#[derive(Debug, Default)]
pub struct DirectoryPreviewer {
    cache: Arc<Mutex<PreviewCache>>,
    _config: DirectoryPreviewerConfig,
}

#[derive(Debug, Default)]
pub struct DirectoryPreviewerConfig {}

impl DirectoryPreviewer {
    pub fn new(config: Option<DirectoryPreviewerConfig>) -> Self {
        DirectoryPreviewer {
            cache: Arc::new(Mutex::new(PreviewCache::default())),
            _config: config.unwrap_or_default(),
        }
    }

    pub fn preview(&mut self, entry: &Entry) -> Arc<Preview> {
        if let Some(preview) = self.cache.lock().get(&entry.name) {
            return preview;
        }
        let preview = meta::loading(&entry.name);
        self.cache
            .lock()
            .insert(entry.name.clone(), preview.clone());
        let entry_c = entry.clone();
        let cache = self.cache.clone();
        tokio::spawn(async move {
            let preview = Arc::new(build_tree_preview(&entry_c));
            cache.lock().insert(entry_c.name.clone(), preview.clone());
        });
        preview
    }
}

fn build_tree_preview(entry: &Entry) -> Preview {
    let path = Path::new(&entry.name);
    let tree = tree(path, MAX_DEPTH, FIRST_LEVEL_MAX_ENTRIES, &mut 0);
    let tree_string = tree.to_string();
    Preview {
        title: entry.name.clone(),
        content: PreviewContent::PlainText(
            tree_string
                .lines()
                .map(std::borrow::ToOwned::to_owned)
                .collect(),
        ),
        icon: entry.icon,
    }
}

fn label<P: AsRef<Path>>(p: P, strip: &str) -> String {
    let icon = FileIcon::from(&p);
    let path = p.as_ref().strip_prefix(strip).unwrap_or(p.as_ref());
    format!("{} {}", icon, path.display())
}

const MAX_DEPTH: u8 = 4;
const FIRST_LEVEL_MAX_ENTRIES: u8 = 30;
const NESTED_MAX_ENTRIES: u8 = 10;
const MAX_ENTRIES: u8 = 200;

fn tree<P: AsRef<Path>>(
    p: P,
    max_depth: u8,
    nested_max_entries: u8,
    total_entry_count: &mut u8,
) -> Tree<String> {
    let mut root = Tree::new(label(
        p.as_ref(),
        p.as_ref().parent().unwrap().to_str().unwrap(),
    ));
    let w = walk_builder(p.as_ref(), 1, None, None)
        .max_depth(Some(1))
        .build();
    let mut level_entry_count: u8 = 0;

    for path in w.skip(1).filter_map(Result::ok) {
        let m = path.metadata().unwrap();
        if m.is_dir() && max_depth > 1 {
            root.push(tree(
                path.path(),
                max_depth - 1,
                NESTED_MAX_ENTRIES,
                total_entry_count,
            ));
        } else {
            root.push(Tree::new(label(
                path.path(),
                p.as_ref().to_str().unwrap(),
            )));
        }
        level_entry_count += 1;
        *total_entry_count += 1;
        if level_entry_count >= nested_max_entries
            || *total_entry_count >= MAX_ENTRIES
        {
            root.push(Tree::new(String::from("...")));
            break;
        }
    }

    root
}
