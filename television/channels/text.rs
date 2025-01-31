use super::{OnAir, TelevisionChannel};
use crate::channels::entry::{Entry, PreviewType};
use crate::matcher::{config::Config, injector::Injector, Matcher};
use crate::utils::files::{walk_builder, DEFAULT_NUM_THREADS};
use crate::utils::strings::{
    proportion_of_printable_ascii_characters, PRINTABLE_ASCII_THRESHOLD,
};
use devicons::FileIcon;
use ignore::WalkState;
use rustc_hash::{FxBuildHasher, FxHashSet};
use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, Read, Seek},
    path::{Path, PathBuf},
    sync::{atomic::AtomicUsize, Arc},
};
use tracing::{debug, trace, warn};

#[derive(Debug, Clone)]
struct CandidateLine {
    path: PathBuf,
    line: String,
    line_number: usize,
}

impl CandidateLine {
    fn new(path: PathBuf, line: String, line_number: usize) -> Self {
        CandidateLine {
            path,
            line,
            line_number,
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct Channel {
    matcher: Matcher<CandidateLine>,
    crawl_handle: tokio::task::JoinHandle<()>,
    selected_entries: FxHashSet<Entry>,
}

impl Channel {
    pub fn new(directories: Vec<PathBuf>) -> Self {
        let matcher = Matcher::new(Config::default());
        // start loading files in the background
        let crawl_handle = tokio::spawn(crawl_for_candidates(
            directories,
            matcher.injector(),
        ));
        Channel {
            matcher,
            crawl_handle,
            selected_entries: HashSet::with_hasher(FxBuildHasher),
        }
    }

    fn from_file_paths(file_paths: Vec<PathBuf>) -> Self {
        let matcher = Matcher::new(Config::default());
        let injector = matcher.injector();
        let current_dir = std::env::current_dir().unwrap();
        let crawl_handle = tokio::spawn(async move {
            let mut lines_in_mem = 0;
            for path in file_paths {
                if lines_in_mem > MAX_LINES_IN_MEM {
                    break;
                }
                if let Some(injected_lines) =
                    try_inject_lines(&injector, &current_dir, &path)
                {
                    lines_in_mem += injected_lines;
                }
            }
        });

        Channel {
            matcher,
            crawl_handle,
            selected_entries: HashSet::with_hasher(FxBuildHasher),
        }
    }

    fn from_text_entries(entries: Vec<Entry>) -> Self {
        let matcher = Matcher::new(Config::default());
        let injector = matcher.injector();
        let load_handle = tokio::spawn(async move {
            for entry in entries.into_iter().take(MAX_LINES_IN_MEM) {
                let v = entry.value.unwrap();
                injector.push(
                    CandidateLine::new(
                        entry.name.into(),
                        v,
                        entry.line_number.unwrap(),
                    ),
                    |e, cols| {
                        cols[0] = e.line.clone().into();
                    },
                );
            }
        });

        Channel {
            matcher,
            crawl_handle: load_handle,
            selected_entries: HashSet::with_hasher(FxBuildHasher),
        }
    }
}

impl Default for Channel {
    fn default() -> Self {
        Self::new(vec![std::env::current_dir().unwrap()])
    }
}

/// Since we're limiting the number of lines in memory, it makes sense to also limit the number of files
/// we're willing to search in when piping from the `Files` channel.
/// This prevents blocking the UI for too long when piping from a channel with a lot of files.
///
/// This should be calculated based on the number of lines we're willing to keep in memory:
/// `MAX_LINES_IN_MEM / 100` (assuming 100 lines per file on average).
const MAX_PIPED_FILES: usize = MAX_LINES_IN_MEM / 200;

impl From<&mut TelevisionChannel> for Channel {
    fn from(value: &mut TelevisionChannel) -> Self {
        match value {
            c @ TelevisionChannel::Files(_) => {
                let entries = if c.selected_entries().is_empty() {
                    c.results(
                        c.result_count().min(
                            u32::try_from(MAX_PIPED_FILES).unwrap_or(u32::MAX),
                        ),
                        0,
                    )
                } else {
                    c.selected_entries().iter().cloned().collect()
                };
                Self::from_file_paths(
                    entries
                        .iter()
                        .flat_map(|entry| {
                            PathBuf::from(entry.name.clone()).canonicalize()
                        })
                        .collect(),
                )
            }
            c @ TelevisionChannel::GitRepos(_) => {
                let entries = if c.selected_entries().is_empty() {
                    c.results(c.result_count(), 0)
                } else {
                    c.selected_entries().iter().cloned().collect()
                };
                Self::new(
                    entries
                        .iter()
                        .flat_map(|entry| {
                            PathBuf::from(entry.name.clone()).canonicalize()
                        })
                        .collect(),
                )
            }
            c @ TelevisionChannel::Text(_) => {
                let entries = if c.selected_entries().is_empty() {
                    c.results(c.result_count(), 0)
                } else {
                    c.selected_entries().iter().cloned().collect()
                };
                Self::from_text_entries(entries)
            }
            c @ TelevisionChannel::Dirs(_) => {
                let entries = if c.selected_entries().is_empty() {
                    c.results(c.result_count(), 0)
                } else {
                    c.selected_entries().iter().cloned().collect()
                };
                Self::new(
                    entries
                        .iter()
                        .map(|entry| PathBuf::from(&entry.name))
                        .collect(),
                )
            }
            _ => unreachable!(),
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
                let line = item.matched_string;
                let display_path =
                    item.inner.path.to_string_lossy().to_string();
                Entry::new(display_path, PreviewType::Files)
                    .with_value(line)
                    .with_value_match_ranges(&item.match_indices)
                    .with_icon(FileIcon::from(item.inner.path.as_path()))
                    .with_line_number(item.inner.line_number)
            })
            .collect()
    }

    fn get_result(&self, index: u32) -> Option<Entry> {
        self.matcher.get_result(index).map(|item| {
            let display_path = item.inner.path.to_string_lossy().to_string();
            Entry::new(display_path, PreviewType::Files)
                .with_icon(FileIcon::from(item.inner.path.as_path()))
                .with_line_number(item.inner.line_number)
        })
    }

    fn selected_entries(&self) -> &FxHashSet<Entry> {
        &self.selected_entries
    }

    fn toggle_selection(&mut self, entry: &Entry) {
        if self.selected_entries.contains(entry) {
            self.selected_entries.remove(entry);
        } else {
            self.selected_entries.insert(entry.clone());
        }
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
        self.crawl_handle.abort();
    }
}

/// The maximum file size we're willing to search in.
///
/// This is to prevent taking humongous amounts of memory when searching in
/// a lot of files (e.g. starting tv in $HOME).
const MAX_FILE_SIZE: u64 = 4 * 1024 * 1024;

/// The maximum number of lines we're willing to keep in memory.
///
/// TODO: this should be configurable by the user depending on the amount of
/// memory they have/are willing to use.
///
/// This is to prevent taking humongous amounts of memory when searching in
/// a lot of files (e.g. starting tv in $HOME).
///
/// This is a soft limit, we might go over it a bit.
///
/// A typical line should take somewhere around 100 bytes in memory (for utf8 english text),
/// so this should take around 100 x `10_000_000` = 1GB of memory.
const MAX_LINES_IN_MEM: usize = 10_000_000;

#[allow(clippy::unused_async)]
async fn crawl_for_candidates(
    directories: Vec<PathBuf>,
    injector: Injector<CandidateLine>,
) {
    if directories.is_empty() {
        return;
    }
    let current_dir = std::env::current_dir().unwrap();
    let mut walker =
        walk_builder(&directories[0], *DEFAULT_NUM_THREADS, None, None);
    directories[1..].iter().for_each(|path| {
        walker.add(path);
    });

    let lines_in_mem = Arc::new(AtomicUsize::new(0));

    walker.build_parallel().run(|| {
        let injector = injector.clone();
        let current_dir = current_dir.clone();
        let lines_in_mem = lines_in_mem.clone();
        Box::new(move |result| {
            if lines_in_mem.load(std::sync::atomic::Ordering::Relaxed)
                > MAX_LINES_IN_MEM
            {
                return WalkState::Quit;
            }
            if let Ok(entry) = result {
                if entry.file_type().unwrap().is_file() {
                    if let Ok(m) = entry.metadata() {
                        if m.len() > MAX_FILE_SIZE {
                            return WalkState::Continue;
                        }
                    }
                    // try to inject the lines of the file
                    if let Some(injected_lines) =
                        try_inject_lines(&injector, &current_dir, entry.path())
                    {
                        lines_in_mem.fetch_add(
                            injected_lines,
                            std::sync::atomic::Ordering::Relaxed,
                        );
                    }
                }
            }
            WalkState::Continue
        })
    });
}

fn try_inject_lines(
    injector: &Injector<CandidateLine>,
    current_dir: &PathBuf,
    path: &Path,
) -> Option<usize> {
    match File::open(path) {
        Ok(file) => {
            // is the file a text-based file?
            let mut reader = std::io::BufReader::new(&file);
            let mut buffer = [0u8; 128];
            match reader.read(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read == 0
                        || proportion_of_printable_ascii_characters(
                            &buffer[..bytes_read],
                        ) < PRINTABLE_ASCII_THRESHOLD
                    {
                        debug!("Skipping non-text file {:?}", path);
                        return None;
                    }
                    reader.seek(std::io::SeekFrom::Start(0)).unwrap();
                }
                Err(e) => {
                    warn!("Error reading file {:?}: {:?}", path, e);
                    return None;
                }
            }
            // read the lines of the file
            let mut line_number = 0;
            let mut injected_lines = 0;
            for maybe_line in reader.lines() {
                match maybe_line {
                    Ok(l) => {
                        line_number += 1;
                        if l.is_empty() {
                            trace!("Empty line");
                            continue;
                        }
                        let candidate = CandidateLine::new(
                            path.strip_prefix(current_dir)
                                .unwrap_or(path)
                                .to_path_buf(),
                            l.clone(),
                            line_number,
                        );
                        let () = injector.push(candidate, |e, cols| {
                            cols[0] = e.line.clone().into();
                        });
                        injected_lines += 1;
                    }
                    Err(e) => {
                        warn!("Error reading line: {:?}", e);
                        break;
                    }
                }
            }
            Some(injected_lines)
        }
        Err(e) => {
            warn!("Error opening file {:?}: {:?}", path, e);
            None
        }
    }
}
