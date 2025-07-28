use crate::{
    channels::{
        entry::Entry,
        prototypes::{ChannelPrototype, SourceSpec, Template},
    },
    frecency::Frecency,
    matcher::{
        Matcher, config::Config, injector::Injector, matched_item::MatchedItem,
    },
    utils::command::shell_command,
};
use rustc_hash::{FxBuildHasher, FxHashSet};
use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use std::process::Stdio;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, trace};

const RELOAD_RENDERING_DELAY: Duration = Duration::from_millis(200);

pub struct Channel {
    pub prototype: ChannelPrototype,
    matcher: Matcher<String>,
    selected_entries: FxHashSet<Entry>,
    crawl_handle: Option<tokio::task::JoinHandle<()>>,
    current_source_index: usize,
    /// Indicates if the channel is currently reloading to prevent UI flickering
    /// by delaying the rendering of a new frame.
    pub reloading: Arc<AtomicBool>,
    /// Track current dataset items as they're loaded for frecency filtering
    current_dataset: FxHashSet<String>,
    /// Receiver for batched dataset updates from the loading task
    dataset_rx: Option<mpsc::UnboundedReceiver<Vec<String>>>,
}

impl Channel {
    pub fn new(prototype: ChannelPrototype) -> Self {
        let config = Config::default().prefer_prefix(true);
        let matcher = Matcher::new(&config);
        let current_source_index = 0;
        Self {
            prototype,
            matcher,
            selected_entries: HashSet::with_hasher(FxBuildHasher),
            crawl_handle: None,
            current_source_index,
            reloading: Arc::new(AtomicBool::new(false)),
            current_dataset: FxHashSet::default(),
            dataset_rx: None,
        }
    }

    pub fn load(&mut self) {
        // Clear the current dataset at the start of each load
        self.current_dataset.clear();

        let (dataset_tx, dataset_rx) = mpsc::unbounded_channel();
        self.dataset_rx = Some(dataset_rx);

        let injector = self.matcher.injector();
        let crawl_handle = tokio::spawn(load_candidates(
            self.prototype.source.clone(),
            self.current_source_index,
            injector,
            dataset_tx,
        ));
        self.crawl_handle = Some(crawl_handle);
    }

    pub fn reload(&mut self) {
        if self.reloading.load(Ordering::Relaxed) {
            debug!("Reload already in progress, skipping.");
            return;
        }
        self.reloading.store(true, Ordering::Relaxed);

        if let Some(handle) = self.crawl_handle.take() {
            if !handle.is_finished() {
                handle.abort();
            }
        }
        self.matcher.restart();
        self.load();
        // Spawn a thread that turns off reloading after a short delay
        // to avoid UI flickering (this boolean is used by `Television::should_render`)
        let reloading = self.reloading.clone();
        tokio::spawn(async move {
            tokio::time::sleep(RELOAD_RENDERING_DELAY).await;
            reloading.store(false, Ordering::Relaxed);
        });
    }

    pub fn current_command(&self) -> &str {
        self.prototype
            .source
            .command
            .get_nth(self.current_source_index)
            .raw()
    }

    pub fn find(&mut self, pattern: &str) {
        self.matcher.find(pattern);
    }

    /// Try to update the dataset from the loading task if available
    fn try_update_dataset(&mut self) {
        if let Some(rx) = &mut self.dataset_rx {
            // Process all available batches (non-blocking)
            while let Ok(batch) = rx.try_recv() {
                // Extend current dataset with the new batch
                self.current_dataset.extend(batch);
            }
        }
    }

    /// Filter recent items to only include those that exist in the current dataset
    fn filter_recent_items_by_current_dataset(
        &self,
        recent_items: &[String],
    ) -> Vec<String> {
        let mut filtered = Vec::with_capacity(recent_items.len());
        filtered.extend(
            recent_items
                .iter()
                .filter(|item| self.current_dataset.contains(*item))
                .cloned(),
        );
        filtered
    }

