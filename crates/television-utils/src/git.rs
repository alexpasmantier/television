use color_eyre::{eyre::OptionExt, Result};
use std::path::Path;

use git2::{BranchType, Repository, RepositoryOpenFlags};

pub fn discover_repositories<P>(paths: Vec<P>) -> Vec<Repository>
where
    P: AsRef<Path>,
{
    let resolved_repositories = paths.iter().filter_map(|path| {
        let repo = Repository::open_ext(
            path,
            RepositoryOpenFlags::FROM_ENV,
            Vec::<&Path>::new(),
        );
        match repo {
            Ok(repo) => Some(repo),
            Err(_) => None,
        }
    });
    resolved_repositories.collect()
}

pub struct Branch {
    pub name: String,
    pub is_head: bool,
    pub is_remote: bool,
}

pub fn get_branches(repo: &Repository) -> Result<Vec<Branch>> {
    repo.branches(None)?
        .map(|b| {
            let (branch, branch_type) = b?;
            let branch_name = branch
                .name()?
                .ok_or_eyre("Branch name was not valid utf8")?;
            let is_head = branch.is_head();
            let is_remote = matches!(branch_type, BranchType::Remote);
            Ok(Branch {
                name: branch_name.to_string(),
                is_head,
                is_remote,
            })
        })
        .collect()
}
