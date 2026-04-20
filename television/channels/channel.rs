use crate::{
    channels::{
        entry::Entry,
        entry_processor::{
            AnsiProcessor, DisplayProcessor, EntryProcessor, PlainProcessor,
        },
        prototypes::{CommandSpec, Template},
    },
    frecency::FrecencyHandle,
    matcher::{
        Matcher, config::Config, config::SortStrategy, injector::Injector,
    },
    utils::command::shell_command,
};
use rustc_hash::{FxBuildHasher, FxHashSet};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command as TokioCommand;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    time::Instant,
};
use tracing::debug;

/// Factory that rebuilds a fresh `SortStrategy` for each new staging matcher.
///
/// `SortStrategy::Custom` holds a `Box<dyn SortFn>` which is not `Clone`, so we
/// can't reuse the strategy across matchers. Instead we keep a closure that
/// knows how to produce a new one (re-capturing any shared state such as the
/// frecency cache) on every reload.
type SortStrategyFactory<D> = Box<dyn FnMut() -> SortStrategy<D> + Send>;

pub struct Channel<P: EntryProcessor> {
    pub source_command: CommandSpec,
    pub source_entry_delimiter: Option<char>,
    pub source_output: Option<Template>,
    pub supports_preview: bool,
    processor: P,
    /// The matcher serving results to the UI. See `reload()` for the swap.
    matcher: Matcher<P::Data>,
    selected_entries: FxHashSet<Entry>,
    crawl_handle: Option<tokio::task::JoinHandle<()>>,
    current_source_index: usize,
    /// Accumulates the new source's output during a reload; swapped into
    /// `matcher` by `try_swap_staging()`.
    staging_matcher: Option<Matcher<P::Data>>,
    staging_crawl_handle: Option<tokio::task::JoinHandle<()>>,
    sort_strategy_factory: SortStrategyFactory<P::Data>,
    /// Whether this channel reads from stdin directly instead of spawning a
    /// source command. When true, `load()` reads `tokio::io::stdin()` and
    /// `reload()` is a no-op (stdin can only be consumed once).
    is_stdin: bool,
}