    /// Fuzzy match against a list of recent items
    #[allow(clippy::cast_possible_truncation)]
    fn fuzzy_match_recent_items(
        pattern: &str,
        recent_items: &[String],
    ) -> Vec<MatchedItem<String>> {
        if recent_items.is_empty() {
            return Vec::new();
        }

        // Create a temporary matcher for recent items
        let config = Config::default().prefer_prefix(true);
        let mut recent_matcher = Matcher::new(&config);

        // Inject recent items into the matcher
        let injector = recent_matcher.injector();
        for item in recent_items {
            injector.push(item.clone(), |e, cols| {
                cols[0] = e.clone().into();
            });
        }

        // Apply the pattern
        recent_matcher.find(pattern);

        // Let the matcher process
        recent_matcher.tick();

        // Get all matches (recent items are small, so we can get all)
        recent_matcher.results(recent_items.len() as u32, 0)
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn results(
        &mut self,
        num_entries: u32,
        offset: u32,
        frecency: Option<&Frecency>,
    ) -> Vec<Entry> {
        // Try to update dataset from loading task
        self.try_update_dataset();

        self.matcher.tick();

        let results = if let Some(frecency_data) = frecency {
            // Frecency-aware results with dataset validation
            let recent_items = frecency_data.get_recent_items();

            // Early exit if no recent items to avoid unnecessary work
            if recent_items.is_empty() {
                self.matcher.results(num_entries, offset)
            } else {
                let filtered_recent_items =
                    self.filter_recent_items_by_current_dataset(&recent_items);

                // If no recent items pass validation, fall back to regular matching
                if filtered_recent_items.is_empty() {
                    self.matcher.results(num_entries, offset)
                } else {
                    // Fuzzy match the validated recent items
                    let recent_matches = Self::fuzzy_match_recent_items(
                        &self.matcher.last_pattern,
                        &filtered_recent_items,
                    );

                    // Get regular results, excluding recent matches to avoid duplicates
                    let remaining_slots = num_entries
                        .saturating_sub(recent_matches.len() as u32);

                    let mut regular_matches = Vec::new();
                    if remaining_slots > 0 {
                        // Fetch full list to account for deduplication
                        let nucleo_results =
                            self.matcher.results(num_entries, 0);

                        // Use Vec::contains for small recent items list (faster than HashSet creation)
                        regular_matches.reserve(remaining_slots as usize);
                        for item in nucleo_results {
                            if !filtered_recent_items.contains(&item.inner) {
                                regular_matches.push(item);
                                if regular_matches.len()
                                    >= remaining_slots as usize
                                {
                                    break;
                                }
                            }
                        }
                    }

                    // Combine with recent items prioritized first
                    let mut combined = recent_matches;
                    combined.extend(regular_matches);

                    // Apply pagination
                    combined
                        .into_iter()
                        .skip(offset as usize)
                        .take(num_entries as usize)
                        .collect()
                }
            }
        } else {
            // No frecency: use standard nucleo matching
            self.matcher.results(num_entries, offset)
        };

        let mut entries = Vec::with_capacity(results.len());

        for item in results {
            let entry = Entry::new(item.inner)
                .with_display(item.matched_string)
                .with_match_indices(&item.match_indices)
                .ansi(self.prototype.source.ansi);
            entries.push(entry);
        }

        entries
    }

    pub fn get_result(&mut self, index: u32) -> Option<Entry> {
        let item = self.matcher.get_result(index);

        if let Some(item) = item {
            let mut entry = Entry::new(item.inner.clone())
                .with_display(item.matched_string)
                .with_match_indices(&item.match_indices);
            if let Some(p) = &self.prototype.preview {
                // FIXME: this should be done by the previewer instead
                if let Some(offset_expr) = &p.offset {
                    let offset_str =
                        offset_expr.format(&item.inner).unwrap_or_default();

                    entry = entry.with_line_number(
                        offset_str.parse::<usize>().unwrap_or(0),
                    );
                }
            }
            Some(entry)
        } else {
            None
        }
    }

    pub fn selected_entries(&self) -> &FxHashSet<Entry> {
        &self.selected_entries
    }

    pub fn toggle_selection(&mut self, entry: &Entry) {
        if self.selected_entries.contains(entry) {
            self.selected_entries.remove(entry);
        } else {
            self.selected_entries.insert(entry.clone());
        }
    }

    pub fn result_count(&self) -> u32 {
        self.matcher.matched_item_count
    }

    pub fn total_count(&self) -> u32 {
        self.matcher.total_item_count
    }

    pub fn running(&self) -> bool {
        self.matcher.status.running
            || (self.crawl_handle.is_some()
                && !self.crawl_handle.as_ref().unwrap().is_finished())
    }

    pub fn shutdown(&self) {}

    pub fn supports_preview(&self) -> bool {
        self.prototype.preview.is_some()
    }

    /// Cycles to the next source command
    pub fn cycle_sources(&mut self) {
        if self.prototype.source.command.inner.len() > 1 {
            self.current_source_index = (self.current_source_index + 1)
                % self.prototype.source.command.inner.len();
            debug!(
                "Cycling to source command index: {}",
                self.current_source_index
            );
            self.reload();
        } else {
            debug!("No other source commands to cycle through.");
        }
    }
}

const DEFAULT_LINE_BUFFER_SIZE: usize = 512;
const BATCH_SIZE: usize = 100;

/// Helper function to send a batch if it's not empty
fn send_batch_if_not_empty(
    batch: &mut Vec<String>,
    dataset_tx: &mpsc::UnboundedSender<Vec<String>>,
) {
    if !batch.is_empty() {
        let _ = dataset_tx.send(std::mem::take(batch));
    }
}

#[allow(clippy::unused_async)]
async fn load_candidates(
    source: SourceSpec,
    source_command_index: usize,
    injector: Injector<String>,
    dataset_tx: mpsc::UnboundedSender<Vec<String>>,
) {
    debug!("Loading candidates from command: {:?}", source.command);
    let mut current_batch = Vec::with_capacity(BATCH_SIZE);

    let mut child = shell_command(
        source.command.get_nth(source_command_index).raw(),
        source.command.interactive,
        &source.command.env,
    )
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("failed to execute process");

    if let Some(out) = child.stdout.take() {
        let mut produced_output = false;
        let mut reader = BufReader::new(out);
        let mut buf = Vec::with_capacity(DEFAULT_LINE_BUFFER_SIZE);

        let delimiter = source
            .entry_delimiter
            .as_ref()
            .map(|d| *d as u8)
            .unwrap_or(b'\n');

        let strip_ansi = Template::parse("{strip_ansi}").unwrap();

        loop {
            buf.clear();
            let n = reader.read_until(delimiter, &mut buf).unwrap();
            if n == 0 {
                break; // EOF
            }

            // Remove trailing delimiter
            if buf.last() == Some(&delimiter) {
                buf.pop();
            }

            if buf.is_empty() {
                continue;
            }

            if let Ok(l) = std::str::from_utf8(&buf) {
                trace!("Read line: {}", l);
                if !l.trim().is_empty() {
                    let entry = l.to_string();

                    // Add to current batch
                    current_batch.push(entry.clone());

                    // Send batch if it reaches the batch size
                    if current_batch.len() >= BATCH_SIZE {
                        send_batch_if_not_empty(
                            &mut current_batch,
                            &dataset_tx,
                        );
                    }

                    let () = injector.push(entry, |e, cols| {
                        if source.ansi {
                            cols[0] = strip_ansi.format(e).unwrap_or_else(|_| {
                                panic!(
                                    "Failed to strip ANSI from entry '{}'",
                                    e
                                );
                            }).into();
                        } else if let Some(display) = &source.display {
                            let formatted = display.format(e).unwrap_or_else(|_| {
                                panic!(
                                    "Failed to format display expression '{}' with entry '{}'",
                                    display.raw(),
                                    e
                                );
                            });
                            cols[0] = formatted.into();
                        } else {
                            cols[0] = e.clone().into();
                        }
                    });
                    produced_output = true;
                }
            }
        }

        // if the command didn't produce any output, check stderr and display that instead
        if !produced_output {
            let reader = BufReader::new(child.stderr.take().unwrap());
            for line in reader.lines() {
                let line = line.unwrap();
                if !line.trim().is_empty() {
                    // Add to current batch
                    current_batch.push(line.clone());

                    // Send batch if it reaches the batch size
                    if current_batch.len() >= BATCH_SIZE {
                        send_batch_if_not_empty(
                            &mut current_batch,
                            &dataset_tx,
                        );
                    }

                    let () = injector.push(line, |e, cols| {
                        cols[0] = e.clone().into();
                    });
                }
            }
        }
    }
    let _ = child.wait();

    // Send any remaining entries in the final batch
    send_batch_if_not_empty(&mut current_batch, &dataset_tx);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matcher::config::Config;

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    async fn test_load_candidates_default_delimiter() {
        let source_spec: SourceSpec = toml::from_str(
            r#"
            command = "echo 'test1\ntest2\ntest3'"
            "#,
        )
        .unwrap();

        let mut matcher = Matcher::<String>::new(&Config::default());
        let injector = matcher.injector();
        let (dataset_tx, _dataset_rx) = mpsc::unbounded_channel();

        load_candidates(source_spec, 0, injector, dataset_tx).await;

        // Check if the matcher has the expected results
        matcher.find("test");
        matcher.tick();
        let results = matcher.results(10, 0);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].inner, "test1");
        assert_eq!(results[1].inner, "test2");
        assert_eq!(results[2].inner, "test3");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    async fn test_load_candidates_null_byte_delimiter() {
        let source_spec: SourceSpec = toml::from_str(
            r#"command = "printf 'test1\\0test2\\0test3\\0'"
            entry_delimiter = "\\0""#,
        )
        .unwrap();

        let mut matcher = Matcher::<String>::new(&Config::default());
        let injector = matcher.injector();
        let (dataset_tx, _dataset_rx) = mpsc::unbounded_channel();

        load_candidates(source_spec, 0, injector, dataset_tx).await;

        // Check if the matcher has the expected results
        matcher.find("test");
        matcher.tick();
        let results = matcher.results(10, 0);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].inner, "test1");
        assert_eq!(results[1].inner, "test2");
        assert_eq!(results[2].inner, "test3");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    async fn test_load_candidates_null_byte_and_newlines() {
        let source_spec: SourceSpec = toml::from_str(
            r#"command = "printf 'test1\\0test2\\ntest3\\0'"
            entry_delimiter = "\\0""#,
        )
        .unwrap();

        let mut matcher = Matcher::<String>::new(&Config::default());
        let injector = matcher.injector();
        let (dataset_tx, _dataset_rx) = mpsc::unbounded_channel();

        load_candidates(source_spec, 0, injector, dataset_tx).await;

        // Check if the matcher has the expected results
        matcher.find("test");
        matcher.tick();
        let results = matcher.results(10, 0);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].inner, "test1");
        assert_eq!(results[1].inner, "test2\ntest3");
    }
}
