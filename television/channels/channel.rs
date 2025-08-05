use crate::{
    channels::{
        entry::Entry,
        prototypes::{CommandSpec, Template},
    },
    matcher::{Matcher, config::Config, injector::Injector},
    utils::command::shell_command,
};
use rustc_hash::{FxBuildHasher, FxHashSet};
use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use tracing::{debug, trace};

const RELOAD_RENDERING_DELAY: Duration = Duration::from_millis(200);

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
        }
    }

    pub fn load(&mut self) {
        let injector = self.matcher.injector();
        let crawl_handle = tokio::spawn(load_candidates(
            self.source_command.clone(),
            self.source_entry_delimiter,
            self.current_source_index,
            self.source_ansi,
            self.source_display.clone(),
            injector,
        ));
        self.crawl_handle = Some(crawl_handle);
    }

    pub fn reload(&mut self) {
        if self.reloading.load(std::sync::atomic::Ordering::Relaxed) {
            debug!("Reload already in progress, skipping.");
            return;
        }
        self.reloading
            .store(true, std::sync::atomic::Ordering::Relaxed);

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
            reloading.store(false, std::sync::atomic::Ordering::Relaxed);
        });
    }

    pub fn current_command(&self) -> &str {
        self.source_command.get_nth(self.current_source_index).raw()
    }

    pub fn find(&mut self, pattern: &str) {
        self.matcher.find(pattern);
    }

    pub fn results(&mut self, num_entries: u32, offset: u32) -> Vec<Entry> {
        self.matcher.tick();

        let results = self.matcher.results(num_entries, offset);

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
        if let Some(item) = self.matcher.get_result(index) {
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

const DEFAULT_LINE_BUFFER_SIZE: usize = 512;

#[allow(clippy::unused_async)]
async fn load_candidates(
    command: CommandSpec,
    entry_delimiter: Option<char>,
    command_index: usize,
    ansi: bool,
    display: Option<Template>,
    injector: Injector<String>,
) {
    debug!("Loading candidates from command: {:?}", command);
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
                    let () = injector.push(l.to_string(), |e, cols| {
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
                    let () = injector.push(line, |e, cols| {
                        cols[0] = e.clone().into();
                    });
                }
            }
        }
    }
    let _ = child.wait();
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

        load_candidates(
            source_spec.command,
            source_spec.entry_delimiter,
            0,
            source_spec.ansi,
            source_spec.display,
            injector,
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

        load_candidates(
            source_spec.command,
            source_spec.entry_delimiter,
            0,
            source_spec.ansi,
            source_spec.display,
            injector,
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

        load_candidates(
            source_spec.command,
            source_spec.entry_delimiter,
            0,
            source_spec.ansi,
            source_spec.display,
            injector,
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