impl<P: EntryProcessor> Channel<P> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_command: CommandSpec,
        source_entry_delimiter: Option<char>,
        source_output: Option<Template>,
        supports_preview: bool,
        no_sort: bool,
        processor: P,
        frecency: Option<(FrecencyHandle, String)>,
        is_stdin: bool,
    ) -> Self {
        let config = Config::default().prefer_prefix(true);

        let mut sort_strategy_factory: SortStrategyFactory<P::Data> =
            if no_sort {
                Box::new(|| SortStrategy::Index)
            } else if let Some((frecency_handle, channel_name)) = frecency {
                let cache = frecency_handle.create_cache(channel_name);
                Box::new(move || {
                    let cache = cache.clone();
                    SortStrategy::Custom(Box::new(move |m1, i1, m2, i2| {
                        let scores = cache.snapshot();
                        let key1 = P::frecency_key(&i1);
                        let key2 = P::frecency_key(&i2);
                        let f1 = scores.get(&key1);
                        let f2 = scores.get(&key2);

                        match (f1, f2) {
                            (Some(s1), Some(s2)) => s2
                                .cmp(&s1)
                                .then_with(|| m2.score.cmp(&m1.score)),
                            (Some(_), None) => Ordering::Less,
                            (None, Some(_)) => Ordering::Greater,
                            (None, None) => m2.score.cmp(&m1.score),
                        }
                    }))
                })
            } else {
                Box::new(|| SortStrategy::Score)
            };

        let matcher = Matcher::new(&config, sort_strategy_factory());
        let current_source_index = 0;
        Self {
            source_command,
            source_entry_delimiter,
            source_output,
            supports_preview,
            processor,
            matcher,
            selected_entries: HashSet::with_hasher(FxBuildHasher),
            crawl_handle: None,
            current_source_index,
            staging_matcher: None,
            staging_crawl_handle: None,
            sort_strategy_factory,
            is_stdin,
        }
    }

    pub fn load(&mut self) {
        let injector = self.matcher.injector();
        let processor = self.processor.clone();
        let crawl_handle = if self.is_stdin {
            tokio::spawn(load_stdin_candidates(
                self.source_entry_delimiter,
                processor,
                injector,
            ))
        } else {
            tokio::spawn(load_candidates(
                self.source_command.clone(),
                self.source_entry_delimiter,
                self.current_source_index,
                processor,
                injector,
            ))
        };
        self.crawl_handle = Some(crawl_handle);
    }

    /// Reload the channel's source with fzf-style atomic-swap semantics.
    ///
    /// Builds a fresh *staging* matcher and feeds the new source into it; the
    /// live matcher keeps serving the previous snapshot until `tick()` swaps
    /// staging in. The swap is event-driven (no time cap): it fires when
    /// staging has items, or when the crawl task exits with no output. A
    /// source that hangs indefinitely keeps the previous snapshot on screen.
    pub fn reload(&mut self) {
        if self.is_stdin {
            debug!("Stdin channel cannot be reloaded, skipping.");
            return;
        }
        if self.staging_matcher.is_some() {
            debug!("Reload already in progress, skipping.");
            return;
        }

        // Mirror the current pattern onto staging so the post-swap snapshot is
        // already filtered to what the user typed during the reload window.
        let config = Config::default().prefer_prefix(true);
        let mut staging =
            Matcher::new(&config, (self.sort_strategy_factory)());
        if !self.matcher.last_pattern.is_empty() {
            staging.find(&self.matcher.last_pattern);
        }

        let injector = staging.injector();
        let processor = self.processor.clone();
        let crawl_handle = tokio::spawn(load_candidates(
            self.source_command.clone(),
            self.source_entry_delimiter,
            self.current_source_index,
            processor,
            injector,
        ));

        self.staging_matcher = Some(staging);
        self.staging_crawl_handle = Some(crawl_handle);
    }

    pub fn current_command(&self) -> &str {
        self.source_command.get_nth(self.current_source_index).raw()
    }

    pub fn find(&mut self, pattern: &str) {
        self.matcher.find(pattern);
        // Mirror onto staging so the post-swap snapshot is already filtered.
        if let Some(staging) = self.staging_matcher.as_mut() {
            staging.find(pattern);
        }
    }

    /// Let the background matcher thread make progress.
    ///
    /// Cheap, call every update cycle. Also drives the staging swap when a
    /// reload is in flight.
    pub fn tick(&mut self) {
        self.matcher.tick();
        self.try_swap_staging();
    }

    /// Swap staging into the live slot once staging has items, or once its
    /// crawl task exits with nothing emitted. A source that hangs with no
    /// output never triggers a swap — the previous snapshot stays on screen.
    fn try_swap_staging(&mut self) {
        let Some(staging) = self.staging_matcher.as_mut() else {
            return;
        };
        staging.tick();
        let has_items = staging.total_item_count > 0;
        let source_finished = self
            .staging_crawl_handle
            .as_ref()
            .is_some_and(tokio::task::JoinHandle::is_finished);
        if !has_items && !source_finished {
            return;
        }

        self.matcher = self.staging_matcher.take().unwrap();

        // The new source may still be streaming more items; move its handle
        // into `crawl_handle` so `running()` keeps reporting accurately.
        if let Some(old) = self.crawl_handle.take()
            && !old.is_finished()
        {
            old.abort();
        }
        self.crawl_handle = self.staging_crawl_handle.take();
    }

    pub fn results(&mut self, num_entries: u32, offset: u32) -> Vec<Entry> {
        // Channel-level tick so a pending staging swap is applied before we
        // read results; a bare `self.matcher.tick()` would return one last
        // snapshot from the old matcher.
        self.tick();

        let results = self.matcher.results(num_entries, offset);

        // PERF: this could be preallocated and reused by the caller
        let mut entries = Vec::with_capacity(results.len());

        for item in results {
            entries.push(
                self.processor.make_entry(item, self.source_output.as_ref()),
            );
        }

        entries
    }

    pub fn get_result(&mut self, index: u32) -> Option<Entry> {
        self.matcher.get_result(index).map(|item| {
            self.processor.make_entry(item, self.source_output.as_ref())
        })
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

    /// Whether the channel is actively producing results the user can see.
    ///
    /// Ignores the staging matcher on purpose: during an atomic-swap reload
    /// the user is still looking at a stable list, so the UI's loading
    /// indicator should stay off until the swap happens. The initial load is
    /// still reported (no live results yet), and after a swap the former
    /// staging handle moves into `crawl_handle` so this stays accurate.
    pub fn running(&self) -> bool {
        self.matcher.status.running
            || (self.crawl_handle.is_some()
                && !self.crawl_handle.as_ref().unwrap().is_finished())
    }

    pub fn shutdown(&self) {}

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

    pub fn supports_preview(&self) -> bool {
        self.supports_preview
    }

    pub fn source_index(&self) -> usize {
        self.current_source_index
    }

    pub fn source_count(&self) -> usize {
        self.source_command.inner.len()
    }

    pub fn is_stdin(&self) -> bool {
        self.is_stdin
    }
}

