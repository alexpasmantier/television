use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::debug;

const HISTORY_FILE_NAME: &str = "history.json";
pub const DEFAULT_HISTORY_SIZE: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// The search query/pattern that was typed
    pub query: String,
    /// The channel that the entry belongs to
    pub channel: String,
    /// The timestamp of the entry
    pub timestamp: u64,
}

impl PartialEq for HistoryEntry {
    fn eq(&self, other: &Self) -> bool {
        self.query == other.query && self.channel == other.channel
    }
}

impl HistoryEntry {
    pub fn new(entry: String, channel: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            query: entry,
            channel,
            timestamp,
        }
    }
}

#[derive(Debug, Clone)]
pub struct History {
    entries: Vec<HistoryEntry>,
    current_index: Option<usize>,
    max_size: usize,
    file_path: PathBuf,
    current_channel: String,
    global_mode: bool,
}

impl History {
    pub fn new(
        max_size: usize,
        channel_name: &str,
        global_mode: bool,
        data_dir: &Path,
    ) -> Self {
        let file_path = data_dir.join(HISTORY_FILE_NAME);

        Self {
            entries: Vec::with_capacity(max_size),
            current_index: None,
            max_size,
            file_path,
            current_channel: channel_name.to_string(),
            global_mode,
        }
    }

    /// Initialize the history by loading previously persisted entries from disk.
    pub fn init(&mut self) -> Result<()> {
        // if max_size is 0, history is disabled
        if self.max_size > 0 {
            self.load_from_file()?;
        }
        Ok(())
    }

    /// Add a new history entry, if it's not a duplicate.
    pub fn add_entry(&mut self, query: String, channel: String) -> Result<()> {
        if self.max_size == 0 {
            return Ok(());
        }
        // Don't add empty queries
        if query.trim().is_empty() {
            return Ok(());
        }

        // Don't add duplicate consecutive queries
        if let Some(last_entry) = self.entries.last() {
            if last_entry.query == query && last_entry.channel == channel {
                return Ok(());
            }
        }

        // Trim history if it's going to exceed `max_size`
        if self.entries.len() + 1 > self.max_size {
            self.entries.drain(0..=self.entries.len() - self.max_size);
        }

        let history_entry = HistoryEntry::new(query, channel);
        self.entries.push(history_entry);

        // Reset current index when adding new entry
        self.current_index = None;

        Ok(())
    }

    /// Get the previous history entry based on the configured mode.
    pub fn get_previous_entry(&mut self) -> Option<&HistoryEntry> {
        let channel_filter =
            (!self.global_mode).then_some(self.current_channel.as_str());

        let search_end = match self.current_index {
            None => self.entries.len(),
            Some(0) => {
                return self.entries.first().filter(|entry| {
                    channel_filter.is_none_or(|ch| entry.channel == ch)
                });
            }
            Some(i) => i,
        };

        self.entries
            .get(..search_end)?
            .iter()
            .enumerate()
            .rev()
            .find(|(_, entry)| {
                channel_filter.is_none_or(|ch| entry.channel == ch)
            })
            .map(|(idx, entry)| {
                self.current_index = Some(idx);
                entry
            })
    }

    /// Get the next history entry based on the configured mode.
    pub fn get_next_entry(&mut self) -> Option<&HistoryEntry> {
        let channel_filter =
            (!self.global_mode).then_some(self.current_channel.as_str());
        let search_start = self.current_index? + 1;

        self.entries
            .get(search_start..)?
            .iter()
            .enumerate()
            .find(|(_, entry)| {
                channel_filter.is_none_or(|ch| entry.channel == ch)
            })
            .map(|(offset, entry)| {
                self.current_index = Some(search_start + offset);
                entry
            })
            .or_else(|| {
                self.current_index = None; // Reset navigation at end
                None
            })
    }

