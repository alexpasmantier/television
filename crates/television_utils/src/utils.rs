use std::num::NonZeroUsize;

pub mod files;
pub mod indices;
pub mod strings;
pub mod syntax;

/// Heuristic to determine if stdin is readable.
///
/// This is used to determine if we should use the stdin channel.
pub fn is_readable_stdin() -> bool {
    use std::io::IsTerminal;

    #[cfg(unix)]
    fn imp() -> bool {
        use std::{
            fs::File,
            os::{fd::AsFd, unix::fs::FileTypeExt},
        };

        let stdin = std::io::stdin();
        let Ok(fd) = stdin.as_fd().try_clone_to_owned() else {
            return false;
        };
        let file = File::from(fd);
        let Ok(md) = file.metadata() else {
            return false;
        };
        let ft = md.file_type();
        let is_file = ft.is_file();
        let is_fifo = ft.is_fifo();
        let is_socket = ft.is_socket();
        is_file || is_fifo || is_socket
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

/// Get the number of threads to use by default.
///
/// This will use the number of available threads if possible, but will default to 1 if the number
/// of available threads cannot be determined. It will also never use more than 32 threads to avoid
/// startup overhead.
pub fn default_num_threads() -> NonZeroUsize {
    // default to 1 thread if we can't determine the number of available threads
    let default = NonZeroUsize::MIN;
    // never use more than 32 threads to avoid startup overhead
    let limit = NonZeroUsize::new(32).unwrap();

    std::thread::available_parallelism()
        .unwrap_or(default)
        .min(limit)
}
