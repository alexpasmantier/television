use crate::utils::files::{read_into_lines_capped, ReadResult};
use crate::utils::syntax::HighlightedLines;
use image::ImageReader;
use parking_lot::Mutex;
use ratatui::layout::Rect;
use rustc_hash::{FxBuildHasher, FxHashSet};
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

use crate::channels::entry;
use crate::preview::cache::PreviewCache;
use crate::preview::{previewers::meta, Preview, PreviewContent};
use crate::utils::{
    files::FileType,
    image::Image,
    strings::preprocess_line,
    syntax::{self, load_highlighting_assets, HighlightingAssetsExt},
};

#[derive(Debug, Default)]
pub struct FilePreviewer {
    cache: Arc<Mutex<PreviewCache>>,
    pub syntax_set: Arc<SyntaxSet>,
    pub syntax_theme: Arc<Theme>,
    concurrent_preview_tasks: Arc<AtomicU8>,
    in_flight_previews: Arc<Mutex<FxHashSet<String>>>,
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
            in_flight_previews: Arc::new(Mutex::new(HashSet::with_hasher(
                FxBuildHasher,
            ))),
        }
    }

    pub fn cached(&self, entry: &entry::Entry) -> Option<Arc<Preview>> {
        self.cache.lock().get(&entry.name)
    }

    pub fn preview(
        &mut self,
        entry: &entry::Entry,
        preview_window: Option<Rect>,
    ) -> Option<Arc<Preview>> {
        if let Some(preview) = self.cached(entry) {
            debug!("Preview cache hit for {:?}", entry.name);
            if preview.partial_offset.is_some() {
                // preview is partial, spawn a task to compute the next chunk
                // and return the partial preview
                debug!("Spawning partial preview task for {:?}", entry.name);
                self.handle_preview_request(
                    entry,
                    Some(preview.clone()),
                    preview_window,
                );
            }
            Some(preview)
        } else {
            // preview is not in cache, spawn a task to compute the preview
            debug!("Preview cache miss for {:?}", entry.name);
            self.handle_preview_request(entry, None, preview_window);
            None
        }
    }

    pub fn handle_preview_request(
        &mut self,
        entry: &entry::Entry,
        partial_preview: Option<Arc<Preview>>,
        preview_window: Option<Rect>,
    ) {
        if self.in_flight_previews.lock().contains(&entry.name) {
            debug!("Preview already in flight for {:?}", entry.name);
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
            let in_flight_previews = self.in_flight_previews.clone();
            tokio::spawn(async move {
                try_preview(
                    &entry_c,
                    partial_preview,
                    &cache,
                    &syntax_set,
                    &syntax_theme,
                    &concurrent_tasks,
                    &in_flight_previews,
                    preview_window,
                );
            });
        }
    }

    #[allow(dead_code)]
    fn cache_preview(&mut self, key: String, preview: &Arc<Preview>) {
        self.cache.lock().insert(key, preview);
    }
}

/// The size of the buffer used to read the file in bytes.
/// This ends up being the max size of partial previews.
const PARTIAL_BUFREAD_SIZE: usize = 64 * 1024;

