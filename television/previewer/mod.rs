use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use ansi_to_tui::IntoText;
use anyhow::{Context, Result};
use image::{DynamicImage, ImageDecoder};
use parking_lot::Mutex;
use ratatui::text::Text;
use tokio::process::Command as TokioCommand;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::spawn,
    time::timeout,
};
use tracing::{debug, trace, warn};

use crate::{
    channels::{
        entry::Entry,
        prototypes::{CommandSpec, Template},
    },
    previewer::cache::Cache,
    utils::{
        command::shell_command,
        strings::{
            EMPTY_STRING, ReplaceNonPrintableConfig,
            replace_non_printable_bulk,
        },
    },
};

mod cache;
pub mod state;

pub struct Config {
    request_max_age: Duration,
    job_timeout: Duration,
}

pub const DEFAULT_REQUEST_MAX_AGE: Duration = Duration::from_millis(5000);
pub const DEFAULT_JOB_TIMEOUT: Duration = Duration::from_millis(500);
/// Longer timeout for image decoding and URL downloads.
pub const IMAGE_JOB_TIMEOUT: Duration = Duration::from_millis(5000);

impl Default for Config {
    fn default() -> Self {
        Self {
            request_max_age: DEFAULT_REQUEST_MAX_AGE,
            job_timeout: DEFAULT_JOB_TIMEOUT,
        }
    }
}

#[allow(
    clippy::large_enum_variant,
    reason = "requests are almost exclusively preview jobs"
)]
#[derive(PartialEq, Eq)]
pub enum Request {
    Preview(Ticket),
    Shutdown,
    CycleCommand,
}

impl PartialOrd for Request {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Request {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            // Shutdown/Cycle signals always have priority
            (Self::Shutdown | Self::CycleCommand, _) => Ordering::Greater,
            (_, Self::Shutdown | Self::CycleCommand) => Ordering::Less,
            // Otherwise fall back to ticket age comparison
            (Self::Preview(t1), Self::Preview(t2)) => t1.cmp(t2),
        }
    }
}

#[derive(PartialEq, Eq)]
pub struct Ticket {
    entry: Entry,
    timestamp: Instant,
}

impl PartialOrd for Ticket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ticket {
    fn cmp(&self, other: &Self) -> Ordering {
        self.age().cmp(&other.age())
    }
}

impl Ticket {
    pub fn new(entry: Entry) -> Self {
        Self {
            entry,
            timestamp: Instant::now(),
        }
    }

    fn age(&self) -> Duration {
        Instant::now().duration_since(self.timestamp)
    }
}

