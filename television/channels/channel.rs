use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use std::process::Stdio;

use rustc_hash::{FxBuildHasher, FxHashSet};
use tracing::debug;

use crate::channels::prototypes::SourceSpec;
use crate::channels::{entry::Entry, prototypes::ChannelPrototype};
use crate::matcher::Matcher;
use crate::matcher::{config::Config, injector::Injector};
use crate::utils::command::shell_command;

pub struct Channel {
    pub prototype: ChannelPrototype,
    matcher: Matcher<String>,
    selected_entries: FxHashSet<Entry>,
    crawl_handle: Option<tokio::task::JoinHandle<()>>,
    current_source_index: usize,
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
        }
    }

    pub fn load(&mut self) {
        let injector = self.matcher.injector();
        let crawl_handle = tokio::spawn(load_candidates(
            self.prototype.source.clone(),
            self.current_source_index,
            injector,
        ));
        self.crawl_handle = Some(crawl_handle);
    }

    pub fn reload(&mut self) {
        if let Some(handle) = self.crawl_handle.take() {
            if !handle.is_finished() {
                handle.abort();
            }
        }
        self.matcher.restart();
        self.load();
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

    pub fn results(&mut self, num_entries: u32, offset: u32) -> Vec<Entry> {
        self.matcher.tick();

        let results = self.matcher.results(num_entries, offset);

        let mut entries = Vec::with_capacity(results.len());

        for item in results {
            let entry = Entry::new(item.inner)
                .with_display(item.matched_string)
                .with_match_indices(&item.match_indices);
            entries.push(entry);
        }

        entries
    }

    pub fn get_result(&self, index: u32) -> Entry {
        let item = self.matcher.get_result(index).expect("Invalid index");
        let mut entry = Entry::new(item.inner.clone())
            .with_display(item.matched_string)
            .with_match_indices(&item.match_indices);
        if let Some(p) = &self.prototype.preview {
            // FIXME: this should be done by the previewer instead
            if let Some(offset_expr) = &p.offset {
                let offset_str = offset_expr
                    .format(&item.inner)
                    .unwrap_or_else(|_| panic!("Failed to format offset expression '{}' with name '{}'", offset_expr.raw(), item.inner));

                entry = entry.with_line_number(
                    offset_str.parse::<usize>().unwrap_or(0),
                );
            }
        }
        entry
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

#[allow(clippy::unused_async)]
async fn load_candidates(
    source: SourceSpec,
    source_command_index: usize,
    injector: Injector<String>,
) {
    debug!("Loading candidates from command: {:?}", source.command);
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
        let reader = BufReader::new(out);
        let mut produced_output = false;

        #[allow(clippy::manual_flatten)]
        for line in reader.lines() {
            if let Ok(l) = line {
                if !l.trim().is_empty() {
                    // PERF: Optimize string handling - avoid unnecessary clones
                    let entry_string = if let Some(display) = &source.display {
                        display.format(&l).unwrap_or_else(|_| {
                            panic!(
                                "Failed to format display expression '{}' with entry '{}'",
                                display.raw(),
                                l
                            )
                        })
                    } else {
                        l
                    };

                    let () = injector.push(entry_string, |e, cols| {
                        // PERF: Reduced cloning by handling the string more efficiently
                        cols[0] = e.clone().into();
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
