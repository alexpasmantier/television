use crate::channels::OnAir;
use crate::entry::{Entry, PreviewType};
use std::path::{Path, PathBuf};

use devicons::FileIcon;
use television_fuzzy::{
    matcher::{config::Config, injector::Injector},
    Matcher,
};
use television_utils::{
    git::{discover_repositories, get_branches},
    strings::preprocess_line,
};
use tokio::task::JoinHandle;
use tracing::debug;

pub struct Channel {
    matcher: Matcher<Branch>,
    icon: FileIcon,
    crawl_handle: JoinHandle<()>,
}

impl Channel {
    pub fn new<P>(repo_paths: Vec<P>) -> Self
    where
        P: AsRef<Path> + Send,
    {
        let matcher = Matcher::new(Config::default().match_paths(true));
        let injector = matcher.injector();
        let paths: Vec<PathBuf> = repo_paths
            .iter()
            .map(|p| p.as_ref().to_path_buf())
            .collect();

        let collect_handle =
            tokio::spawn(load_branches_for_repo_paths(paths, injector));

        Channel {
            matcher,
            icon: FileIcon::from("git"),
            crawl_handle: collect_handle,
        }
    }
}

impl Default for Channel {
    fn default() -> Self {
        let current_dir = std::env::current_dir().unwrap();
        Self::new(vec![current_dir])
    }
}

#[derive(Clone)]
struct Branch {
    name: String,
    is_head: bool,
    is_remote: bool,
    repo_name: String,
}

impl Branch {
    fn new(
        name: String,
        is_head: bool,
        is_remote: bool,
        repo_name: String,
    ) -> Self {
        Self {
            name,
            is_head,
            is_remote,
            repo_name,
        }
    }
}

async fn load_branches_for_repo_paths(
    paths: Vec<impl AsRef<Path>>,
    injector: Injector<Branch>,
) {
    let resolved_repositories = discover_repositories(paths);
    for repo in &resolved_repositories {
        if let Ok(branches) = get_branches(repo) {
            for branch in &branches {
                let branch_name = branch.name.clone();
                let repo_name = {
                    let p = repo.path();
                    let file_name = if p.ends_with(".git") {
                        p.parent().unwrap().file_name()
                    } else {
                        p.file_name()
                    };
                    file_name.unwrap().to_string_lossy().to_string()
                };
                injector.push(
                    Branch::new(
                        preprocess_line(&branch_name),
                        branch.is_head,
                        branch.is_remote,
                        preprocess_line(&repo_name),
                    ),
                    |e, cols| {
                        cols[0] = e.name.clone().into();
                    },
                );
            }
        }
    }
}

impl OnAir for Channel {
    fn find(&mut self, pattern: &str) {
        self.matcher.find(pattern);
    }

    fn results(&mut self, num_entries: u32, offset: u32) -> Vec<Entry> {
        self.matcher.tick();
        self.matcher
            .results(num_entries, offset)
            .into_iter()
            .map(|item| {
                let path = item.matched_string;
                Entry::new(path.clone(), PreviewType::Directory)
                    .with_name_match_ranges(item.match_indices)
                    .with_icon(self.icon)
            })
            .collect()
    }

    fn get_result(&self, index: u32) -> Option<Entry> {
        self.matcher.get_result(index).map(|item| {
            let branch_name = item.matched_string;
            Entry::new(branch_name.clone(), PreviewType::Basic)
                .with_icon(self.icon)
                .with_display_name(format!(
                    "{}/{}",
                    item.inner.repo_name, branch_name
                ))
        })
    }

    fn result_count(&self) -> u32 {
        self.matcher.matched_item_count
    }

    fn total_count(&self) -> u32 {
        self.matcher.total_item_count
    }

    fn running(&self) -> bool {
        self.matcher.status.running
    }

    fn shutdown(&self) {
        debug!("Shutting down git repos channel");
        self.crawl_handle.abort();
    }
}