const DEFAULT_LINE_BUFFER_SIZE: usize = 256;
// Batch size for pushing candidates to the injector
// 10k * 500 bytes (pessimistic avg line size) = ~5 MB
const BATCH_SIZE: usize = 10_000;
// Automatically flush batch after this interval
const UPDATE_INTERVAL: Duration = Duration::from_millis(200);
// Maximum number of concurrent flush tasks to prevent unbounded memory growth
// 4 * 10_000 * average line size = ~20 MB
const MAX_CONCURRENT_FLUSHES: usize = 4;
const DEFAULT_DELIMITER: u8 = b'\n';

/// Collects entries before pushing them to the injector.
#[allow(clippy::unused_async)]
pub async fn load_candidates<P: EntryProcessor>(
    command: CommandSpec,
    entry_delimiter: Option<char>,
    command_index: usize,
    processor: P,
    injector: Injector<P::Data>,
) {
    debug!("Loading candidates from command: {:?}", command);
    let mut std_command = shell_command(
        command.get_nth(command_index).raw(),
        command.interactive,
        &command.env,
        command.shell,
    );
    std_command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = TokioCommand::from(std_command)
        .spawn()
        .expect("failed to execute process"); // FIXME: handle error

    if let Some(out) = child.stdout.take() {
        let mut produced_output = false;
        let mut reader = BufReader::new(out);
        let mut buf = Vec::with_capacity(DEFAULT_LINE_BUFFER_SIZE);
        let mut batch = Vec::with_capacity(BATCH_SIZE);
        let mut flush_handles = tokio::task::JoinSet::new();

        let delimiter = entry_delimiter
            .as_ref()
            .map(|d| *d as u8)
            .unwrap_or(DEFAULT_DELIMITER);

        let mut last_flush = Instant::now();
        while {
            buf.clear();
            let n = reader.read_until(delimiter, &mut buf).await.unwrap_or(0);
            n > 0
        } {
            batch.push(buf.clone());

            // Flush batch when it reaches the target size
            if batch.len() >= BATCH_SIZE
                || last_flush.elapsed() >= UPDATE_INTERVAL
            {
                if flush_handles.len() >= MAX_CONCURRENT_FLUSHES {
                    // Wait for any task to complete
                    let _ = flush_handles.join_next().await;
                }

                let batch_to_flush = std::mem::replace(
                    &mut batch,
                    Vec::with_capacity(BATCH_SIZE),
                );
                let inj = injector.clone();
                let proc = processor.clone();
                flush_handles.spawn_blocking(move || {
                    flush_batch(batch_to_flush, &inj, &proc, delimiter);
                });
                produced_output = true;
                last_flush = Instant::now();
            }
        }

        debug!("Finished reading command output.");

        // Flush any remaining entries in the batch
        if !batch.is_empty() {
            let inj = injector.clone();
            let proc = processor.clone();
            flush_handles.spawn_blocking(move || {
                flush_batch(batch, &inj, &proc, delimiter);
            });
            produced_output = true;
        }

        // Wait for all remaining flush tasks to complete
        while flush_handles.join_next().await.is_some() {}

        // if the command didn't produce any output, check stderr and display that instead
        if !produced_output {
            let tv_message =
                "Command produced no output on stdout, checking stderr...";
            processor.push_to_injector(tv_message.to_string(), &injector);
            let stderr = child.stderr.take().unwrap();
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if line.trim().is_empty() {
                    continue;
                }
                processor.push_to_injector(line, &injector);
            }
        }
    }
    let _ = child.wait().await;
}

