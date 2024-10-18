use std::path::Path;
use std::sync::Arc;

use devicons::FileIcon;
use termtree::Tree;

use crate::entry::Entry;

use crate::previewers::cache::PreviewCache;
use crate::previewers::{Preview, PreviewContent};
use crate::utils::files::walk_builder;

pub struct DirectoryPreviewer {
    cache: PreviewCache,
}

impl DirectoryPreviewer {
    pub fn new() -> Self {
        DirectoryPreviewer {
            cache: PreviewCache::default(),
        }
    }

    pub fn preview(&mut self, entry: &Entry) -> Arc<Preview> {
        if let Some(preview) = self.cache.get(&entry.name) {
            return preview;
        }
        let preview = Arc::new(build_tree_preview(entry));
        self.cache.insert(entry.name.clone(), preview.clone());
        preview
    }
}

fn build_tree_preview(entry: &Entry) -> Preview {
    let path = Path::new(&entry.name);
    let tree = tree(path).unwrap();
    let tree_string = tree.to_string();
    Preview {
        title: entry.name.clone(),
        content: PreviewContent::PlainText(
            tree_string
                .lines()
                .map(std::borrow::ToOwned::to_owned)
                .collect(),
        ),
    }
}

fn label<P: AsRef<Path>>(p: P, strip: &str) -> String {
    //let path = p.as_ref().file_name().unwrap().to_str().unwrap().to_owned();
    let path = p.as_ref().strip_prefix(strip).unwrap();
    let icon = FileIcon::from(&path);
    format!("{} {}", icon, path.display())
}

/// PERF: (urgent) change to use the ignore crate here
fn tree<P: AsRef<Path>>(p: P) -> std::io::Result<Tree<String>> {
    let result = std::fs::read_dir(&p)?
        .filter_map(std::result::Result::ok)
        .fold(
            Tree::new(label(
                p.as_ref(),
                p.as_ref().parent().unwrap().to_str().unwrap(),
            )),
            |mut root, entry| {
                let m = entry.metadata().unwrap();
                if m.is_dir() {
                    root.push(tree(entry.path()).unwrap());
                } else {
                    root.push(Tree::new(label(
                        entry.path(),
                        p.as_ref().to_str().unwrap(),
                    )));
                }
                root
            },
        );
    Ok(result)
}
