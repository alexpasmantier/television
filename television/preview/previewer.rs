use crate::channels::cable::prototypes::ChannelPrototype;
use crate::preview::{Preview, PreviewContent};
use crate::utils::command::shell_command;
use crate::{
    channels::{entry::Entry, preview::PreviewCommand},
    preview::cache::PreviewCache,
};
use parking_lot::Mutex;
use rustc_hash::FxHashSet;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use tracing::debug;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Previewer {
    cache: Arc<Mutex<PreviewCache>>,
    concurrent_preview_tasks: Arc<AtomicU8>,
    in_flight_previews: Arc<Mutex<FxHashSet<String>>>,
    command: PreviewCommand,
}

impl Previewer {
    // we could use a target scroll here to make the previewer
    // faster, but since it's already running in the background and quite
    // fast for most standard file sizes, plus we're caching the previews,
    // I'm not sure the extra complexity is worth it.
    pub fn request(&mut self, entry: &Entry) -> Option<Arc<Preview>> {
        // check if we have a preview in cache for the current request
        if let Some(preview) = self.cached(entry) {
            return Some(preview);
        }

        // start a background task to compute the preview
        self.preview(entry);

        None
    }
}

const MAX_CONCURRENT_PREVIEW_TASKS: u8 = 3;

impl Previewer {
    pub fn new(command: PreviewCommand) -> Self {
        Previewer {
            cache: Arc::new(Mutex::new(PreviewCache::default())),
            concurrent_preview_tasks: Arc::new(AtomicU8::new(0)),
            in_flight_previews: Arc::new(Mutex::new(FxHashSet::default())),
            command,
        }
    }

    pub fn cached(&self, entry: &Entry) -> Option<Arc<Preview>> {
        self.cache.lock().get(&entry.name)
    }

    pub fn preview(&mut self, entry: &Entry) {
        if self.in_flight_previews.lock().contains(&entry.name) {
            debug!("Preview already in flight for {:?}", entry.name);
            return;
        }

        if self.concurrent_preview_tasks.load(Ordering::Relaxed)
            < MAX_CONCURRENT_PREVIEW_TASKS
        {
            self.in_flight_previews.lock().insert(entry.name.clone());
            self.concurrent_preview_tasks
                .fetch_add(1, Ordering::Relaxed);
            let cache = self.cache.clone();
            let entry_c = entry.clone();
            let concurrent_tasks = self.concurrent_preview_tasks.clone();
            let command = self.command.clone();
            let in_flight_previews = self.in_flight_previews.clone();
            tokio::spawn(async move {
                try_preview(
                    &command,
                    &entry_c,
                    &cache,
                    &concurrent_tasks,
                    &in_flight_previews,
                );
            });
        } else {
            debug!(
                "Too many concurrent preview tasks, skipping {:?}",
                entry.name
            );
        }
    }
}

pub fn try_preview(
    command: &PreviewCommand,
    entry: &Entry,
    cache: &Arc<Mutex<PreviewCache>>,
    concurrent_tasks: &Arc<AtomicU8>,
    in_flight_previews: &Arc<Mutex<FxHashSet<String>>>,
) {
    debug!("Computing preview for {:?}", entry.name);
    let command = command.format_with(entry);
    debug!("Formatted preview command: {:?}", command);

    let child = shell_command(false)
        .arg(&command)
        .output()
        .expect("failed to execute process");

    if child.status.success() {
        let content = String::from_utf8_lossy(&child.stdout);
        let preview = Arc::new(Preview::new(
            entry.name.clone(),
            PreviewContent::AnsiText(content.to_string()),
            None,
            u16::try_from(content.lines().count()).unwrap_or(u16::MAX),
        ));

        cache.lock().insert(entry.name.clone(), &preview);
    } else {
        let content = String::from_utf8_lossy(&child.stderr);
        let preview = Arc::new(Preview::new(
            entry.name.clone(),
            PreviewContent::AnsiText(content.to_string()),
            None,
            u16::try_from(content.lines().count()).unwrap_or(u16::MAX),
        ));
        cache.lock().insert(entry.name.clone(), &preview);
    }

    concurrent_tasks.fetch_sub(1, Ordering::Relaxed);
    in_flight_previews.lock().remove(&entry.name);
}

impl From<&ChannelPrototype> for Option<Previewer> {
    fn from(value: &ChannelPrototype) -> Self {
        Option::<PreviewCommand>::from(value).map(Previewer::new)
    }
}
