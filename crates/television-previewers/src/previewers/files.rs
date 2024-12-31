use color_eyre::Result;
use parking_lot::Mutex;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};

use syntect::{highlighting::Theme, parsing::SyntaxSet};
use tracing::{debug, warn};

use super::cache::PreviewCache;
use crate::previewers::{meta, Preview, PreviewContent};
use television_channels::entry;
use television_utils::{
    files::{get_file_size, FileType},
    strings::preprocess_line,
    syntax::{self, load_highlighting_assets, HighlightingAssetsExt},
};

#[derive(Debug, Default)]
pub struct FilePreviewer {
    cache: Arc<Mutex<PreviewCache>>,
    pub syntax_set: Arc<SyntaxSet>,
    pub syntax_theme: Arc<Theme>,
    concurrent_preview_tasks: Arc<AtomicU8>,
    last_previewed: Arc<Mutex<Arc<Preview>>>,
    in_flight_previews: Arc<Mutex<HashSet<String>>>,
}

#[derive(Debug, Clone, Default)]
pub struct FilePreviewerConfig {
    pub theme: String,
}

impl FilePreviewerConfig {
    pub fn new(theme: String) -> Self {
        FilePreviewerConfig { theme }
    }
}

/// The maximum file size that we will try to preview.
/// 4 MB
const MAX_FILE_SIZE: u64 = 4 * 1024 * 1024;

const MAX_CONCURRENT_PREVIEW_TASKS: u8 = 3;

const BAT_THEME_ENV_VAR: &str = "BAT_THEME";

