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
        Matcher, Notify, SortStrategy, injector::Injector, matcher_threads,
    },
    utils::command::shell_command,
};
use rustc_hash::{FxBuildHasher, FxHashSet};
use std::collections::HashSet;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use tokio::process::Command as TokioCommand;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader},
    time::Instant,
};
use tracing::debug;

const RELOAD_RENDERING_DELAY: Duration = Duration::from_millis(200);

pub struct Channel<P: EntryProcessor> {
    pub source_command: CommandSpec,
    pub source_entry_delimiter: Option<char>,
    pub source_output: Option<Template>,
    pub supports_preview: bool,
    processor: P,
    matcher: Matcher<P::Data>,
    selected_entries: FxHashSet<Entry>,
    crawl_handle: Option<tokio::task::JoinHandle<()>>,
    current_source_index: usize,
    /// Indicates if the channel is currently reloading to prevent UI flickering
    /// by delaying the rendering of a new frame.
    pub reloading: Arc<AtomicBool>,
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
        notify: Notify,
    ) -> Self {
        let sort_strategy = if no_sort {
            SortStrategy::Index
        } else if let Some((frecency_handle, channel_name)) = frecency {
            let cache = frecency_handle.create_cache(channel_name);
            SortStrategy::Hoisted {
                table: Box::new(move || cache.table()),
                key: Box::new(P::frecency_key),
            }
        } else {
            SortStrategy::Score
        };

        let matcher =
            Matcher::with_notify(sort_strategy, matcher_threads(), notify);
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
            reloading: Arc::new(AtomicBool::new(false)),
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

    pub fn reload(&mut self) {
        if self.is_stdin {
            debug!("Stdin channel cannot be reloaded, skipping.");
            return;
        }
        if self.reloading.load(std::sync::atomic::Ordering::Relaxed) {
            debug!("Reload already in progress, skipping.");
            return;
        }
        self.reloading
            .store(true, std::sync::atomic::Ordering::Relaxed);

        if let Some(handle) = self.crawl_handle.take()
            && !handle.is_finished()
        {
            handle.abort();
        }
        self.matcher.restart();
        self.load();
        // Spawn a thread that turns off reloading after a short delay
        // to avoid UI flickering (this boolean is used by `Television::should_render`)
        let reloading = self.reloading.clone();
        tokio::spawn(async move {
            tokio::time::sleep(RELOAD_RENDERING_DELAY).await;
            reloading.store(false, std::sync::atomic::Ordering::Relaxed);
        });
    }

    pub fn current_command(&self) -> &str {
        self.source_command
            .get_nth(self.current_source_index)
            .template()
            .raw()
    }

    pub fn current_source_name(&self) -> Option<&str> {
        self.source_command
            .get_nth(self.current_source_index)
            .name()
    }

    pub fn find(&mut self, pattern: &str) {
        self.matcher.find(pattern);
    }

    pub fn results(&mut self, num_entries: u32, offset: u32) -> Vec<Entry> {
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
        self.matcher.matched_item_count()
    }

    pub fn total_count(&self) -> u32 {
        self.matcher.total_item_count()
    }

    pub fn running(&self) -> bool {
        self.matcher.running()
            || (self.crawl_handle.is_some()
                && !self.crawl_handle.as_ref().unwrap().is_finished())
    }

    pub fn wait_for_idle(&self) {
        self.matcher.wait_for_idle();
    }

    pub fn wait_for_idle_timeout(&self, timeout: Duration) {
        self.matcher.wait_for_idle_timeout(timeout);
    }

    /// Stop the source: abort the reader task (which kills the source
    /// process via `kill_on_drop`) and drop the store. Batches still in
    /// flight are discarded by the matcher's generation check.
    pub fn shutdown(&mut self) {
        if let Some(handle) = self.crawl_handle.take() {
            handle.abort();
        }
        self.matcher.restart();
    }

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

    pub fn reloading(&self) -> bool {
        self.reloading.load(std::sync::atomic::Ordering::Relaxed)
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

// Read the source's output in chunks of at least this size: reading in bulk
// instead of line by line keeps the per-line overhead (syscalls, timestamps,
// allocations) off the reader loop
const READ_CHUNK_SIZE: usize = 64 * 1024;
// Flush accumulated bytes to a processing task after this size
// (~100k entries at 40 bytes per line)
const FLUSH_SIZE: usize = 4 * 1024 * 1024;
// Automatically flush after this interval so first results reach the
// screen quickly on slow sources
const UPDATE_INTERVAL: Duration = Duration::from_millis(200);
// Maximum number of concurrent flush tasks to prevent unbounded memory growth
// 4 * ~2x FLUSH_SIZE (raw bytes + processed entries) = ~32 MB
const MAX_CONCURRENT_FLUSHES: usize = 4;
const DEFAULT_DELIMITER: u8 = b'\n';

/// Reads `reader` in large chunks and ships complete entries to blocking
/// tasks that split, process and push them to the injector in batches.
///
/// Returns whether the reader produced any output.
async fn stream_entries<R, P>(
    mut reader: R,
    delimiter: u8,
    processor: &P,
    injector: &Injector<P::Data>,
) -> bool
where
    R: AsyncRead + Unpin,
    P: EntryProcessor,
{
    let mut acc: Vec<u8> = Vec::new();
    let mut flush_handles = tokio::task::JoinSet::new();
    let mut produced_output = false;
    let mut last_flush = Instant::now();

    loop {
        acc.reserve(READ_CHUNK_SIZE);
        let n = reader.read_buf(&mut acc).await.unwrap_or(0);
        if n == 0 {
            break;
        }

        if acc.len() >= FLUSH_SIZE || last_flush.elapsed() >= UPDATE_INTERVAL {
            // Only complete entries are flushed: bytes after the last
            // delimiter stay in the accumulator
            if let Some(pos) = memchr::memrchr(delimiter, &acc) {
                let rest = acc.split_off(pos + 1);
                let chunk = std::mem::replace(&mut acc, rest);

                if flush_handles.len() >= MAX_CONCURRENT_FLUSHES {
                    // Wait for any task to complete
                    let _ = flush_handles.join_next().await;
                }
                let inj = injector.clone();
                let mut proc = processor.clone();
                flush_handles.spawn_blocking(move || {
                    flush_chunk(&chunk, &inj, &mut proc, delimiter);
                });
                produced_output = true;
                last_flush = Instant::now();
            }
        }
    }

    // Flush whatever is left (the last entry may not be delimited)
    if !acc.is_empty() {
        let inj = injector.clone();
        let mut proc = processor.clone();
        flush_handles.spawn_blocking(move || {
            flush_chunk(&acc, &inj, &mut proc, delimiter);
        });
        produced_output = true;
    }

    // Wait for all remaining flush tasks to complete
    while flush_handles.join_next().await.is_some() {}

    produced_output
}

/// Collects entries before pushing them to the injector.
pub async fn load_candidates<P: EntryProcessor>(
    command: CommandSpec,
    entry_delimiter: Option<char>,
    command_index: usize,
    mut processor: P,
    injector: Injector<P::Data>,
) {
    debug!("Loading candidates from command: {:?}", command);
    let mut std_command = shell_command(
        command.get_nth(command_index).template().raw(),
        command.interactive,
        &command.env,
        command.shell,
    );
    std_command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = TokioCommand::from(std_command)
        // Kill the source process when the reader task is dropped (reload,
        // channel switch, quit) instead of letting it run to completion
        .kill_on_drop(true)
        .spawn()
        .expect("failed to execute process"); // FIXME: handle error

    if let Some(out) = child.stdout.take() {
        let delimiter = entry_delimiter
            .as_ref()
            .map(|d| *d as u8)
            .unwrap_or(DEFAULT_DELIMITER);

        let produced_output =
            stream_entries(out, delimiter, &processor, &injector).await;

        debug!("Finished reading command output.");

        // if the command didn't produce any output, check stderr and display that instead
        if !produced_output {
            let tv_message =
                "Command produced no output on stdout, checking stderr...";
            let (data, haystack) = processor.process(tv_message.to_string());
            injector.push(data, haystack);
            let stderr = child.stderr.take().unwrap();
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if line.trim().is_empty() {
                    continue;
                }
                let (data, haystack) = processor.process(line);
                injector.push(data, haystack);
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

    let delimiter = entry_delimiter
        .as_ref()
        .map(|d| *d as u8)
        .unwrap_or(DEFAULT_DELIMITER);

    stream_entries(stdin, delimiter, &processor, &injector).await;

    debug!("Finished reading stdin.");
}

/// Splits a chunk of complete entries on `delimiter`, filters
/// empty/whitespace-only lines and runs the processor up front so the whole
/// chunk is pushed under a single injector call.
/// This is called from a blocking task spawned in the threadpool.
fn flush_chunk<P: EntryProcessor>(
    chunk: &[u8],
    injector: &Injector<P::Data>,
    processor: &mut P,
    delimiter: u8,
) {
    let mut entries = Vec::new();
    let mut start = 0;
    for end in memchr::memchr_iter(delimiter, chunk)
        .chain(std::iter::once(chunk.len()))
    {
        let line = &chunk[start..end];
        start = end + 1;
        if line.is_empty() || line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        if let Ok(line) = std::str::from_utf8(line) {
            entries.push(processor.process(line.to_string()));
        }
    }
    injector.push_batch(entries);
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
        notify: Notify,
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
                notify,
            )),
            (true, None) => ChannelKind::Ansi(Channel::new(
                source_command,
                source_entry_delimiter,
                source_output,
                supports_preview,
                no_sort,
                AnsiProcessor::new(),
                frecency,
                is_stdin,
                notify,
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
                notify,
            )),
        }
    }

    // Generate all mutable delegation methods
    delegate_to_channel!(mut
        load() -> (),
        reload() -> (),
        find(pattern: &str) -> (),
        results(num_entries: u32, offset: u32) -> Vec<Entry>,
        get_result(index: u32) -> Option<Entry>,
        toggle_selection(entry: &Entry) -> (),
        cycle_sources() -> (),
        shutdown() -> (),
    );

    // Generate all immutable delegation methods
    delegate_to_channel!(ref
        current_command() -> &str,
        current_source_name() -> Option<&str>,
        selected_entries() -> &FxHashSet<Entry>,
        result_count() -> u32,
        total_count() -> u32,
        running() -> bool,
        wait_for_idle() -> (),
        wait_for_idle_timeout(timeout: Duration) -> (),
        supports_preview() -> bool,
        reloading() -> bool,
        source_index() -> usize,
        source_count() -> usize,
        is_stdin() -> bool,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channels::prototypes::SourceSpec;
    use crate::utils::ansi::StyleRuns;

    const MATCHER_TEST_THREADS: usize = 1;

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
            Matcher::<()>::new(SortStrategy::Score, MATCHER_TEST_THREADS);
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
        matcher.wait_for_idle();
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
            Matcher::<()>::new(SortStrategy::Score, MATCHER_TEST_THREADS);
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
        matcher.wait_for_idle();
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
            Matcher::<()>::new(SortStrategy::Score, MATCHER_TEST_THREADS);
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
        matcher.wait_for_idle();
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
            Matcher::<()>::new(SortStrategy::Score, MATCHER_TEST_THREADS);
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
        matcher.wait_for_idle();
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

        let mut matcher = Matcher::<StyleRuns>::new(
            SortStrategy::Score,
            MATCHER_TEST_THREADS,
        );
        let injector = matcher.injector();

        load_candidates(
            source_spec.command,
            source_spec.entry_delimiter,
            0,
            AnsiProcessor::new(),
            injector,
        )
        .await;

        // Check if the matcher has the expected results (ANSI codes should be stripped)
        matcher.find("test");
        matcher.wait_for_idle();
        let results = matcher.results(10, 0);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].matched_string, "test1");
        assert_eq!(results[1].matched_string, "test2");
        assert_eq!(results[2].matched_string, "test3");
    }
}