#[derive(Debug, Clone)]
pub enum PreviewContent {
    Text(Text<'static>),
    Image(Arc<DynamicImage>),
}

impl PartialEq for PreviewContent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Text(a), Self::Text(b)) => a == b,
            // Images are compared by pointer identity
            (Self::Image(a), Self::Image(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Eq for PreviewContent {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Preview {
    pub entry_raw: String,
    pub formatted_command: String,
    pub title: String,
    pub content: PreviewContent,
    pub target_line: Option<u16>,
    pub total_lines: u16,
    pub footer: Option<String>,
    pub preview_index: usize,
    pub preview_count: usize,
}

const DEFAULT_PREVIEW_TITLE: &str = "Select an entry to preview";

impl Default for Preview {
    fn default() -> Self {
        Self {
            entry_raw: EMPTY_STRING.to_string(),
            formatted_command: EMPTY_STRING.to_string(),
            title: DEFAULT_PREVIEW_TITLE.to_string(),
            content: PreviewContent::Text(Text::from(EMPTY_STRING)),
            target_line: None,
            total_lines: 1,
            footer: None,
            preview_index: 0,
            preview_count: 1,
        }
    }
}

impl Preview {
    #[allow(clippy::too_many_arguments)]
    fn new(
        entry_raw: String,
        formatted_command: String,
        title: &str,
        content: PreviewContent,
        line_number: Option<u16>,
        total_lines: u16,
        footer: Option<String>,
        preview_index: usize,
        preview_count: usize,
    ) -> Self {
        Self {
            entry_raw,
            formatted_command,
            title: title.to_string(),
            content,
            target_line: line_number,
            total_lines,
            footer,
            preview_index,
            preview_count,
        }
    }
}

pub struct Previewer {
    config: Config,
    requests_tx: UnboundedSender<Request>,
    requests_rx: UnboundedReceiver<Request>,
    last_job_entry: Option<Entry>,
    command: CommandSpec,
    /// The current cycle index for commands with multiple variants.
    cycle_index: usize,
    title_template: Option<Template>,
    footer_template: Option<Template>,
    offset_expr: Option<Template>,
    results: UnboundedSender<Preview>,
    cache: Option<Arc<Mutex<Cache>>>,
}

impl Previewer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        command: &CommandSpec,
        offset_expr: Option<Template>,
        title_template: Option<Template>,
        footer_template: Option<Template>,
        config: Config,
        requests_rx: UnboundedReceiver<Request>,
        requests_tx: UnboundedSender<Request>,
        results_tx: UnboundedSender<Preview>,
        cache: bool,
    ) -> Self {
        let cache = if cache {
            Some(Arc::new(Mutex::new(Cache::default())))
        } else {
            None
        };
        Self {
            config,
            requests_tx,
            requests_rx,
            last_job_entry: None,
            command: command.clone(),
            cycle_index: 0,
            title_template,
            footer_template,
            offset_expr,
            results: results_tx,
            cache,
        }
    }

    pub async fn run(mut self) {
        let mut buffer = Vec::with_capacity(32);
        loop {
            let num = self.requests_rx.recv_many(&mut buffer, 32).await;
            if num > 0 {
                debug!("Previewer received {num} request(s)!");
                // only keep the newest request
                match buffer.drain(..).max().unwrap() {
                    Request::Preview(ticket) => {
                        if ticket.age() > self.config.request_max_age {
                            debug!("Preview request is stale, skipping");
                            continue;
                        }
                        let results_handle = self.results.clone();
                        self.last_job_entry = Some(ticket.entry.clone());
                        let preview_command = self.command.clone();
                        let cache = self.cache.clone();
                        let offset_expr = self.offset_expr.clone();
                        let title_template = self.title_template.clone();
                        let footer_template = self.footer_template.clone();
                        let effective = ticket.entry.output().unwrap_or_else(|_| ticket.entry.raw.clone());
                        let job_timeout = if is_image_path(&effective) || is_pdf_path(&effective) {
                            IMAGE_JOB_TIMEOUT
                        } else {
                            self.config.job_timeout
                        };
                        let job = spawn(try_preview(
                            preview_command,
                            self.cycle_index,
                            title_template,
                            footer_template,
                            offset_expr,
                            ticket.entry,
                            results_handle,
                            cache,
                        ));
                        match timeout(job_timeout, job).await {
                            Ok(Ok(Ok(()))) => {
                                trace!("Preview job completed successfully");
                            }
                            Ok(Ok(Err(e))) => warn!(
                                "Failed to generate preview for entry '{}': {}",
                                &self.last_job_entry.clone().unwrap().raw,
                                e
                            ),
                            Ok(Err(join_err)) => {
                                warn!(
                                    "Preview join error for '{}': {}",
                                    self.last_job_entry.clone().unwrap().raw,
                                    join_err
                                );
                            }
                            Err(e) => {
                                warn!("Preview job timeout: {}", e);
                            }
                        }
                    }
                    Request::CycleCommand => {
                        trace!("Cycling preview command.");
                        self.cycle_command();
                    }
                    Request::Shutdown => {
                        trace!(
                            "Received shutdown signal, breaking out of the previewer loop."
                        );
                        break;
                    }
                }
            } else {
                trace!(
                    "Preview request channel closed and no messages left, breaking out of the previewer loop."
                );
                break;
            }
        }
    }

    pub fn cycle_command(&mut self) {
        self.cycle_index = (self.cycle_index + 1) % self.command.inner.len();
        // re-request preview for the last entry if any
        if let Some(entry) = &self.last_job_entry {
            let _ = self
                .requests_tx
                .send(Request::Preview(Ticket::new(entry.clone())));
        }
    }
}

fn sanitize_text(text: &mut Text<'static>) {
    text.lines.iter_mut().for_each(|line| {
        // replace non-printable characters
        line.spans.iter_mut().for_each(|span| {
            span.content = replace_non_printable_bulk(
                &span.content,
                &ReplaceNonPrintableConfig::default(),
            )
            .0
            .into_owned()
            .into();
        });
    });
}

/// Known image file extensions for native preview.
const IMAGE_EXTENSIONS: &[&str] = &[
    "bmp", "gif", "ico", "jfif", "jpeg", "jpg", "pbm", "pgm", "png",
    "pnm", "ppm", "qoi", "tga", "tif", "tiff", "webp",
];

/// File extensions that can be converted to images for native preview.
const PDF_EXTENSIONS: &[&str] = &["pdf"];

fn is_image_path(path: &str) -> bool {
    // Strip query parameters for URL paths before checking extension
    let clean = if path.starts_with("http://") || path.starts_with("https://") {
        path.split('?').next().unwrap_or(path)
    } else {
        path
    };
    Path::new(clean)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| {
            IMAGE_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str())
        })
}

