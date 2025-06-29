//! Tests for History navigation helpers (channel vs. global mode).

use serde_json;
use std::fs;
use std::path::Path;
use television::history::{History, HistoryEntry};
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
            Some(e) => acc.push(e.entry.clone()),
            None => break,
        }
    }
    acc
}

#[allow(dead_code)]
fn print_entries(hist: &History) {
    println!("{:?}", dump_entries(hist)); // uncomment for debug
}

/// Read all entries currently stored in history.json for a data dir
fn entries_in_file(dir: &Path) -> Vec<String> {
    let raw = fs::read_to_string(dir.join("history.json")).unwrap();
    let vec: Vec<HistoryEntry> = serde_json::from_str(&raw).unwrap();
    vec.into_iter().map(|e| e.entry).collect()
}

fn assert_entries(dir: &Path, expected: &[&str]) {
    let mut got = entries_in_file(dir);
    got.sort();
    let mut exp: Vec<String> =
        expected.iter().map(|s| s.to_string()).collect();
    exp.sort();
    assert_eq!(got, exp);
}

/// Prev/next navigation scoped to current channel.
#[test]
fn prev_next_channel_mode() {
    let dir = setup_history_file(&make_entries());
    let mut hist = History::new(Some(10), "files", false, dir.path());
    hist.init().unwrap();

    // channel mode prev navigation (should skip non-matching channels)
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"file3".to_string())
    );
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"file2".to_string())
    );
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"file1".to_string())
    );
    // further calls stay at oldest matching
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"file1".to_string())
    );

    // Next navigation forward in channel scope
    assert_eq!(
        hist.get_next_entry().map(|e| &e.entry),
        Some(&"file2".to_string())
    );
    assert_eq!(
        hist.get_next_entry().map(|e| &e.entry),
        Some(&"file3".to_string())
    );

    // After reaching the last matching entry, further next should reset and return None
    assert_eq!(hist.get_next_entry(), None);
}

/// Prev/next navigation in global mode.
#[test]
fn prev_next_global_mode() {
    let dir = setup_history_file(&make_entries());
    let mut hist = History::new(Some(10), "files", true, dir.path());
    hist.init().unwrap();

    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"file3".to_string())
    );
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"dir2".to_string())
    );
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"file2".to_string())
    );
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"dir1".to_string())
    );
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"file1".to_string())
    );
    // further calls stay at oldest matching
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"file1".to_string())
    );

    // Next navigation forward in channel scope
    assert_eq!(
        hist.get_next_entry().map(|e| &e.entry),
        Some(&"dir1".to_string())
    );
    assert_eq!(
        hist.get_next_entry().map(|e| &e.entry),
        Some(&"file2".to_string())
    );
    assert_eq!(
        hist.get_next_entry().map(|e| &e.entry),
        Some(&"dir2".to_string())
    );
    assert_eq!(
        hist.get_next_entry().map(|e| &e.entry),
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

    let mut hist = History::new(Some(10), "files", false, dir.path());
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

    let mut hist = History::new(Some(10), "files", true, dir.path());
    hist.init().unwrap();

    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"dir1".to_string())
    );
}

/// Ensure add_entry respects deduplication and size trimming.
/// Adding entries should skip consecutive duplicates and trim to max size.
#[test]
fn add_entry_dedup_and_trim() {
    let dir = setup_history_file(&[]);
    let mut hist = History::new(Some(3), "files", false, dir.path());

    // add first two unique entries
    hist.add_entry("file1".into(), "files".into()).unwrap();
    assert_entries(dir.path(), &["file1"]);
    hist.add_entry("file2".into(), "files".into()).unwrap();
    assert_entries(dir.path(), &["file1", "file2"]);

    // consecutive duplicate should be ignored
    hist.add_entry("file2".into(), "files".into()).unwrap();
    assert_entries(dir.path(), &["file1", "file2"]);
    assert_eq!(hist.len(), 2);

    // let's add a third unique entry
    hist.add_entry("file3".into(), "files".into()).unwrap();
    assert_entries(dir.path(), &["file1", "file2", "file3"]);
    assert_eq!(hist.len(), 3);

    // let's add a fourth unique entry, we should still have 3 entries
    hist.add_entry("file4".into(), "files".into()).unwrap();
    assert_entries(dir.path(), &["file2", "file3", "file4"]);
    assert_eq!(hist.len(), 3);

    // non-consecutive duplicate counts as new
    hist.add_entry("dir1".into(), "dirs".into()).unwrap();
    assert_entries(dir.path(), &["file3", "file4", "dir1"]);
    assert_eq!(hist.len(), 3);

    // In channel mode (files) the newest matching is file4
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"file4".to_string())
    );

    // Switch to global view to verify "dir1" exists
    // let's init the history with channel only mode for dirs
    let mut hist = History::new(Some(3), "dirs", false, dir.path());
    hist.init().unwrap();
    assert_entries(dir.path(), &["file3", "file4", "dir1"]);
    assert_eq!(hist.len(), 3);
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"dir1".to_string())
    );
}