/// Reads lines from process stdin and pushes them to the injector.
///
/// This is used by the stdin channel to read piped input directly in Rust,
/// avoiding platform-specific issues with shell `cat` (e.g. `PowerShell`'s
/// `Get-Content` alias on Windows).
pub async fn load_stdin_candidates<P: EntryProcessor>(
    entry_delimiter: Option<char>,
    processor: P,
    injector: Injector<P::Data>,
) {
    debug!("Loading candidates from stdin");
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut buf = Vec::with_capacity(DEFAULT_LINE_BUFFER_SIZE);
    let mut batch = Vec::with_capacity(BATCH_SIZE);
    let mut flush_handles = tokio::task::JoinSet::new();

    let delimiter = entry_delimiter
        .as_ref()
        .map(|d| *d as u8)
        .unwrap_or(DEFAULT_DELIMITER);

    let mut last_flush = Instant::now();
    while {
        buf.clear();
        let n = reader.read_until(delimiter, &mut buf).await.unwrap_or(0);
        n > 0
    } {
        batch.push(buf.clone());

        if batch.len() >= BATCH_SIZE || last_flush.elapsed() >= UPDATE_INTERVAL
        {
            if flush_handles.len() >= MAX_CONCURRENT_FLUSHES {
                let _ = flush_handles.join_next().await;
            }

            let batch_to_flush =
                std::mem::replace(&mut batch, Vec::with_capacity(BATCH_SIZE));
            let inj = injector.clone();
            let proc = processor.clone();
            flush_handles.spawn_blocking(move || {
                flush_batch(batch_to_flush, &inj, &proc, delimiter);
            });
            last_flush = Instant::now();
        }
    }

    debug!("Finished reading stdin.");

    if !batch.is_empty() {
        let inj = injector.clone();
        let proc = processor.clone();
        flush_handles.spawn_blocking(move || {
            flush_batch(batch, &inj, &proc, delimiter);
        });
    }

    while flush_handles.join_next().await.is_some() {}
}

/// Flushes a batch of entries to the injector.
/// This is called from a blocking task spawned in the threadpool.
fn flush_batch<P: EntryProcessor>(
    batch: Vec<Vec<u8>>,
    injector: &Injector<P::Data>,
    processor: &P,
    delimiter: u8,
) {
    // decode utf8 and filter empty/whitespace-only lines
    for mut bytes in batch {
        if bytes.is_empty() || bytes.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        if bytes.last() == Some(&delimiter) {
            bytes.pop();
        }
        if let Ok(line) = String::from_utf8(bytes) {
            processor.push_to_injector(line, injector);
        }
    }
}

/// Channels can be in one of several modes depending on the source configuration.
///
/// - Plain: no ANSI processing, no display template (uses Matcher<()> for memory efficiency)
/// - Ansi: strips ANSI codes for matching (uses Matcher<String>)
/// - Display: applies custom display template for matching (uses Matcher<String>)
pub enum ChannelKind {
    Plain(Channel<PlainProcessor>),
    Ansi(Channel<AnsiProcessor>),
    Display(Channel<DisplayProcessor>),
}

