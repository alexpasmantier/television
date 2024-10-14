use devicons::FileIcon;
use nucleo::{
    pattern::{CaseMatching, Normalization},
    Config, Injector, Nucleo,
};
use std::{
    fs::File,
    io::{BufRead, Read, Seek},
    path::PathBuf,
    sync::Arc,
};

use tracing::{debug, info};

use super::TelevisionChannel;
use crate::entry::Entry;
use crate::fuzzy::MATCHER;
use crate::previewers::PreviewType;
use crate::utils::{
    files::{is_not_text, is_valid_utf8, walk_builder, DEFAULT_NUM_THREADS},
    strings::preprocess_line,
};

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
}

impl Channel {
    pub fn new() -> Self {
        let matcher = Nucleo::new(Config::DEFAULT, Arc::new(|| {}), None, 1);
        // start loading files in the background
        tokio::spawn(load_candidates(
            std::env::current_dir().unwrap(),
            matcher.injector(),
        ));
        Channel {
            matcher,
            last_pattern: String::new(),
            result_count: 0,
            total_count: 0,
        }
    }

    const MATCHER_TICK_TIMEOUT: u64 = 10;
}

impl TelevisionChannel for Channel {
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
                .with_line_number(item.data.line_number)
        })
    }
}

#[allow(clippy::unused_async)]
async fn load_candidates(path: PathBuf, injector: Injector<CandidateLine>) {
    let current_dir = std::env::current_dir().unwrap();
    let walker = walk_builder(&path, *DEFAULT_NUM_THREADS).build_parallel();

    walker.run(|| {
        let injector = injector.clone();
        let current_dir = current_dir.clone();
        Box::new(move |result| {
            if let Ok(entry) = result {
                if entry.file_type().unwrap().is_file() {
                    // iterate over the lines of the file
                    match File::open(entry.path()) {
                        Ok(file) => {
                            let mut reader = std::io::BufReader::new(&file);
                            let mut buffer = [0u8; 128];
                            match reader.read(&mut buffer) {
                                Ok(bytes_read) => {
                                    if (bytes_read == 0)
                                        || is_not_text(&buffer)
                                            .unwrap_or(false)
                                        || !is_valid_utf8(&buffer)
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
                                        // Send the line via the async channel
                                        let _ = injector.push(
                                            candidate,
                                            |c, cols| {
                                                cols[0] =
                                                    c.line.clone().into();
                                            },
                                        );
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