fn is_pdf_path(path: &str) -> bool {
    let clean = if path.starts_with("http://") || path.starts_with("https://") {
        path.split('?').next().unwrap_or(path)
    } else {
        path
    };
    Path::new(clean)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| {
            PDF_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str())
        })
}

/// Convert a PDF's first page to a JPEG image via pdftoppm.
/// Returns the path to the cached JPEG on success.
fn pdf_to_image_cached(pdf_path: &Path) -> anyhow::Result<PathBuf> {
    use std::hash::{DefaultHasher, Hash, Hasher};

    let cache_dir = std::env::temp_dir().join("tv-pdf-cache");
    std::fs::create_dir_all(&cache_dir)?;

    let mut hasher = DefaultHasher::new();
    pdf_path.hash(&mut hasher);
    let hash = hasher.finish();
    let cached_base = cache_dir.join(format!("{hash:x}"));
    // pdftoppm appends the page number suffix, e.g. "abc.jpg"
    let cached_jpg = cache_dir.join(format!("{hash:x}.jpg"));

    if cached_jpg.exists() {
        return Ok(cached_jpg);
    }

    let status = std::process::Command::new("pdftoppm")
        .args([
            "-f", "1", "-l", "1",
            "-scale-to-x", "2048", "-scale-to-y", "-1",
            "-singlefile", "-jpeg",
        ])
        .arg("--")
        .arg(pdf_path)
        .arg(&cached_base)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;

    if status.success() && cached_jpg.exists() {
        Ok(cached_jpg)
    } else {
        anyhow::bail!("pdftoppm failed with status {status}")
    }
}

fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

/// Return the cached file path for a URL if it already exists.
fn cached_image_path(url: &str) -> Option<PathBuf> {
    use std::hash::{DefaultHasher, Hash, Hasher};

    let cache_dir = std::env::temp_dir().join("tv-image-cache");
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let hash = hasher.finish();

    let clean_url = url.split('?').next().unwrap_or(url);
    let ext = Path::new(clean_url)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    let cached = cache_dir.join(format!("{hash:x}.{ext}"));

    if cached.exists() { Some(cached) } else { None }
}

/// Download a URL to a temp cache file. Returns the cached path on success.
fn download_to_cache(url: &str) -> anyhow::Result<PathBuf> {
    use std::hash::{DefaultHasher, Hash, Hasher};

    let cache_dir = std::env::temp_dir().join("tv-image-cache");
    std::fs::create_dir_all(&cache_dir)?;

    // Hash URL for cache key
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let hash = hasher.finish();

    // Extract extension from URL (strip query params)
    let clean_url = url.split('?').next().unwrap_or(url);
    let ext = Path::new(clean_url)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    let cached = cache_dir.join(format!("{hash:x}.{ext}"));

    if cached.exists() {
        return Ok(cached);
    }

    let resp = ureq::get(url).call()?;

    // Don't download files larger than 20 MB
    if let Some(len) = resp.headers().get("content-length") {
        if let Ok(size) = len.to_str().unwrap_or("0").parse::<u64>() {
            if size > 20_000_000 {
                anyhow::bail!("Image too large to preview: {size} bytes");
            }
        }
    }

    let body = resp.into_body().read_to_vec()?;
    std::fs::write(&cached, &body)?;
    Ok(cached)
}