    fn load_from_file(&mut self) -> Result<()> {
        if !self.file_path.exists() {
            debug!("History file not found: {}", self.file_path.display());
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.file_path)?;
        if content.trim().is_empty() {
            debug!("History file is empty: {}", self.file_path.display());
            return Ok(());
        }

        let mut loaded_entries: Vec<HistoryEntry> =
            serde_json::from_str(&content)?;

        // Keep only the most recent entries if file is too large
        if loaded_entries.len() > self.max_size {
            loaded_entries.drain(0..loaded_entries.len() - self.max_size);
        }

        self.entries = loaded_entries;
        Ok(())
    }

    pub fn save_to_file(&self) -> Result<()> {
        if self.max_size == 0 {
            debug!("History is disabled, not saving to file.");
            return Ok(());
        }
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json_content = serde_json::to_string_pretty(&self.entries)?;
        std::fs::write(&self.file_path, json_content)?;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get all entries in the history.
    pub fn get_entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    /// Update the current channel context for this history instance.
    pub fn update_channel_context(
        &mut self,
        channel_name: &str,
        global_mode: bool,
    ) {
        self.current_channel = channel_name.to_string();
        self.global_mode = global_mode;

        // Reset navigation state when switching channels
        self.current_index = None;
    }
}

#[cfg(test)]
mod tests {

    //! Tests for History navigation helpers (channel vs. global mode).

    use crate::history::{History, HistoryEntry};
    use serde_json;
    use std::{fs, path::Path};
    use tempfile::tempdir;

    /// Helper to create a history.json file with the provided entries inside a temp data dir.
    fn setup_history_file(entries: &[HistoryEntry]) -> tempfile::TempDir {
        let dir = tempdir().expect("failed to create tempdir");
        let json = serde_json::to_string_pretty(entries).unwrap();
        fs::write(dir.path().join("history.json"), json).unwrap();
        dir
    }

    fn make_entries() -> Vec<HistoryEntry> {
        vec![
            HistoryEntry::new("file1".into(), "files".into()),
            HistoryEntry::new("dir1".into(), "dirs".into()),
            HistoryEntry::new("file2".into(), "files".into()),
            HistoryEntry::new("dir2".into(), "dirs".into()),
            HistoryEntry::new("file3".into(), "files".into()),
        ]
    }

    /// Return a Vec of entry strings in newest→oldest order.
    #[allow(dead_code)]
    fn dump_entries(hist: &History) -> Vec<String> {
        let mut h = hist.clone();
        let mut acc = Vec::new();
        let max = h.len();
        for _ in 0..max {
            match h.get_previous_entry() {
                Some(e) => acc.push(e.query.clone()),
                None => break,
            }
        }
        acc
    }

    #[allow(dead_code)]
    fn print_entries(hist: &History) {
        println!("{:?}", dump_entries(hist));
    }

    /// Read all entries currently stored in history.json for a data dir
    fn entries_in_file(dir: &Path) -> Vec<String> {
        let raw = fs::read_to_string(dir.join("history.json")).unwrap();
        let vec: Vec<HistoryEntry> = serde_json::from_str(&raw).unwrap();
        vec.into_iter().map(|e| e.query).collect()
    }

    #[allow(dead_code)]
    fn assert_entries_in_file(dir: &Path, expected: &[&str]) {
        let mut got = entries_in_file(dir);
        got.sort();
        let mut exp: Vec<String> =
            expected.iter().map(|&s| s.to_string()).collect();
        exp.sort();
        assert_eq!(got, exp);
    }

    #[allow(dead_code)]
    fn assert_entries(hist: &History, expected: &[&str]) {
        let mut got: Vec<String> =
            hist.get_entries().iter().map(|e| e.query.clone()).collect();
        got.sort();
        let mut exp: Vec<String> =
            expected.iter().map(|&s| s.to_string()).collect();
        exp.sort();
        assert_eq!(got, exp);
    }

