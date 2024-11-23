use color_eyre::Result;
//use image::{ImageReader, Rgb};
//use ratatui_image::picker::Picker;
use parking_lot::Mutex;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek};
use std::path::PathBuf;
use std::sync::Arc;

use syntect::{
    highlighting::{Theme, ThemeSet},
    parsing::SyntaxSet,
};
use tracing::{debug, warn};

use super::cache::PreviewCache;
use crate::previewers::{meta, Preview, PreviewContent};
use television_channels::entry;
use television_utils::files::get_file_size;
use television_utils::files::FileType;
use television_utils::strings::preprocess_line;
use television_utils::syntax::{
    self, load_highlighting_assets, HighlightingAssetsExt,
};

#[derive(Debug, Default)]
pub struct FilePreviewer {
    cache: Arc<Mutex<PreviewCache>>,
    pub syntax_set: Arc<SyntaxSet>,
    pub syntax_theme: Arc<Theme>,
    //image_picker: Arc<Mutex<Picker>>,
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

impl FilePreviewer {
    pub fn new(config: Option<FilePreviewerConfig>) -> Self {
        let hl_assets = load_highlighting_assets();
        let syntax_set = hl_assets.get_syntax_set().unwrap().clone();

        let theme = config.map_or_else(
            || {
                let theme_set = ThemeSet::load_defaults();
                theme_set.themes["base16-ocean.dark"].clone()
            },
            |c| hl_assets.get_theme_no_output(&c.theme).clone(),
        );
        //info!("getting image picker");
        //let image_picker = get_image_picker();
        //info!("got image picker");

        FilePreviewer {
            cache: Arc::new(Mutex::new(PreviewCache::default())),
            syntax_set: Arc::new(syntax_set),
            syntax_theme: Arc::new(theme),
            //image_picker: Arc::new(Mutex::new(image_picker)),
        }
    }

    /// The maximum file size that we will try to preview.
    /// 4 MB
    const MAX_FILE_SIZE: u64 = 4 * 1024 * 1024;

    /// Get a preview for a file entry.
    ///
    /// # Panics
    /// Panics if seeking to the start of the file fails.
    pub fn preview(&mut self, entry: &entry::Entry) -> Arc<Preview> {
        let path_buf = PathBuf::from(&entry.name);

        // do we have a preview in cache for that entry?
        if let Some(preview) = self.cache.lock().get(&entry.name) {
            return preview.clone();
        }
        debug!("No preview in cache for {:?}", entry.name);

        // check file size
        if get_file_size(&path_buf).map_or(false, |s| s > Self::MAX_FILE_SIZE)
        {
            debug!("File too large: {:?}", entry.name);
            let preview = meta::file_too_large(&entry.name);
            self.cache_preview(entry.name.clone(), preview.clone());
            return preview;
        }

        // try to determine file type
        debug!("Computing preview for {:?}", entry.name);
        if let FileType::Text = FileType::from(&path_buf) {
            debug!("File is text-based: {:?}", entry.name);
            match File::open(&path_buf) {
                Ok(file) => {
                    // insert a loading preview into the cache
                    let preview = meta::loading(&entry.name);
                    self.cache_preview(entry.name.clone(), preview.clone());

                    // compute the highlighted version in the background
                    let mut reader = BufReader::new(file);
                    reader.seek(std::io::SeekFrom::Start(0)).unwrap();
                    self.compute_highlighted_text_preview(entry, reader);
                    preview
                }
                Err(e) => {
                    warn!("Error opening file: {:?}", e);
                    let p = meta::not_supported(&entry.name);
                    self.cache_preview(entry.name.clone(), p.clone());
                    p
                }
            }
        } else {
            debug!("File isn't text-based: {:?}", entry.name);
            let preview = meta::not_supported(&entry.name);
            self.cache_preview(entry.name.clone(), preview.clone());
            preview
        }
    }

    //async fn compute_image_preview(&self, entry: &entry::Entry) {
    //    let cache = self.cache.clone();
    //    let picker = self.image_picker.clone();
    //    let entry_c = entry.clone();
    //    tokio::spawn(async move {
    //        info!("Loading image: {:?}", entry_c.name);
    //        if let Ok(dyn_image) =
    //            ImageReader::open(entry_c.name.clone()).unwrap().decode()
    //        {
    //            let image = picker.lock().await.new_resize_protocol(dyn_image);
    //            let preview = Arc::new(Preview::new(
    //                entry_c.name.clone(),
    //                PreviewContent::Image(image),
    //            ));
    //            cache
    //                .lock()
    //                .await
    //                .insert(entry_c.name.clone(), preview.clone());
    //        }
    //    });
    //}

    fn compute_highlighted_text_preview(
        &self,
        entry: &entry::Entry,
        reader: BufReader<File>,
    ) {
        let cache = self.cache.clone();
        let syntax_set = self.syntax_set.clone();
        let syntax_theme = self.syntax_theme.clone();
        let entry_c = entry.clone();
        tokio::spawn(async move {
            debug!(
                "Computing highlights in the background for {:?}",
                entry_c.name
            );
            let lines: Vec<String> = reader
                .lines()
                .map_while(Result::ok)
                // we need to add a newline here because sublime syntaxes expect one
                // to be present at the end of each line
                .map(|line| preprocess_line(&line).0 + "\n")
                .collect();

            match syntax::compute_highlights_for_path(
                &PathBuf::from(&entry_c.name),
                lines,
                &syntax_set,
                &syntax_theme,
            ) {
                Ok(highlighted_lines) => {
                    debug!(
                        "Successfully computed highlights for {:?}",
                        entry_c.name
                    );
                    cache.lock().insert(
                        entry_c.name.clone(),
                        Arc::new(Preview::new(
                            entry_c.name,
                            PreviewContent::SyntectHighlightedText(
                                highlighted_lines,
                            ),
                            entry_c.icon,
                        )),
                    );
                    debug!("Inserted highlighted preview into cache");
                }
                Err(e) => {
                    warn!("Error computing highlights: {:?}", e);
                    let preview = meta::not_supported(&entry_c.name);
                    cache.lock().insert(entry_c.name.clone(), preview);
                }
            };
        });
    }

    fn cache_preview(&mut self, key: String, preview: Arc<Preview>) {
        self.cache.lock().insert(key, preview);
    }
}

//fn get_image_picker() -> Picker {
//    let mut picker = match Picker::from_termios() {
//        Ok(p) => p,
//        Err(_) => Picker::new((7, 14)),
//    };
//    picker.guess_protocol();
//    picker.background_color = Some(Rgb::<u8>([255, 0, 255]));
//    picker
//}

/// This should be enough to most standard terminal sizes
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
    ))
}