#[allow(clippy::too_many_arguments)]
pub fn try_preview(
    entry: &entry::Entry,
    partial_preview: Option<Arc<Preview>>,
    cache: &Arc<Mutex<PreviewCache>>,
    syntax_set: &Arc<SyntaxSet>,
    syntax_theme: &Arc<Theme>,
    concurrent_tasks: &Arc<AtomicU8>,
    in_flight_previews: &Arc<Mutex<FxHashSet<String>>>,
    preview_window: Option<Rect>,
) {
    debug!("Computing preview for {:?}", entry.name);
    let path = PathBuf::from(&entry.name);

    // if we're dealing with a partial preview, no need to re-check for textual content
    if partial_preview.is_some()
        || matches!(FileType::from(&path), FileType::Text)
    {
        debug!("File is text-based: {:?}", entry.name);
        match File::open(path) {
            Ok(mut file) => {
                // if we're dealing with a partial preview, seek to the provided offset
                // and use the previous state to compute the next chunk of the preview
                let cached_lines = if let Some(p) = partial_preview {
                    if let PreviewContent::SyntectHighlightedText(hl) =
                        &p.content
                    {
                        let _ = file.seek(std::io::SeekFrom::Start(
                            // this is always Some in this case
                            p.partial_offset.unwrap() as u64,
                        ));
                        Some(hl.clone())
                    } else {
                        None
                    }
                } else {
                    None
                };
                // compute the highlighted version in the background
                match read_into_lines_capped(file, PARTIAL_BUFREAD_SIZE) {
                    ReadResult::Full(lines) => {
                        if let Some(content) = compute_highlighted_text_preview(
                            entry,
                            &lines
                                .iter()
                                .map(|l| preprocess_line(l).0 + "\n")
                                .collect::<Vec<_>>(),
                            syntax_set,
                            syntax_theme,
                            cached_lines.as_ref(),
                        ) {
                            let total_lines = content.total_lines();
                            let preview = Arc::new(Preview::new(
                                entry.name.clone(),
                                content,
                                entry.icon,
                                None,
                                total_lines,
                            ));
                            cache.lock().insert(entry.name.clone(), &preview);
                        }
                    }
                    ReadResult::Partial(p) => {
                        if let Some(content) = compute_highlighted_text_preview(
                            entry,
                            &p.lines
                                .iter()
                                .map(|l| preprocess_line(l).0 + "\n")
                                .collect::<Vec<_>>(),
                            syntax_set,
                            syntax_theme,
                            cached_lines.as_ref(),
                        ) {
                            let total_lines = content.total_lines();
                            let preview = Arc::new(Preview::new(
                                entry.name.clone(),
                                content,
                                entry.icon,
                                Some(p.bytes_read),
                                total_lines,
                            ));
                            cache.lock().insert(entry.name.clone(), &preview);
                        }
                    }
                    ReadResult::Error(e) => {
                        warn!("Error reading file: {:?}", e);
                        let p = meta::not_supported(&entry.name);
                        cache.lock().insert(entry.name.clone(), &p);
                    }
                }
            }
            Err(e) => {
                warn!("Error opening file: {:?}", e);
                let p = meta::not_supported(&entry.name);
                cache.lock().insert(entry.name.clone(), &p);
            }
        }
    } else if matches!(FileType::from(&path), FileType::Image) {
        debug!("File is an image: {:?}", entry.name);
        let (window_height, window_width) = if let Some(preview_window) =
            preview_window
        {
            // it should be a better way to know the size of the border to remove than this magic number
            let padding_width = 5;
            let padding_height = 3;
            (
                (preview_window.height - padding_height) * 2,
                preview_window.width - padding_width,
            )
        } else {
            warn!("Error opening image, impossible to display without information about the size of the preview window");
            let p = meta::not_supported(&entry.name);
            cache.lock().insert(entry.name.clone(), &p);
            return;
        };
        match ImageReader::open(path).unwrap().decode() {
            Ok(image) => {
                cache.lock().insert(
                    entry.name.clone(),
                    &meta::loading(&format!("Loading {}", entry.name)),
                );
                let image = Image::from_dynamic_image(
                    image,
                    u32::from(window_height),
                    u32::from(window_width),
                );
                let total_lines =
                    image.pixel_grid.len().try_into().unwrap_or(u16::MAX);
                let content = PreviewContent::Image(image);
                let preview = Arc::new(Preview::new(
                    entry.name.clone(),
                    content,
                    entry.icon,
                    None,
                    total_lines,
                ));
                cache.lock().insert(entry.name.clone(), &preview);
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
    in_flight_previews.lock().remove(&entry.name);
}

fn compute_highlighted_text_preview(
    entry: &entry::Entry,
    lines: &[String],
    syntax_set: &SyntaxSet,
    syntax_theme: &Theme,
    previous_lines: Option<&HighlightedLines>,
) -> Option<PreviewContent> {
    debug!(
        "Computing highlights in the background for {:?}",
        entry.name
    );

    match syntax::compute_highlights_incremental(
        &PathBuf::from(&entry.name),
        lines,
        syntax_set,
        syntax_theme,
        previous_lines,
    ) {
        Ok(highlighted_lines) => {
            Some(PreviewContent::SyntectHighlightedText(highlighted_lines))
        }
        Err(e) => {
            warn!("Error computing highlights: {:?}", e);
            None
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
    let total_lines = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    Arc::new(Preview::new(
        title.to_string(),
        PreviewContent::PlainText(lines),
        None,
        None,
        total_lines,
    ))
}