    /// Prev/next navigation scoped to current channel.
    #[test]
    fn prev_next_channel_mode() {
        let dir = setup_history_file(&make_entries());
        let mut hist = History::new(10, "files", false, dir.path());
        hist.init().unwrap();

        // channel mode prev navigation (should skip non-matching channels)
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"file3".to_string())
        );
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"file2".to_string())
        );
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"file1".to_string())
        );
        // further calls stay at oldest matching
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"file1".to_string())
        );

        // Next navigation forward in channel scope
        assert_eq!(
            hist.get_next_entry().map(|e| &e.query),
            Some(&"file2".to_string())
        );
        assert_eq!(
            hist.get_next_entry().map(|e| &e.query),
            Some(&"file3".to_string())
        );

        // After reaching the last matching entry, further next should reset and return None
        assert_eq!(hist.get_next_entry(), None);
    }

    /// Prev/next navigation in global mode.
    #[test]
    fn prev_next_global_mode() {
        let dir = setup_history_file(&make_entries());
        let mut hist = History::new(10, "files", true, dir.path());
        hist.init().unwrap();

        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"file3".to_string())
        );
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"dir2".to_string())
        );
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"file2".to_string())
        );
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"dir1".to_string())
        );
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"file1".to_string())
        );
        // further calls stay at oldest matching
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"file1".to_string())
        );

        // Next navigation forward in channel scope
        assert_eq!(
            hist.get_next_entry().map(|e| &e.query),
            Some(&"dir1".to_string())
        );
        assert_eq!(
            hist.get_next_entry().map(|e| &e.query),
            Some(&"file2".to_string())
        );
        assert_eq!(
            hist.get_next_entry().map(|e| &e.query),
            Some(&"dir2".to_string())
        );
        assert_eq!(
            hist.get_next_entry().map(|e| &e.query),
            Some(&"file3".to_string())
        );

        // After reaching the last matching entry, further next should reset and return None
        assert_eq!(hist.get_next_entry(), None);
    }

    /// Channel-scoped navigation when the history file contains no entries for the
    /// current channel – navigation should immediately return `None` both ways.
    #[test]
    fn channel_mode_no_matching_entries() {
        let only_b = vec![HistoryEntry::new("dir1".into(), "dirs".into())];
        let dir = setup_history_file(&only_b);

        let mut hist = History::new(10, "files", false, dir.path());
        hist.init().unwrap();

        assert!(hist.get_previous_entry().is_none());
        assert!(hist.get_next_entry().is_none());
    }

    /// Global mode navigation when there is only a single entry that does not
    /// belong to the current channel – that entry should still be accessible.
    #[test]
    fn global_mode_single_nonmatching_entry() {
        let only_b = vec![HistoryEntry::new("dir1".into(), "dirs".into())];
        let dir = setup_history_file(&only_b);

        let mut hist = History::new(10, "files", true, dir.path());
        hist.init().unwrap();

        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"dir1".to_string())
        );
    }

    /// Ensure `add_entry` respects deduplication and size trimming.
    /// Adding entries should skip consecutive duplicates and trim to max size.
    #[test]
    fn add_entry_dedup_and_trim() {
        let dir = setup_history_file(&[]);
        let mut hist = History::new(3, "files", false, dir.path());

        // add first two unique entries
        hist.add_entry("file1".into(), "files".into()).unwrap();
        assert_entries(&hist, &["file1"]);
        hist.add_entry("file2".into(), "files".into()).unwrap();
        assert_entries(&hist, &["file1", "file2"]);

        // consecutive duplicates should be ignored
        hist.add_entry("file2".into(), "files".into()).unwrap();
        assert_entries(&hist, &["file1", "file2"]);
        assert_eq!(hist.len(), 2);

        // let's add a third unique entry
        hist.add_entry("file3".into(), "files".into()).unwrap();
        assert_entries(&hist, &["file1", "file2", "file3"]);
        assert_eq!(hist.len(), 3);

        // let's add a fourth unique entry, we should still have 3 entries
        hist.add_entry("file4".into(), "files".into()).unwrap();
        assert_entries(&hist, &["file2", "file3", "file4"]);
        assert_eq!(hist.len(), 3);

        // non-consecutive duplicate counts as new
        hist.add_entry("dir1".into(), "dirs".into()).unwrap();
        assert_entries(&hist, &["file3", "file4", "dir1"]);
        assert_eq!(hist.len(), 3);

        // In channel mode (files) the newest matching is file4
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"file4".to_string())
        );

        // Persist so we can load later
        hist.save_to_file().unwrap();

        // Switch to global view to verify "dir1" exists
        // let's init the history with channel only mode for dirs
        let mut hist = History::new(3, "dirs", false, dir.path());
        hist.init().unwrap();
        assert_entries(&hist, &["file3", "file4", "dir1"]);
        assert_eq!(hist.len(), 3);
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"dir1".to_string())
        );
    }

    /// Global mode: initializing with smaller `max_size` trims older entries.
    #[test]
    fn init_trim_global_mode() {
        let dir = setup_history_file(&make_entries());

        // max size 3, global mode
        let mut hist = History::new(3, "files", true, dir.path());
        hist.init().unwrap();

        // Should keep only the 3 newest entries (file2, dir2, file3)
        assert_eq!(hist.len(), 3);
        assert_eq!(dump_entries(&hist), vec!["file3", "dir2", "file2"]);
    }

    /// Test loading from a non-existent history file.
    #[test]
    fn init_from_nonexistent_file() {
        let dir = tempdir().expect("failed to create tempdir");
        let mut hist = History::new(10, "files", false, dir.path());

        // Should succeed and have empty entries
        hist.init().unwrap();
        assert_eq!(hist.len(), 0);
        assert!(hist.is_empty());

        // Navigation should return None
        assert!(hist.get_previous_entry().is_none());
        assert!(hist.get_next_entry().is_none());
    }

    /// Test loading from an empty history file.
    #[test]
    fn init_from_empty_file() {
        let dir = tempdir().expect("failed to create tempdir");
        fs::write(dir.path().join("history.json"), "").unwrap();

        let mut hist = History::new(10, "files", false, dir.path());
        hist.init().unwrap();

        assert_eq!(hist.len(), 0);
        assert!(hist.is_empty());
    }

    /// Test that empty queries are ignored.
    #[test]
    fn add_entry_ignores_empty_queries() {
        let dir = tempdir().expect("failed to create tempdir");
        let mut hist = History::new(10, "files", false, dir.path());

        // Try to add empty and whitespace-only queries
        hist.add_entry(String::new(), "files".into()).unwrap();
        hist.add_entry("  ".into(), "files".into()).unwrap();
        hist.add_entry("\t\n".into(), "files".into()).unwrap();

        assert_eq!(hist.len(), 0);
        assert!(hist.is_empty());

        // Add a real entry to confirm it works
        hist.add_entry("real_entry".into(), "files".into()).unwrap();
        assert_eq!(hist.len(), 1);
        assert_entries(&hist, &["real_entry"]);
    }

    /// Test navigation on completely empty history.
    #[test]
    fn navigation_empty_history() {
        let dir = tempdir().expect("failed to create tempdir");
        let mut hist = History::new(10, "files", false, dir.path());

        // Both modes should return None on empty history
        assert!(hist.get_previous_entry().is_none());
        assert!(hist.get_next_entry().is_none());

        // Global mode should also return None
        let mut hist_global = History::new(10, "files", true, dir.path());
        assert!(hist_global.get_previous_entry().is_none());
        assert!(hist_global.get_next_entry().is_none());
    }

    /// Test navigation state after adding entries.
    #[test]
    fn navigation_after_adding_entries() {
        let dir = tempdir().expect("failed to create tempdir");
        let mut hist = History::new(10, "files", false, dir.path());

        // Add first file
        hist.add_entry("file1".into(), "files".into()).unwrap();

        // Should be able to navigate to it
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"file1".to_string())
        );

        // Add another file - should reset navigation
        hist.add_entry("file2".into(), "files".into()).unwrap();

        // Should now get the newest file first
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"file2".to_string())
        );
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"file1".to_string())
        );
    }

    /// Test non-consecutive duplicates are properly handled.
    #[test]
    fn non_consecutive_duplicates() {
        let dir = tempdir().expect("failed to create tempdir");
        let mut hist = History::new(10, "files", false, dir.path());

        hist.add_entry("file1".into(), "files".into()).unwrap();
        hist.add_entry("file2".into(), "files".into()).unwrap();
        hist.add_entry("file1".into(), "files".into()).unwrap(); // Non-consecutive duplicate

        assert_eq!(hist.len(), 3);
        assert_entries(&hist, &["file1", "file2", "file1"]);

        // Both instances should be navigable
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"file1".to_string())
        );
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"file2".to_string())
        );
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"file1".to_string())
        );
    }

    /// Test cross-channel duplicates.
    #[test]
    fn cross_channel_duplicates() {
        let dir = tempdir().expect("failed to create tempdir");
        let mut hist = History::new(10, "files", false, dir.path());

        hist.add_entry("same_name".into(), "files".into()).unwrap();
        hist.add_entry("same_name".into(), "dirs".into()).unwrap();
        hist.add_entry("same_name".into(), "files".into()).unwrap();

        assert_eq!(hist.len(), 3);
        assert_entries(&hist, &["same_name", "same_name", "same_name"]);

        // In channel mode, should only see the files entries
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"same_name".to_string())
        );
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"same_name".to_string())
        );

        // Persist so we can load later
        hist.save_to_file().unwrap();

        // In global mode, should see all three
        let mut hist_global = History::new(10, "files", true, dir.path());
        hist_global.init().unwrap();
        assert_eq!(
            hist_global.get_previous_entry().map(|e| &e.query),
            Some(&"same_name".to_string())
        );
        assert_eq!(
            hist_global.get_previous_entry().map(|e| &e.query),
            Some(&"same_name".to_string())
        );
        assert_eq!(
            hist_global.get_previous_entry().map(|e| &e.query),
            Some(&"same_name".to_string())
        );
    }

    /// Test navigation state preservation during mixed operations.
    #[test]
    fn navigation_state_preservation() {
        let dir = setup_history_file(&make_entries());
        let mut hist = History::new(10, "files", false, dir.path());
        hist.init().unwrap();

        // Start navigation
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"file3".to_string())
        );
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"file2".to_string())
        );

        // Add a new entry - should reset navigation state
        hist.add_entry("new_file".into(), "files".into()).unwrap();

        // Should start from newest again
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"new_file".to_string())
        );
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"file3".to_string())
        );
    }

    /// Test mixed navigation patterns.
    #[test]
    fn mixed_navigation_patterns() {
        let dir = setup_history_file(&make_entries());
        let mut hist = History::new(10, "files", false, dir.path());
        hist.init().unwrap();

        // Go back three times
        hist.get_previous_entry();
        hist.get_previous_entry();
        hist.get_previous_entry();

        // Go forward once
        assert_eq!(
            hist.get_next_entry().map(|e| &e.query),
            Some(&"file2".to_string())
        );

        // Go back again
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"file1".to_string())
        );

        // Go forward to end and beyond
        hist.get_next_entry();
        hist.get_next_entry();
        assert!(hist.get_next_entry().is_none());

        // After reset, should be able to start navigation again
        assert_eq!(
            hist.get_previous_entry().map(|e| &e.query),
            Some(&"file3".to_string())
        );
    }

    /// Test `get_next` without previous navigation.
    #[test]
    fn next_without_previous() {
        let dir = setup_history_file(&make_entries());
        let mut hist = History::new(10, "files", false, dir.path());
        hist.init().unwrap();

        // Calling get_next without any previous navigation should return None
        assert!(hist.get_next_entry().is_none());
    }

    #[test]
    fn add_entry_with_history_size_zero() {
        let dir = tempdir().expect("failed to create tempdir");
        let mut hist = History::new(0, "files", false, dir.path());

        // Adding an entry should not change the history
        hist.add_entry("file1".into(), "files".into()).unwrap();
        assert!(hist.is_empty());
        assert_eq!(hist.len(), 0);

        // Navigation should return None
        assert!(hist.get_previous_entry().is_none());
        assert!(hist.get_next_entry().is_none());
    }
}
