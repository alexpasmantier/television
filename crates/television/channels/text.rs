use devicons::FileIcon;
use nucleo::{
    pattern::{CaseMatching, Normalization},
    Config, Injector, Nucleo,
};
use std::{
    fs::File,
    io::{BufRead, Read, Seek},
    path::{Path, PathBuf},
    sync::{atomic::AtomicUsize, Arc},
};

use tracing::{debug, info};

use super::OnAir;
use crate::previewers::PreviewType;
use crate::utils::{
    files::{is_not_text, walk_builder, DEFAULT_NUM_THREADS},
    strings::preprocess_line,
};
use crate::{
    entry::Entry, utils::strings::proportion_of_printable_ascii_characters,
};
use crate::{fuzzy::MATCHER, utils::strings::PRINTABLE_ASCII_THRESHOLD};

#[derive(Debug)]
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
    matcher: Nucleo<CandidateLine>,
    last_pattern: String,
    result_count: u32,
    total_count: u32,
    running: bool,
    crawl_handle: tokio::task::JoinHandle<()>,
}

impl Channel {
    pub fn new(working_dir: &Path) -> Self {
        let matcher = Nucleo::new(Config::DEFAULT, Arc::new(|| {}), None, 1);
        // start loading files in the background
        let crawl_handle = tokio::spawn(load_candidates(
            working_dir.to_path_buf(),
            matcher.injector(),
        ));
        Channel {
            matcher,
            last_pattern: String::new(),
            result_count: 0,
            total_count: 0,
            running: false,
            crawl_handle,
        }
    }

    const MATCHER_TICK_TIMEOUT: u64 = 2;
}

impl Default for Channel {
    fn default() -> Self {
        Self::new(&std::env::current_dir().unwrap())
    }
}

impl OnAir for Channel {
    fn find(&mut self, pattern: &str) {
        if pattern != self.last_pattern {
            self.matcher.pattern.reparse(
                0,
                pattern,
                CaseMatching::Smart,
                Normalization::Smart,
                pattern.starts_with(&self.last_pattern),
            );
            self.last_pattern = pattern.to_string();
        }
    }

    fn result_count(&self) -> u32 {
        self.result_count
    }

    fn total_count(&self) -> u32 {
        self.total_count
    }

    fn results(&mut self, num_entries: u32, offset: u32) -> Vec<Entry> {
        let status = self.matcher.tick(Self::MATCHER_TICK_TIMEOUT);
        let snapshot = self.matcher.snapshot();
        if status.changed {
            self.result_count = snapshot.matched_item_count();
            self.total_count = snapshot.item_count();
        }
        self.running = status.running;
        let mut indices = Vec::new();
        let mut matcher = MATCHER.lock();

        snapshot
            .matched_items(
                offset
                    ..(num_entries + offset)
                        .min(snapshot.matched_item_count()),
            )
            .map(move |item| {
                snapshot.pattern().column_pattern(0).indices(
                    item.matcher_columns[0].slice(..),
                    &mut matcher,
                    &mut indices,
                );
                indices.sort_unstable();
                indices.dedup();
                let indices = indices.drain(..);

                let line = item.matcher_columns[0].to_string();
                let display_path =
                    item.data.path.to_string_lossy().to_string();
                Entry::new(
                    display_path.clone() + &item.data.line_number.to_string(),
                    PreviewType::Files,
                )
                .with_display_name(display_path)
                .with_value(line)
                .with_value_match_ranges(indices.map(|i| (i, i + 1)).collect())
                .with_icon(FileIcon::from(item.data.path.as_path()))
                .with_line_number(item.data.line_number)
            })
            .collect()
    }

    fn get_result(&self, index: u32) -> Option<Entry> {
        let snapshot = self.matcher.snapshot();
        snapshot.get_matched_item(index).map(|item| {
            let display_path = item.data.path.to_string_lossy().to_string();
            Entry::new(display_path.clone(), PreviewType::Files)
                .with_display_name(
                    display_path.clone()
                        + ":"
                        + &item.data.line_number.to_string(),
                )
                .with_icon(FileIcon::from(item.data.path.as_path()))
                .with_line_number(item.data.line_number)
        })
    }

    fn running(&self) -> bool {
        self.running
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
/// so this should take around 100 x `5_000_000` = 500MB of memory.
const MAX_IN_MEMORY_LINES: usize = 5_000_000;

#[allow(clippy::unused_async)]
async fn load_candidates(path: PathBuf, injector: Injector<CandidateLine>) {
    let current_dir = std::env::current_dir().unwrap();
    let walker =
        walk_builder(&path, *DEFAULT_NUM_THREADS, None, None).build_parallel();

    let lines_in_mem = Arc::new(AtomicUsize::new(0));

    walker.run(|| {
        let injector = injector.clone();
        let current_dir = current_dir.clone();
        let lines_in_mem = lines_in_mem.clone();
        Box::new(move |result| {
            if lines_in_mem.load(std::sync::atomic::Ordering::Relaxed) > MAX_IN_MEMORY_LINES {
                return ignore::WalkState::Quit;
            }
            if let Ok(entry) = result {
                if entry.file_type().unwrap().is_file() {
                    if let Ok(m) = entry.metadata() {
                        if m.len() > MAX_FILE_SIZE {
                            return ignore::WalkState::Continue;
                        }
                    }
                    // iterate over the lines of the file
                    match File::open(entry.path()) {
                        Ok(file) => {
                            // is the file a text-based file?
                            let mut reader = std::io::BufReader::new(&file);
                            let mut buffer = [0u8; 128];
                            match reader.read(&mut buffer) {
                                Ok(bytes_read) => {
                                    if (bytes_read == 0)
                                        || is_not_text(&buffer)
                                            .unwrap_or(false)
                                        || proportion_of_printable_ascii_characters(&buffer)
                                            < PRINTABLE_ASCII_THRESHOLD
                                    {
                                        return ignore::WalkState::Continue;
                                    }
                                    reader
                                        .seek(std::io::SeekFrom::Start(0))
                                        .unwrap();
                                }
                                Err(_) => {
                                    return ignore::WalkState::Continue;
                                }
                            }
                            // read the lines of the file
                            let mut line_number = 0;
                            for maybe_line in reader.lines() {
                                match maybe_line {
                                    Ok(l) => {
                                        line_number += 1;
                                        let line = preprocess_line(&l);
                                        if line.is_empty() {
                                            debug!("Empty line");
                                            continue;
                                        }
                                        let candidate = CandidateLine::new(
                                            entry
                                                .path()
                                                .strip_prefix(&current_dir)
                                                .unwrap()
                                                .to_path_buf(),
                                            line,
                                            line_number,
                                        );
                                        let _ = injector.push(
                                            candidate,
                                            |c, cols| {
                                                cols[0] =
                                                    c.line.clone().into();
                                            },
                                        );
                                        lines_in_mem.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                    }
                                    Err(e) => {
                                        info!("Error reading line: {:?}", e);
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            info!("Error opening file: {:?}", e);
                        }
                    }
                }
            }
            ignore::WalkState::Continue
        })
    });
}