#[allow(clippy::too_many_arguments)]
fn build_preview_from_text(
    formatted_command: &str,
    entry: &Entry,
    text: Text<'static>,
    title_template: Option<&Template>,
    footer_template: Option<&Template>,
    offset_expr: Option<&Template>,
    preview_index: usize,
    preview_count: usize,
) -> Result<Preview> {
    let total_lines = u16::try_from(text.lines.len()).unwrap_or(0);

    // try to extract a line number from the offset expression if provided
    let line_number = if let Some(offset_expr) = offset_expr.as_ref() {
        let offset_str = offset_expr.format(&entry.raw)?;
        offset_str.parse::<u16>().ok()
    } else {
        None
    };

    let title = if let Some(title_template) = title_template.as_ref() {
        title_template.format(&entry.raw)?
    } else {
        entry.display().to_string()
    };
    let footer = if let Some(footer_template) = footer_template.as_ref() {
        Some(footer_template.format(&entry.raw)?)
    } else {
        None
    };

    Ok(Preview::new(
        entry.raw.clone(),
        formatted_command.to_string(),
        &title,
        PreviewContent::Text(text),
        line_number,
        total_lines,
        footer,
        preview_index,
        preview_count,
    ))
}

fn build_preview_from_image(
    formatted_command: &str,
    entry: &Entry,
    img: DynamicImage,
    title_template: Option<&Template>,
    footer_template: Option<&Template>,
    preview_index: usize,
    preview_count: usize,
) -> Result<Preview> {
    let title = if let Some(title_template) = title_template.as_ref() {
        title_template.format(&entry.raw)?
    } else {
        entry.display().to_string()
    };
    let footer = if let Some(footer_template) = footer_template.as_ref() {
        Some(footer_template.format(&entry.raw)?)
    } else {
        None
    };

    Ok(Preview::new(
        entry.raw.clone(),
        formatted_command.to_string(),
        &title,
        PreviewContent::Image(Arc::new(img)),
        None,
        0,
        footer,
        preview_index,
        preview_count,
    ))
}

