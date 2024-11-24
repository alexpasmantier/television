use crate::previewers::cache::PreviewCache;
use crate::previewers::{Preview, PreviewContent};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use television_channels::entry::Entry;
use tracing::debug;

#[derive(Debug, Default)]
pub struct CommandPreviewer {
    cache: Arc<Mutex<PreviewCache>>,
    config: CommandPreviewerConfig,
    concurrent_preview_tasks: Arc<AtomicU8>,
    last_previewed: Arc<Mutex<Arc<Preview>>>,
}

#[derive(Default, Debug, Clone)]
pub struct CommandPreviewerConfig {}

impl CommandPreviewerConfig {
    pub fn new() -> Self {
        CommandPreviewerConfig {}
    }
}

const MAX_CONCURRENT_PREVIEW_TASKS: u8 = 2;

impl CommandPreviewer {
    pub fn new(config: Option<CommandPreviewerConfig>) -> Self {
        let config = config.unwrap_or_default();
        CommandPreviewer {
            cache: Arc::new(Mutex::new(PreviewCache::default())),
            config,
            concurrent_preview_tasks: Arc::new(AtomicU8::new(0)),
            last_previewed: Arc::new(Mutex::new(Arc::new(Preview::default()))),
        }
    }

    pub fn preview(&mut self, entry: &Entry, command: &str) -> Arc<Preview> {
        // do we have a preview in cache for that entry?
        if let Some(preview) = self.cache.lock().get(&entry.name) {
            return preview.clone();
        }
        debug!("No preview in cache for {:?}", entry.name);

        if self.concurrent_preview_tasks.load(Ordering::Relaxed)
            < MAX_CONCURRENT_PREVIEW_TASKS
        {
            self.concurrent_preview_tasks
                .fetch_add(1, Ordering::Relaxed);
            let cache = self.cache.clone();
            let entry_c = entry.clone();
            let concurrent_tasks = self.concurrent_preview_tasks.clone();
            let command = command.to_string();
            let last_previewed = self.last_previewed.clone();
            tokio::spawn(async move {
                try_preview(
                    &command,
                    &entry_c,
                    &cache,
                    &concurrent_tasks,
                    &last_previewed,
                );
            });
        }

        self.last_previewed.lock().clone()
    }
}

pub fn try_preview(
    command: &str,
    entry: &Entry,
    cache: &Arc<Mutex<PreviewCache>>,
    concurrent_tasks: &Arc<AtomicU8>,
    last_previewed: &Arc<Mutex<Arc<Preview>>>,
) {
    debug!("Computing preview for {:?}", entry.name);
    // this is kept dead simple for now but we could add a dedicated
    // parser to allow things like `{1}`, `{2..3}` etc.
    let command = command.replace("{}", &entry.name);

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let content = String::from_utf8(output.stdout)
            .unwrap_or(String::from("Failed to read output\n"));
        let preview = Arc::new(Preview::new(
            entry.name.clone(),
            PreviewContent::AnsiText(content),
            None,
        ));

        cache.lock().insert(entry.name.clone(), &preview);
        let mut tp = last_previewed.lock();
        *tp = preview;
    } else {
        let content = String::from_utf8(output.stderr)
            .unwrap_or(String::from("Failed to read error\n"));
        let preview = Arc::new(Preview::new(
            entry.name.clone(),
            PreviewContent::AnsiText(content),
            None,
        ));
        cache.lock().insert(entry.name.clone(), &preview);
    }

    concurrent_tasks.fetch_sub(1, Ordering::Relaxed);
}
