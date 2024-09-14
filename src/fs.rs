use anyhow::Result;
use ignore::{types::TypesBuilder, WalkBuilder};

use std::path::{Path, PathBuf};

pub fn walk_builder(
    paths: Vec<&Path>,
    ignored_paths: &[PathBuf],
    n_threads: usize,
    respect_gitignore: bool,
    filter_filetypes: Vec<String>,
) -> WalkBuilder {
    let mut builder = WalkBuilder::new(paths[0]);
    // add all paths to the builder
    paths.iter().skip(1).for_each(|path| {
        builder.add(*path);
    });

    // ft-based filtering
    let mut types_builder = TypesBuilder::new();
    types_builder.add_defaults();
    add_custom_filetypes(&mut types_builder).unwrap();
    filter_filetypes.iter().for_each(|ft| {
        types_builder.select(ft);
    });
    builder.types(types_builder.build().unwrap());

    // path-based filtering
    let ignored_paths = ignored_paths.to_vec();
    builder.filter_entry(move |entry| {
        for ignore in ignored_paths.iter() {
            if entry.path() == ignore {
                return false;
            }
        }
        true
    });

    // .gitignore filtering
    builder.git_ignore(respect_gitignore);

    builder.threads(n_threads);
    builder
}

fn add_custom_filetypes(types_builder: &mut TypesBuilder) -> Result<()> {
    Ok(types_builder.add("pystrict", "*.py")?)
}

// Original code from https://github.com/BurntSushi/ripgrep/blob/e0f1000df67f82ab0e735bad40e9b45b2d774ef0/crates/cli/src/lib.rs#L249
pub fn is_readable_stdin() -> bool {
    use std::io::IsTerminal;

    #[cfg(unix)]
    fn imp() -> bool {
        use std::{
            fs::File,
            os::{fd::AsFd, unix::fs::FileTypeExt},
        };

        let stdin = std::io::stdin();
        let fd = match stdin.as_fd().try_clone_to_owned() {
            Ok(fd) => fd,
            Err(_) => {
                return false;
            }
        };
        let file = File::from(fd);
        let md = match file.metadata() {
            Ok(md) => md,
            Err(_) => {
                return false;
            }
        };
        let ft = md.file_type();
        let is_file = ft.is_file();
        let is_fifo = ft.is_fifo();
        let is_socket = ft.is_socket();
        let is_readable = is_file || is_fifo || is_socket;
        is_readable
    }

    #[cfg(windows)]
    fn imp() -> bool {
        let stdin = winapi_util::HandleRef::stdin();
        let typ = match winapi_util::file::typ(stdin) {
            Ok(typ) => typ,
            Err(err) => {
                log::debug!(
                    "for heuristic stdin detection on Windows, \
                     could not get file type of stdin \
                     (thus assuming stdin is not readable): {err}",
                );
                return false;
            }
        };
        let is_disk = typ.is_disk();
        let is_pipe = typ.is_pipe();
        let is_readable = is_disk || is_pipe;
        log::debug!(
            "for heuristic stdin detection on Windows, \
             found that is_disk={is_disk} and is_pipe={is_pipe}, \
             and thus concluded that is_stdin_readable={is_readable}",
        );
        is_readable
    }

    #[cfg(not(any(unix, windows)))]
    fn imp() -> bool {
        log::debug!("on non-{{Unix,Windows}}, assuming stdin is not readable");
        false
    }

    !std::io::stdin().is_terminal() && imp()
}