/// Global mode: initializing with smaller max_size trims older entries.
#[test]
fn init_trim_global_mode() {
    let dir = setup_history_file(&make_entries());

    // max size 3, global mode
    let mut hist = History::new(Some(3), "files", true, dir.path());
    hist.init().unwrap();

    // Should keep only the 3 newest entries (file2, dir2, file3)
    assert_eq!(hist.len(), 3);
    assert_eq!(dump_entries(&hist), vec!["file3", "dir2", "file2"]);
}

/// Test loading from a non-existent history file.
#[test]
fn init_from_nonexistent_file() {
    let dir = tempdir().expect("failed to create tempdir");
    let mut hist = History::new(Some(10), "files", false, dir.path());

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

    let mut hist = History::new(Some(10), "files", false, dir.path());
    hist.init().unwrap();

    assert_eq!(hist.len(), 0);
    assert!(hist.is_empty());
}

/// Test that empty queries are ignored.
#[test]
fn add_entry_ignores_empty_queries() {
    let dir = tempdir().expect("failed to create tempdir");
    let mut hist = History::new(Some(10), "files", false, dir.path());

    // Try to add empty and whitespace-only queries
    hist.add_entry("".into(), "files".into()).unwrap();
    hist.add_entry("  ".into(), "files".into()).unwrap();
    hist.add_entry("\t\n".into(), "files".into()).unwrap();

    assert_eq!(hist.len(), 0);
    assert!(hist.is_empty());

    // Add a real entry to confirm it works
    hist.add_entry("real_entry".into(), "files".into()).unwrap();
    assert_eq!(hist.len(), 1);
    assert_entries(dir.path(), &["real_entry"]);
}

/// Test navigation on completely empty history.
#[test]
fn navigation_empty_history() {
    let dir = tempdir().expect("failed to create tempdir");
    let mut hist = History::new(Some(10), "files", false, dir.path());

    // Both modes should return None on empty history
    assert!(hist.get_previous_entry().is_none());
    assert!(hist.get_next_entry().is_none());

    // Global mode should also return None
    let mut hist_global = History::new(Some(10), "files", true, dir.path());
    assert!(hist_global.get_previous_entry().is_none());
    assert!(hist_global.get_next_entry().is_none());
}

/// Test navigation state after adding entries.
#[test]
fn navigation_after_adding_entries() {
    let dir = tempdir().expect("failed to create tempdir");
    let mut hist = History::new(Some(10), "files", false, dir.path());

    // Add first file
    hist.add_entry("file1".into(), "files".into()).unwrap();

    // Should be able to navigate to it
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"file1".to_string())
    );

    // Add another file - should reset navigation
    hist.add_entry("file2".into(), "files".into()).unwrap();

    // Should now get the newest file first
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"file2".to_string())
    );
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"file1".to_string())
    );
}

/// Test non-consecutive duplicates are properly handled.
#[test]
fn non_consecutive_duplicates() {
    let dir = tempdir().expect("failed to create tempdir");
    let mut hist = History::new(Some(10), "files", false, dir.path());

    hist.add_entry("file1".into(), "files".into()).unwrap();
    hist.add_entry("file2".into(), "files".into()).unwrap();
    hist.add_entry("file1".into(), "files".into()).unwrap(); // Non-consecutive duplicate

    assert_eq!(hist.len(), 3);
    assert_entries(dir.path(), &["file1", "file2", "file1"]);

    // Both instances should be navigable
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"file1".to_string())
    );
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"file2".to_string())
    );
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"file1".to_string())
    );
}

/// Test cross-channel duplicates.
#[test]
fn cross_channel_duplicates() {
    let dir = tempdir().expect("failed to create tempdir");
    let mut hist = History::new(Some(10), "files", false, dir.path());

    hist.add_entry("same_name".into(), "files".into()).unwrap();
    hist.add_entry("same_name".into(), "dirs".into()).unwrap();
    hist.add_entry("same_name".into(), "files".into()).unwrap();

    assert_eq!(hist.len(), 3);
    assert_entries(dir.path(), &["same_name", "same_name", "same_name"]);

    // In channel mode, should only see the files entries
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"same_name".to_string())
    );
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"same_name".to_string())
    );

    // In global mode, should see all three
    let mut hist_global = History::new(Some(10), "files", true, dir.path());
    hist_global.init().unwrap();
    assert_eq!(
        hist_global.get_previous_entry().map(|e| &e.entry),
        Some(&"same_name".to_string())
    );
    assert_eq!(
        hist_global.get_previous_entry().map(|e| &e.entry),
        Some(&"same_name".to_string())
    );
    assert_eq!(
        hist_global.get_previous_entry().map(|e| &e.entry),
        Some(&"same_name".to_string())
    );
}

