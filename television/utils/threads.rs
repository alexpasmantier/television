use std::num::NonZeroUsize;

/// Get the number of threads to use by default.
///
/// This will use the number of available threads if possible, but will default to 1 if the number
/// of available threads cannot be determined. It will also never use more than 32 threads to avoid
/// startup overhead.
pub fn default_num_threads() -> usize {
    // default to 1 thread if we can't determine the number of available threads
    let default = NonZeroUsize::MIN;
    // never use more than 32 threads to avoid startup overhead
    let limit = NonZeroUsize::new(32).unwrap();

    std::thread::available_parallelism()
        .unwrap_or(default)
        .min(limit)
        .get()
}
