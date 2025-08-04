use crate::{
    channels::{
        entry::Entry,
        prototypes::{CommandSpec, Template},
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
    Arc, RwLock,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, trace};

const RELOAD_RENDERING_DELAY: Duration = Duration::from_millis(200);
const DEFAULT_LINE_BUFFER_SIZE: usize = 512;

const DATASET_CHANNEL_CAPACITY: usize = 32;
const DATASET_UPDATE_INTERVAL: Duration = Duration::from_millis(50);
const LOAD_CANDIDATE_BATCH_SIZE: usize = 100;

pub struct Channel {
    pub source_command: CommandSpec,
    pub source_entry_delimiter: Option<char>,
    pub source_ansi: bool,
    pub source_display: Option<Template>,
    pub source_output: Option<Template>,
    pub supports_preview: bool,
    matcher: Matcher<String>,
    selected_entries: FxHashSet<Entry>,
    crawl_handle: Option<tokio::task::JoinHandle<()>>,
    current_source_index: usize,
    /// Indicates if the channel is currently reloading to prevent UI flickering
    /// by delaying the rendering of a new frame.
    pub reloading: Arc<AtomicBool>,
    /// Track current dataset items
    current_dataset: Arc<RwLock<FxHashSet<String>>>,
    /// Handle for the dataset update task
    dataset_update_handle: Option<tokio::task::JoinHandle<()>>,
    /// Dedicated matcher for frecent items
    frecency_matcher: Matcher<String>,
}

impl Channel {
    pub fn new(
        source_command: CommandSpec,
        source_entry_delimiter: Option<char>,
        source_ansi: bool,
        source_display: Option<Template>,
        source_output: Option<Template>,
        supports_preview: bool,
    ) -> Self {
        let config = Config::default().prefer_prefix(true);
        let matcher = Matcher::new(&config);
        let current_source_index = 0;
        Self {
            source_command,
            source_entry_delimiter,
            source_ansi,
            source_display,
            source_output,
            supports_preview,
            matcher,
            selected_entries: HashSet::with_hasher(FxBuildHasher),
            crawl_handle: None,
            current_source_index,
            reloading: Arc::new(AtomicBool::new(false)),
            current_dataset: Arc::new(RwLock::new(FxHashSet::default())),
            dataset_update_handle: None,
            frecency_matcher: Matcher::new(&config),
        }
    }

    pub fn load(&mut self) {
        // Clear the current dataset at the start of each load
        if let Ok(mut dataset) = self.current_dataset.write() {
            dataset.clear();
        }

        // Clear recent matcher since dataset is changing
        self.frecency_matcher.restart();

        // Create bounded channel to prevent unbounded memory growth
        let (dataset_tx, dataset_rx) = mpsc::channel(DATASET_CHANNEL_CAPACITY);

        // Create dedicated dataset update task
        let dataset_clone = self.current_dataset.clone();
        let dataset_update_handle = tokio::spawn(async move {
            dataset_update_task(dataset_rx, dataset_clone).await;
        });
        self.dataset_update_handle = Some(dataset_update_handle);

        let injector = self.matcher.injector();
        let crawl_handle = tokio::spawn(load_candidates(
            self.source_command.clone(),
            self.source_entry_delimiter,
            self.current_source_index,
            self.source_ansi,
            self.source_display.clone(),
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

        // Abort existing tasks
        if let Some(handle) = self.crawl_handle.take() {
            if !handle.is_finished() {
                handle.abort();
            }
        }
        if let Some(handle) = self.dataset_update_handle.take() {
            if !handle.is_finished() {
                handle.abort();
            }
        }
        self.matcher.restart();
        self.frecency_matcher.restart();
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
        self.source_command.get_nth(self.current_source_index).raw()
    }

    pub fn find(&mut self, pattern: &str) {
        self.matcher.find(pattern);
        // Mark frecency matcher for reload if pattern changed
        if self.matcher.last_pattern != pattern {
            self.frecency_matcher.restart();
        }
    }

    /// Filter frecent items to only include those that exist in the current dataset
    fn filter_frecent_items_by_current_dataset(
        &self,
        frecent_items: &[String],
    ) -> FxHashSet<String> {
        // Try to read dataset, return empty on lock failure to prevent blocking
        let Ok(dataset) = self.current_dataset.read() else {
            debug!(
                "Failed to acquire dataset read lock, skipping frecency filtering"
            );
            return FxHashSet::default();
        };

        // Iterate over smaller frecent_items (~FRECENT_ITEMS_PRIORITY_COUNT)
        let mut intersection = FxHashSet::with_capacity_and_hasher(
            frecent_items.len().min(dataset.len()),
            FxBuildHasher,
        );

        for item in frecent_items {
            if dataset.contains(item) {
                intersection.insert(item.clone());
            }
        }

        intersection
    }

    /// Fuzzy match against a set of frecent items
    #[allow(clippy::cast_possible_truncation)]
    fn fuzzy_match_frecent_items(
        &mut self,
        pattern: &str,
        frecent_items: &FxHashSet<String>,
        offset: u32,
    ) -> Vec<MatchedItem<String>> {
        if frecent_items.is_empty() {
            return Vec::new();
        }

        // Apply the pattern
        self.frecency_matcher.find(pattern);

        // Let the matcher process
        self.frecency_matcher.tick();

        // Get all matches (frecent items are small, so we can get all)
        self.frecency_matcher
            .results(frecent_items.len() as u32, offset)
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn results(
        &mut self,
        num_entries: u32,
        offset: u32,
        frecency: Option<&Frecency>,
    ) -> Vec<Entry> {
        self.matcher.tick();

        let results = if let Some(frecency_data) = frecency {
            // Frecency-aware results with dataset validation
            let frecent_items = frecency_data.get_frecent_items();

            // Early exit if no frecent items to avoid unnecessary work
            if frecent_items.is_empty() {
                self.matcher.results(num_entries, offset)
            } else {
                // Filter frecent items to only include those in current dataset
                // This prevents searching stale entries that don't exist in the current data source
                let filtered_frecent_items = self
                    .filter_frecent_items_by_current_dataset(&frecent_items);

                // If no frecent items pass validation, fall back to regular matching
                if filtered_frecent_items.is_empty() {
                    self.matcher.results(num_entries, offset)
                } else {
                    // Only repopulate frecency matcher if needed and streaming is complete
                    if self.frecency_matcher.needs_reload() || self.running() {
                        debug!(
                            "Repopulating frecency matcher with {} items",
                            filtered_frecent_items.len()
                        );
                        self.frecency_matcher.restart();
                        let injector = self.frecency_matcher.injector();
                        for item in &filtered_frecent_items {
                            injector.push(item.clone(), |e, cols| {
                                cols[0] = e.clone().into();
                            });
                        }
                        self.frecency_matcher.mark_loaded();
                    }
                    // Fuzzy match the validated frecent items
                    let frecent_matches = self.fuzzy_match_frecent_items(
                        &self.matcher.last_pattern.clone(),
                        &filtered_frecent_items,
                        offset,
                    );

                    // Get regular results, excluding recent matches to avoid duplicates
                    let remaining_slots = num_entries
                        .saturating_sub(frecent_matches.len() as u32);

                    let mut regular_matches = Vec::new();
                    if remaining_slots > 0 {
                        // Fetch full list to account for deduplication
                        let nucleo_results =
                            self.matcher.results(num_entries, offset);

                        // Direct O(1) HashSet lookups for deduplication
                        regular_matches.reserve(remaining_slots as usize);
                        for item in nucleo_results {
                            if !filtered_frecent_items.contains(&item.inner) {
                                regular_matches.push(item);
                                if regular_matches.len()
                                    >= remaining_slots as usize
                                {
                                    break;
                                }
                            }
                        }
                    }

                    // Combine results: frecent items first (highest priority), then regular matches
                    let mut combined = frecent_matches;
                    combined.extend(regular_matches);

                    // Apply pagination
                    combined.into_iter().take(num_entries as usize).collect()
                }
            }
        } else {
            // No frecency: use standard nucleo matching
            self.matcher.results(num_entries, offset)
        };

        let mut entries = Vec::with_capacity(results.len());

        for item in results {
            let mut entry = Entry::new(item.inner)
                .with_display(item.matched_string)
                .with_match_indices(&item.match_indices)
                .ansi(self.source_ansi);
            if let Some(output) = &self.source_output {
                entry = entry.with_output(output.clone());
            }
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
            if let Some(output) = &self.source_output {
                entry = entry.with_output(output.clone());
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

    /// Cycles to the next source command
    pub fn cycle_sources(&mut self) {
        if self.source_command.inner.len() > 1 {
            self.current_source_index = (self.current_source_index + 1)
                % self.source_command.inner.len();
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

/// Dedicated task for updating the dataset from batched updates
/// This runs independently from the UI to prevent blocking
async fn dataset_update_task(
    mut dataset_rx: mpsc::Receiver<Vec<String>>,
    current_dataset: Arc<RwLock<FxHashSet<String>>>,
) {
    debug!("Starting dataset update task");

    let mut update_interval = tokio::time::interval(DATASET_UPDATE_INTERVAL);
    let mut pending_updates = Vec::new();

    loop {
        tokio::select! {
            // Receive new batches
            batch_result = dataset_rx.recv() => {
                 if let Some(batch) = batch_result {
                     pending_updates.push(batch);
                 } else {
                     // Channel closed, process remaining updates and exit
                     if !pending_updates.is_empty() {
                         apply_pending_updates(&pending_updates, &current_dataset);
                     }
                     debug!("Dataset update task exiting - channel closed");
                     break;
                 }
            }
            // Periodic updates to apply accumulated batches
            _ = update_interval.tick() => {
                if !pending_updates.is_empty() {
                    apply_pending_updates(&pending_updates, &current_dataset);
                    pending_updates.clear();
                }
            }
        }
    }
}

/// Apply accumulated updates to the dataset with proper error handling
fn apply_pending_updates(
    pending_updates: &[Vec<String>],
    current_dataset: &Arc<RwLock<FxHashSet<String>>>,
) {
    match current_dataset.write() {
        Ok(mut dataset) => {
            // Pre-calculate capacity to minimize reallocations
            let total_items: usize =
                pending_updates.iter().map(Vec::len).sum();
            dataset.reserve(total_items);

            // Apply all pending updates
            for batch in pending_updates {
                dataset.extend(batch.iter().cloned());
            }

            trace!(
                "Applied {} batches containing {} total items to dataset",
                pending_updates.len(),
                total_items
            );
        }
        Err(e) => {
            debug!(
                "Failed to acquire dataset write lock: {:?}. Skipping batch updates.",
                e
            );
        }
    }
}

/// Helper function to send a batch if it's not empty with backpressure handling
async fn send_batch_if_not_empty(
    batch: &mut Vec<String>,
    dataset_tx: &mpsc::Sender<Vec<String>>,
) -> Result<(), mpsc::error::SendError<Vec<String>>> {
    if batch.is_empty() {
        Ok(())
    } else {
        let batch_to_send = std::mem::take(batch);
        dataset_tx.send(batch_to_send).await
    }
}

async fn load_candidates(
    command: CommandSpec,
    entry_delimiter: Option<char>,
    command_index: usize,
    ansi: bool,
    display: Option<Template>,
    injector: Injector<String>,
    dataset_tx: mpsc::Sender<Vec<String>>,
) {
    debug!("Loading candidates from command: {:?}", command);
    let mut current_batch = Vec::with_capacity(LOAD_CANDIDATE_BATCH_SIZE);

    let mut child = shell_command(
        command.get_nth(command_index).raw(),
        command.interactive,
        &command.env,
    )
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("failed to execute process");

    if let Some(out) = child.stdout.take() {
        let mut produced_output = false;
        let mut reader = BufReader::new(out);
        let mut buf = Vec::with_capacity(DEFAULT_LINE_BUFFER_SIZE);

        let delimiter =
            entry_delimiter.as_ref().map(|d| *d as u8).unwrap_or(b'\n');

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
                    if current_batch.len() >= LOAD_CANDIDATE_BATCH_SIZE {
                        if let Err(e) = send_batch_if_not_empty(
                            &mut current_batch,
                            &dataset_tx,
                        )
                        .await
                        {
                            debug!(
                                "Failed to send dataset batch: {:?}. Dataset may be incomplete.",
                                e
                            );
                            break; // Exit loop if channel is closed
                        }
                    }

                    let () = injector.push(entry, |e, cols| {
                        if ansi {
                            cols[0] = strip_ansi.format(e).unwrap_or_else(|_| {
                                panic!(
                                    "Failed to strip ANSI from entry '{}'",
                                    e
                                );
                            }).into();
                        } else if let Some(display) = &display {
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
                    if current_batch.len() >= LOAD_CANDIDATE_BATCH_SIZE {
                        if let Err(e) = send_batch_if_not_empty(
                            &mut current_batch,
                            &dataset_tx,
                        )
                        .await
                        {
                            debug!(
                                "Failed to send dataset batch: {:?}. Dataset may be incomplete.",
                                e
                            );
                            break; // Exit loop if channel is closed
                        }
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
    if let Err(e) =
        send_batch_if_not_empty(&mut current_batch, &dataset_tx).await
    {
        debug!("Failed to send final dataset batch: {:?}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{channels::prototypes::SourceSpec, matcher::config::Config};

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
        let (dataset_tx, _dataset_rx) =
            mpsc::channel(DATASET_CHANNEL_CAPACITY);

        load_candidates(
            source_spec.command,
            source_spec.entry_delimiter,
            0,
            source_spec.ansi,
            source_spec.display,
            injector,
            dataset_tx,
        )
        .await;

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
        let (dataset_tx, _dataset_rx) =
            mpsc::channel(DATASET_CHANNEL_CAPACITY);

        load_candidates(
            source_spec.command,
            source_spec.entry_delimiter,
            0,
            source_spec.ansi,
            source_spec.display,
            injector,
            dataset_tx,
        )
        .await;

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
        let (dataset_tx, _dataset_rx) =
            mpsc::channel(DATASET_CHANNEL_CAPACITY);

        load_candidates(
            source_spec.command,
            source_spec.entry_delimiter,
            0,
            source_spec.ansi,
            source_spec.display,
            injector,
            dataset_tx,
        )
        .await;

        // Check if the matcher has the expected results
        matcher.find("test");
        matcher.tick();
        let results = matcher.results(10, 0);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].inner, "test1");
        assert_eq!(results[1].inner, "test2\ntest3");
    }
}