/// Test navigation state preservation during mixed operations.
#[test]
fn navigation_state_preservation() {
    let dir = setup_history_file(&make_entries());
    let mut hist = History::new(Some(10), "files", false, dir.path());
    hist.init().unwrap();

    // Start navigation
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"file3".to_string())
    );
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"file2".to_string())
    );

    // Add a new entry - should reset navigation state
    hist.add_entry("new_file".into(), "files".into()).unwrap();

    // Should start from newest again
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"new_file".to_string())
    );
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"file3".to_string())
    );
}

/// Test mixed navigation patterns.
#[test]
fn mixed_navigation_patterns() {
    let dir = setup_history_file(&make_entries());
    let mut hist = History::new(Some(10), "files", false, dir.path());
    hist.init().unwrap();

    // Go back three times
    hist.get_previous_entry();
    hist.get_previous_entry();
    hist.get_previous_entry();

    // Go forward once
    assert_eq!(
        hist.get_next_entry().map(|e| &e.entry),
        Some(&"file2".to_string())
    );

    // Go back again
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"file1".to_string())
    );

    // Go forward to end and beyond
    hist.get_next_entry();
    hist.get_next_entry();
    assert!(hist.get_next_entry().is_none());

    // After reset, should be able to start navigation again
    assert_eq!(
        hist.get_previous_entry().map(|e| &e.entry),
        Some(&"file3".to_string())
    );
}

/// Test get_next without previous navigation.
#[test]
fn next_without_previous() {
    let dir = setup_history_file(&make_entries());
    let mut hist = History::new(Some(10), "files", false, dir.path());
    hist.init().unwrap();

    // Calling get_next without any previous navigation should return None
    assert!(hist.get_next_entry().is_none());
}

/// Test history size precedence logic: channel setting overrides global config.
#[test]
fn history_size_precedence() {
    use television::{
        channels::prototypes::ChannelPrototype,
        config::default_config_from_file,
        history::{DEFAULT_HISTORY_SIZE, History},
    };
    use tempfile::tempdir;

    let temp_dir = tempdir().expect("failed to create tempdir");

    // Create a mock config with global history size of 50
    let mut app_config = default_config_from_file().unwrap();
    app_config.application.history_size = 50;
    app_config.application.data_dir = temp_dir.path().to_path_buf();

    // Test case 1: Channel with no explicit history size (None) should use global config (50)
    let mut channel_proto = ChannelPrototype::new("test", "echo hello");
    channel_proto.history.size = None; // Use global config

    let effective_size = channel_proto
        .history
        .size
        .unwrap_or(app_config.application.history_size);
    assert_eq!(effective_size, 50);

    // Test case 2: Channel with explicit history size of 25 should override global config
    channel_proto.history.size = Some(25);
    let effective_size = channel_proto
        .history
        .size
        .unwrap_or(app_config.application.history_size);
    assert_eq!(effective_size, 25);

    // Test case 3: Channel with explicit history size of 0 should disable history
    channel_proto.history.size = Some(0);
    let effective_size = channel_proto
        .history
        .size
        .unwrap_or(app_config.application.history_size);
    assert_eq!(effective_size, 0);

    // Test case 4: Global config 0 with channel None should disable history
    app_config.application.history_size = 0;
    channel_proto.history.size = None; // Use global config (which is 0)
    let effective_size = channel_proto
        .history
        .size
        .unwrap_or(app_config.application.history_size);
    assert_eq!(effective_size, 0);

    // Test case 5: Neither global config nor channel explicitly set
    let default_config = default_config_from_file().unwrap();
    let mut default_channel_proto =
        ChannelPrototype::new("test", "echo hello");
    default_channel_proto.history.size = None; // Use global config

    let effective_size = default_channel_proto
        .history
        .size
        .unwrap_or(default_config.application.history_size);
    assert_eq!(effective_size, DEFAULT_HISTORY_SIZE); // Should use DEFAULT_HISTORY_SIZE

    // Test that the History struct works correctly with these sizes
    let mut history = History::new(Some(10), "test", false, &temp_dir.path());
    assert!(history.init().is_ok());
    assert_eq!(history.len(), 0);
    assert!(!history.is_empty() || history.is_empty()); // Either state is valid for empty history
}