impl FilePreviewer {
    pub fn new(config: Option<FilePreviewerConfig>) -> Self {
        let hl_assets = load_highlighting_assets();
        let syntax_set = hl_assets.get_syntax_set().unwrap().clone();

        let theme_name = match std::env::var(BAT_THEME_ENV_VAR) {
            Ok(t) => t,
            Err(_) => match config {
                Some(c) => c.theme,
                // this will error and default back nicely
                None => "unknown".to_string(),
            },
        };

        let theme = hl_assets.get_theme_no_output(&theme_name).clone();

        FilePreviewer {
            cache: Arc::new(Mutex::new(PreviewCache::default())),
            syntax_set: Arc::new(syntax_set),
            syntax_theme: Arc::new(theme),
            concurrent_preview_tasks: Arc::new(AtomicU8::new(0)),
            last_previewed: Arc::new(Mutex::new(Arc::new(
                Preview::default().stale(),
            ))),
            in_flight_previews: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Get a preview for a file entry.
    ///
    /// # Panics
    /// Panics if seeking to the start of the file fails.
    pub fn preview(&mut self, entry: &entry::Entry) -> Arc<Preview> {
        // do we have a preview in cache for that entry?
        if let Some(preview) = self.cache.lock().get(&entry.name) {
            return preview;
        }
        debug!("Preview cache miss for {:?}", entry.name);

        // are we already computing a preview in the background for that entry?
        if self.in_flight_previews.lock().contains(&entry.name) {
            debug!("Preview already in flight for {:?}", entry.name);
            return self.last_previewed.lock().clone();
        }

        if self.concurrent_preview_tasks.load(Ordering::Relaxed)
            < MAX_CONCURRENT_PREVIEW_TASKS
        {
            self.in_flight_previews.lock().insert(entry.name.clone());
            self.concurrent_preview_tasks
                .fetch_add(1, Ordering::Relaxed);
            let cache = self.cache.clone();
            let entry_c = entry.clone();
            let syntax_set = self.syntax_set.clone();
            let syntax_theme = self.syntax_theme.clone();
            let concurrent_tasks = self.concurrent_preview_tasks.clone();
            let last_previewed = self.last_previewed.clone();
            let in_flight_previews = self.in_flight_previews.clone();
            tokio::spawn(async move {
                try_preview(
                    &entry_c,
                    &cache,
                    &syntax_set,
                    &syntax_theme,
                    &concurrent_tasks,
                    &last_previewed,
                    &in_flight_previews,
                );
            });
        }

        self.last_previewed.lock().clone()
    }

    #[allow(dead_code)]
    fn cache_preview(&mut self, key: String, preview: Arc<Preview>) {
        self.cache.lock().insert(key, &preview);
    }
}

pub fn try_preview(
    entry: &entry::Entry,
    cache: &Arc<Mutex<PreviewCache>>,
    syntax_set: &Arc<SyntaxSet>,
    syntax_theme: &Arc<Theme>,
    concurrent_tasks: &Arc<AtomicU8>,
    last_previewed: &Arc<Mutex<Arc<Preview>>>,
    in_flight_previews: &Arc<Mutex<HashSet<String>>>,
) {
    debug!("Computing preview for {:?}", entry.name);
    let path = PathBuf::from(&entry.name);
    // check file size
    if get_file_size(&path).map_or(false, |s| s > MAX_FILE_SIZE) {
        debug!("File too large: {:?}", entry.name);
        let preview = meta::file_too_large(&entry.name);
        cache.lock().insert(entry.name.clone(), &preview);
    }

    if matches!(FileType::from(&path), FileType::Text) {
        debug!("File is text-based: {:?}", entry.name);
        match File::open(path) {
            Ok(file) => {
                // compute the highlighted version in the background
                let mut reader = BufReader::new(file);
                reader.seek(std::io::SeekFrom::Start(0)).unwrap();
                let preview = compute_highlighted_text_preview(
                    entry,
                    reader,
                    syntax_set,
                    syntax_theme,
                );
                cache.lock().insert(entry.name.clone(), &preview);
                in_flight_previews.lock().remove(&entry.name);
                let mut tp = last_previewed.lock();
                *tp = preview.stale().into();
            }
            Err(e) => {
                warn!("Error opening file: {:?}", e);
                let p = meta::not_supported(&entry.name);
                cache.lock().insert(entry.name.clone(), &p);
            }
        }
    } else {
        debug!("File isn't text-based: {:?}", entry.name);
        let preview = meta::not_supported(&entry.name);
        cache.lock().insert(entry.name.clone(), &preview);
    }
    concurrent_tasks.fetch_sub(1, Ordering::Relaxed);
}

fn compute_highlighted_text_preview(
    entry: &entry::Entry,
    reader: BufReader<File>,
    syntax_set: &SyntaxSet,
    syntax_theme: &Theme,
) -> Arc<Preview> {
    debug!(
        "Computing highlights in the background for {:?}",
        entry.name
    );
    let lines: Vec<String> = reader
        .lines()
        .map_while(Result::ok)
        // we need to add a newline here because sublime syntaxes expect one
        // to be present at the end of each line
        .map(|line| preprocess_line(&line).0 + "\n")
        .collect();

    match syntax::compute_highlights_for_path(
        &PathBuf::from(&entry.name),
        lines,
        syntax_set,
        syntax_theme,
    ) {
        Ok(highlighted_lines) => {
            debug!("Successfully computed highlights for {:?}", entry.name);
            Arc::new(Preview::new(
                entry.name.clone(),
                PreviewContent::SyntectHighlightedText(highlighted_lines),
                entry.icon,
                false,
            ))
        }
        Err(e) => {
            warn!("Error computing highlights: {:?}", e);
            meta::not_supported(&entry.name)
        }
    }
}

/// This should be enough for most terminal sizes
const TEMP_PLAIN_TEXT_PREVIEW_HEIGHT: usize = 200;

#[allow(dead_code)]
fn plain_text_preview(title: &str, reader: BufReader<&File>) -> Arc<Preview> {
    debug!("Creating plain text preview for {:?}", title);
    let mut lines = Vec::with_capacity(TEMP_PLAIN_TEXT_PREVIEW_HEIGHT);
    // PERF: instead of using lines(), maybe check for the length of the first line instead and
    // truncate accordingly (since this is just a temp preview)
    for maybe_line in reader.lines() {
        match maybe_line {
            Ok(line) => lines.push(preprocess_line(&line).0),
            Err(e) => {
                warn!("Error reading file: {:?}", e);
                return meta::not_supported(title);
            }
        }
        if lines.len() >= TEMP_PLAIN_TEXT_PREVIEW_HEIGHT {
            break;
        }
    }
    Arc::new(Preview::new(
        title.to_string(),
        PreviewContent::PlainText(lines),
        None,
        false,
    ))
}