/// This reduces the boilerplate you'd have to write to have the wrapping enum delegate same
/// implementation methods to the inner channel variants.
///
/// e.g. instead of writing:
/// ```ignore
/// pub fn load(&mut self) {
///     match self {
///         ChannelKind::Plain(ch) => ch.load(),
///         ChannelKind::Ansi(ch) => ch.load(),
///         ChannelKind::Display(ch) => ch.load(),
///     }
/// }
///
/// pub fn current_command(&self) -> &str {
///     match self {
///         ChannelKind::Plain(ch) => ch.current_command(),
///         ChannelKind::Ansi(ch) => ch.current_command(),
///         ChannelKind::Display(ch) => ch.current_command(),
///     }
/// }
/// ```
/// You can just write:
/// ```ignore
/// delegate_to_channel!(mut
///     load() -> (),
/// );
/// delegate_to_channel!(ref
///     current_command() -> &str,
/// );
/// ```
///
/// The `mut` and `ref` keywords indicate whether the method takes `&mut self` or `&self`.
macro_rules! delegate_to_channel {
    // Mutable methods
    (mut $($method:ident($($arg:ident: $arg_ty:ty),*) -> $ret:ty),* $(,)?) => {
        $(
            pub fn $method(&mut self $(, $arg: $arg_ty)*) -> $ret {
                match self {
                    ChannelKind::Plain(ch) => ch.$method($($arg),*),
                    ChannelKind::Ansi(ch) => ch.$method($($arg),*),
                    ChannelKind::Display(ch) => ch.$method($($arg),*),
                }
            }
        )*
    };

    // Immutable methods
    (ref $($method:ident($($arg:ident: $arg_ty:ty),*) -> $ret:ty),* $(,)?) => {
        $(
            pub fn $method(&self $(, $arg: $arg_ty)*) -> $ret {
                match self {
                    ChannelKind::Plain(ch) => ch.$method($($arg),*),
                    ChannelKind::Ansi(ch) => ch.$method($($arg),*),
                    ChannelKind::Display(ch) => ch.$method($($arg),*),
                }
            }
        )*
    };
}

impl ChannelKind {
    /// Creates the appropriate `ChannelKind` variant based on the source configuration.
    ///
    /// This mainly enables us to make some memory savings for the common case of no ANSI processing
    /// and no display template by using `Matcher<()>` instead of `Matcher<String>`.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::fn_params_excessive_bools)]
    pub fn new(
        source_command: CommandSpec,
        source_entry_delimiter: Option<char>,
        source_ansi: bool,
        source_display: Option<Template>,
        source_output: Option<Template>,
        supports_preview: bool,
        no_sort: bool,
        frecency: Option<(FrecencyHandle, String)>,
        is_stdin: bool,
    ) -> Self {
        match (source_ansi, source_display) {
            (false, None) => ChannelKind::Plain(Channel::new(
                source_command,
                source_entry_delimiter,
                source_output,
                supports_preview,
                no_sort,
                PlainProcessor,
                frecency,
                is_stdin,
            )),
            (true, None) => ChannelKind::Ansi(Channel::new(
                source_command,
                source_entry_delimiter,
                source_output,
                supports_preview,
                no_sort,
                AnsiProcessor,
                frecency,
                is_stdin,
            )),
            (_, Some(template)) => ChannelKind::Display(Channel::new(
                source_command,
                source_entry_delimiter,
                source_output,
                supports_preview,
                no_sort,
                DisplayProcessor { template },
                frecency,
                is_stdin,
            )),
        }
    }

    // Generate all mutable delegation methods
    delegate_to_channel!(mut
        load() -> (),
        reload() -> (),
        find(pattern: &str) -> (),
        tick() -> (),
        results(num_entries: u32, offset: u32) -> Vec<Entry>,
        get_result(index: u32) -> Option<Entry>,
        toggle_selection(entry: &Entry) -> (),
        cycle_sources() -> (),
    );