#[allow(clippy::too_many_arguments)]
pub async fn try_preview(
    command: CommandSpec,
    cycle_index: usize,
    title_template: Option<Template>,
    footer_template: Option<Template>,
    offset_expr: Option<Template>,
    entry: Entry,
    results_handle: UnboundedSender<Preview>,
    cache: Option<Arc<Mutex<Cache>>>,
) -> Result<()> {
    let preview_count = command.inner.len();
    let formatted_command = command.get_nth(cycle_index).format(&entry.raw)?;

    // Use the output-formatted path (applies split/template) for image detection.
    // This handles entries where raw contains tab-separated fields (e.g. "url\tname").
    let effective_path = entry
        .output()
        .unwrap_or_else(|_| entry.raw.clone());

    // Try to detect and load image files natively.
    let is_img = is_image_path(&effective_path);
    debug!(
        "try_preview: entry='{}', effective='{}', is_image={}, is_url={}, is_file={}",
        &entry.raw,
        &effective_path,
        is_img,
        is_url(&effective_path),
        Path::new(&effective_path).is_file()
    );
    if is_img {
        let local_path: Option<PathBuf> = if is_url(&effective_path) {
            match cached_image_path(&effective_path) {
                Some(p) => {
                    debug!("Found cached image for URL: {}", &effective_path);
                    Some(p)
                }
                None => {
                    debug!("Image URL not cached, starting background download: {}", &effective_path);
                    let url = effective_path.clone();
                    tokio::task::spawn_blocking(move || {
                        let _ = download_to_cache(&url);
                    });
                    None
                }
            }
        } else if Path::new(&effective_path).is_file() {
            Some(PathBuf::from(&effective_path))
        } else {
            debug!("Image path is not a local file: {}", &effective_path);
            None
        };

        if let Some(path) = local_path {
            debug!("Loading image natively: {}", path.display());
            let img_result = tokio::task::spawn_blocking(move || {
                let mut reader = image::ImageReader::open(&path)?;
                let mut limits = image::Limits::default();
                limits.max_alloc = Some(512 * 1024 * 1024);
                reader.limits(limits);
                // Get orientation before decoding, then apply it
                let mut decoder = reader.into_decoder()?;
                let orientation = decoder.orientation().unwrap_or(image::metadata::Orientation::NoTransforms);
                let mut img = image::DynamicImage::from_decoder(decoder)?;
                img.apply_orientation(orientation);
                Ok::<_, image::ImageError>(img)
            })
            .await;
            match &img_result {
                Ok(Ok(img)) => debug!("Image decoded: {}x{}", img.width(), img.height()),
                Ok(Err(e)) => warn!("Image decode error: {}", e),
                Err(e) => warn!("Image task join error: {}", e),
            }
            if let Ok(Ok(img)) = img_result {
                let preview = build_preview_from_image(
                    &formatted_command,
                    &entry,
                    img,
                    title_template.as_ref(),
                    footer_template.as_ref(),
                    cycle_index,
                    preview_count,
                )?;
                results_handle
                    .send(preview)
                    .with_context(|| "Failed to send image preview result.")?;
                return Ok(());
            }
        }
        // Fall through to command-based preview if image loading fails
    }

    // Try to render PDFs natively by converting first page to an image.
    if is_pdf_path(&effective_path) {
        // Resolve to a local file: download remote PDFs first
        let local_pdf: Option<PathBuf> = if is_url(&effective_path) {
            match cached_image_path(&effective_path) {
                Some(p) => Some(p),
                None => {
                    let url = effective_path.clone();
                    tokio::task::spawn_blocking(move || {
                        let _ = download_to_cache(&url);
                    });
                    None // fall through while downloading
                }
            }
        } else if Path::new(&effective_path).is_file() {
            Some(PathBuf::from(&effective_path))
        } else {
            None
        };

        if let Some(pdf_path) = local_pdf {
        let convert_result = tokio::task::spawn_blocking(move || {
            let jpg_path = pdf_to_image_cached(&pdf_path)?;
            let mut reader = image::ImageReader::open(&jpg_path)?;
            let mut limits = image::Limits::default();
            limits.max_alloc = Some(512 * 1024 * 1024);
            reader.limits(limits);
            Ok::<_, anyhow::Error>(reader.decode()?)
        })
        .await;
        if let Ok(Ok(img)) = convert_result {
            let preview = build_preview_from_image(
                &formatted_command,
                &entry,
                img,
                title_template.as_ref(),
                footer_template.as_ref(),
                cycle_index,
                preview_count,
            )?;
            results_handle
                .send(preview)
                .with_context(|| "Failed to send PDF preview result.")?;
            return Ok(());
        }
        // Fall through to command-based preview if PDF conversion fails
        } // end if let Some(pdf_path)
    }

    // Check if the entry is already cached
    if let Some(cache) = &cache
        && let Some(text) = cache.lock().get(&formatted_command)
    {
        trace!("Preview for command '{}' found in cache", formatted_command);
        let preview = build_preview_from_text(
            &formatted_command,
            &entry,
            text,
            title_template.as_ref(),
            footer_template.as_ref(),
            offset_expr.as_ref(),
            cycle_index,
            preview_count,
        )?;
        results_handle.send(preview).with_context(
            || "Failed to send cached preview result to main thread.",
        )?;
        return Ok(());
    }

    debug!("Executing preview command: {}", &formatted_command);
    let shell_cmd = shell_command(
        &formatted_command,
        command.interactive,
        &command.env,
        command.shell,
    );

    let child = TokioCommand::from(shell_cmd).output().await?;

    let mut text = if child.status.success() {
        child
            .stdout
            .into_text()
            .unwrap_or_else(|_| Text::from(EMPTY_STRING))
    } else {
        child
            .stderr
            .into_text()
            .unwrap_or_else(|_| Text::from(EMPTY_STRING))
    };

    sanitize_text(&mut text);

    let preview = if let Some(cache) = &cache {
        let preview = build_preview_from_text(
            &formatted_command,
            &entry,
            text.clone(),
            title_template.as_ref(),
            footer_template.as_ref(),
            offset_expr.as_ref(),
            cycle_index,
            preview_count,
        )?;
        cache.lock().insert(&formatted_command, &text);
        preview
    } else {
        build_preview_from_text(
            &formatted_command,
            &entry,
            text,
            title_template.as_ref(),
            footer_template.as_ref(),
            offset_expr.as_ref(),
            cycle_index,
            preview_count,
        )?
    };
    // FIXME: ... and just send an Arc here as well
    results_handle
        .send(preview)
        .with_context(|| "Failed to send preview result to main thread.")
}