    // Generate all immutable delegation methods
    delegate_to_channel!(ref
        current_command() -> &str,
        selected_entries() -> &FxHashSet<Entry>,
        result_count() -> u32,
        total_count() -> u32,
        running() -> bool,
        shutdown() -> (),
        supports_preview() -> bool,
        source_index() -> usize,
        source_count() -> usize,
        is_stdin() -> bool,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        channels::prototypes::SourceSpec,
        matcher::config::{Config, SortStrategy},
    };

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    async fn test_load_candidates_default_delimiter() {
        let source_spec: SourceSpec = toml::from_str(
            r#"
            command = "echo 'test1\ntest2\ntest3'"
            "#,
        )
        .unwrap();

        // Use PlainProcessor for no ansi, no display
        let mut matcher =
            Matcher::<()>::new(&Config::default(), SortStrategy::Score);
        let injector = matcher.injector();

        load_candidates(
            source_spec.command,
            source_spec.entry_delimiter,
            0,
            PlainProcessor,
            injector,
        )
        .await;

        // Check if the matcher has the expected results
        matcher.find("test");
        matcher.tick();
        let results = matcher.results(10, 0);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].matched_string, "test1");
        assert_eq!(results[1].matched_string, "test2");
        assert_eq!(results[2].matched_string, "test3");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    async fn test_load_candidates_null_byte_delimiter() {
        let source_spec: SourceSpec = toml::from_str(
            r#"command = "printf 'test1\\0test2\\0test3\\0'"
            entry_delimiter = "\\0""#,
        )
        .unwrap();

        let mut matcher =
            Matcher::<()>::new(&Config::default(), SortStrategy::Score);
        let injector = matcher.injector();

        load_candidates(
            source_spec.command,
            source_spec.entry_delimiter,
            0,
            PlainProcessor,
            injector,
        )
        .await;

        // Check if the matcher has the expected results
        matcher.find("test");
        matcher.tick();
        let results = matcher.results(10, 0);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].matched_string, "test1");
        assert_eq!(results[1].matched_string, "test2");
        assert_eq!(results[2].matched_string, "test3");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    async fn test_load_candidates_null_byte_and_newlines() {
        let source_spec: SourceSpec = toml::from_str(
            r#"command = "printf 'test1\\0test2\\ntest3\\0'"
            entry_delimiter = "\\0""#,
        )
        .unwrap();

        let mut matcher =
            Matcher::<()>::new(&Config::default(), SortStrategy::Score);
        let injector = matcher.injector();

        load_candidates(
            source_spec.command,
            source_spec.entry_delimiter,
            0,
            PlainProcessor,
            injector,
        )
        .await;

        // Check if the matcher has the expected results
        matcher.find("test");
        matcher.tick();
        let results = matcher.results(10, 0);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].matched_string, "test1");
        assert_eq!(results[1].matched_string, "test2\ntest3");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    async fn test_load_candidates_large_input() {
        // Test with more entries than the batch size
        let source_spec: SourceSpec = toml::from_str(
            r#"
            command = "seq 1 1000"
            "#,
        )
        .unwrap();

        let mut matcher =
            Matcher::<()>::new(&Config::default(), SortStrategy::Score);
        let injector = matcher.injector();

        load_candidates(
            source_spec.command,
            source_spec.entry_delimiter,
            0,
            PlainProcessor,
            injector,
        )
        .await;

        // Check if the matcher has the expected results
        matcher.find("");
        matcher.tick();
        let results = matcher.results(1000, 0);
        assert_eq!(results.len(), 1000);
        assert_eq!(results[0].matched_string, "1");
        assert_eq!(results[999].matched_string, "1000");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    async fn test_load_candidates_with_ansi() {
        let source_spec: SourceSpec = toml::from_str(
            r#"
            command = "printf '\\x1b[31mtest1\\x1b[0m\\n\\x1b[32mtest2\\x1b[0m\\n\\x1b[33mtest3\\x1b[0m\\n'"
            ansi = true
            "#,
        )
        .unwrap();

        let mut matcher =
            Matcher::<String>::new(&Config::default(), SortStrategy::Score);
        let injector = matcher.injector();

        load_candidates(
            source_spec.command,
            source_spec.entry_delimiter,
            0,
            AnsiProcessor,
            injector,
        )
        .await;

        // Check if the matcher has the expected results (ANSI codes should be stripped)
        matcher.find("test");
        matcher.tick();
        let results = matcher.results(10, 0);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].matched_string, "test1");
        assert_eq!(results[1].matched_string, "test2");
        assert_eq!(results[2].matched_string, "test3");
    }
}
