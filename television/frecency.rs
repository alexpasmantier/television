//! Frecency-based ranking for previously-selected entries.
//!
//! Frecency combines frequency and recency to rank items. Previously selected
//! entries rank higher than never-selected entries, with more recently selected
//! entries ranking higher within frecency items.

use anyhow::{Context, Result};
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

/// A single frecency record for an entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrecencyRecord {
    /// The raw entry string (canonical key).
    pub raw: String,
    /// Unix timestamp of last access.
    pub last_access: u64,
    /// Number of times this entry has been selected.
    pub access_count: u32,
}

impl FrecencyRecord {
    /// Create a new frecency record.
    pub fn new(raw: String) -> Self {
        Self {
            raw,
            last_access: current_timestamp(),
            access_count: 1,
        }
    }

    /// Record a new access, updating timestamp and incrementing count.
    pub fn record_access(&mut self) {
        self.last_access = current_timestamp();
        self.access_count = self.access_count.saturating_add(1);
    }

    /// Calculate the frecency score for this record.
    ///
    /// Uses Mozilla-style time-decay buckets combined with access count.
    /// Higher scores indicate more relevant items.
    pub fn score(&self, now: u64) -> u64 {
        let age_hours = now.saturating_sub(self.last_access) / 3600;

        // Time-decay buckets (Mozilla-style)
        let recency_weight = match age_hours {
            0..=4 => 100,    // Last 4 hours
            5..=24 => 70,    // Last day
            25..=168 => 50,  // Last week
            169..=720 => 30, // Last month
            _ => 10,         // Older
        };

        // Cap access count contribution to prevent runaway scores
        let count_factor = u64::from(self.access_count).min(20);

        // Max score: 100 * 20 = 2000
        recency_weight * count_factor
    }
}

/// Serializable frecency data structure.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FrecencyData {
    /// Per-channel frecency records, keyed by channel name then entry raw string.
    pub channels: FxHashMap<String, FxHashMap<String, FrecencyRecord>>,
}

/// Thread-safe frecency manager.
pub struct Frecency {
    /// The frecency data, protected by a read-write lock.
    data: RwLock<FrecencyData>,
    /// Path to the persistence file.
    file_path: PathBuf,
    /// Maximum number of entries to keep per channel.
    max_entries_per_channel: usize,
}

/// A handle to the frecency manager, shareable across threads.
pub type FrecencyHandle = Arc<Frecency>;

/// A snapshot of frecency scores for a channel.
#[derive(Debug, Clone, Default)]
pub struct FrecencyScores {
    scores: Arc<FxHashMap<String, u64>>,
}

impl FrecencyScores {
    #[inline]
    pub fn get(&self, key: &str) -> Option<u64> {
        self.scores.get(key).copied()
    }

    /// Get the underlying Arc for efficient access in sort comparisons.
    /// This avoids repeated cloning when accessing scores multiple times.
    #[inline]
    pub fn scores_arc(&self) -> &Arc<FxHashMap<String, u64>> {
        &self.scores
    }
}

/// Thread-safe cache for frecency scores, refreshed periodically.
pub struct FrecencyCache {
    scores: RwLock<FrecencyScores>,
    channel_name: String,
}

impl FrecencyCache {
    pub fn new(channel_name: String) -> Self {
        Self {
            scores: RwLock::new(FrecencyScores::default()),
            channel_name,
        }
    }

    pub fn refresh(&self, frecency: &Frecency) {
        *self.scores.write() = frecency.get_channel_scores(&self.channel_name);
    }

    #[inline]
    pub fn snapshot(&self) -> FrecencyScores {
        self.scores.read().clone()
    }
}
pub type FrecencyCacheHandle = Arc<FrecencyCache>;

const FRECENCY_FILE_NAME: &str = "frecency.json";

impl Frecency {
    /// Create a new frecency manager.
    ///
    /// # Arguments
    /// * `max_entries_per_channel` - Maximum entries to keep per channel
    /// * `data_dir` - Directory to store the frecency file
    pub fn new(max_entries_per_channel: usize, data_dir: &Path) -> Self {
        Self {
            data: RwLock::new(FrecencyData::default()),
            file_path: data_dir.join(FRECENCY_FILE_NAME),
            max_entries_per_channel,
        }
    }

    /// Initialize the frecency manager by loading data from disk.
    pub fn init(&self) -> Result<()> {
        self.load_from_file()
    }

    /// Load frecency data from file.
    fn load_from_file(&self) -> Result<()> {
        if !self.file_path.exists() {
            debug!(
                "Frecency file does not exist at {:?}, starting fresh",
                self.file_path
            );
            return Ok(());
        }

        let contents = std::fs::read_to_string(&self.file_path)
            .context("Failed to read frecency file")?;

        let data: FrecencyData = serde_json::from_str(&contents)
            .context("Failed to parse frecency file")?;

        debug!("Loaded frecency data with {} channels", data.channels.len());

        *self.data.write() = data;
        Ok(())
    }

    /// Save frecency data to file.
    pub fn save_to_file(&self) -> Result<()> {
        let data = self.data.read();
        let contents = serde_json::to_string_pretty(&*data)
            .context("Failed to serialize frecency data")?;

        // Ensure parent directory exists
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create frecency directory")?;
        }

        std::fs::write(&self.file_path, contents)
            .context("Failed to write frecency file")?;

        debug!("Saved frecency data to {:?}", self.file_path);
        Ok(())
    }

    /// Record an access for an entry in a channel.
    ///
    /// # Arguments
    /// * `channel_name` - The name of the channel
    /// * `raw` - The raw entry string (canonical key)
    pub fn record_access(&self, channel_name: &str, raw: &str) {
        let mut data = self.data.write();
        let channel_entries =
            data.channels.entry(channel_name.to_string()).or_default();

        if let Some(record) = channel_entries.get_mut(raw) {
            record.record_access();
            debug!(
                "Updated frecency for '{}' in channel '{}': count={}",
                raw, channel_name, record.access_count
            );
        } else {
            channel_entries
                .insert(raw.to_string(), FrecencyRecord::new(raw.to_string()));
            debug!(
                "Created frecency record for '{}' in channel '{}'",
                raw, channel_name
            );

            // Prune if over limit
            if channel_entries.len() > self.max_entries_per_channel {
                self.prune_channel_entries(channel_entries);
            }
        }
    }

    /// Get the frecency score for an entry, if it exists.
    ///
    /// # Arguments
    /// * `channel_name` - The name of the channel
    /// * `raw` - The raw entry string (canonical key)
    /// * `now` - Current Unix timestamp
    ///
    /// # Returns
    /// The frecency score, or None if the entry has no frecency record.
    pub fn get_score(
        &self,
        channel_name: &str,
        raw: &str,
        now: u64,
    ) -> Option<u64> {
        let data = self.data.read();
        data.channels
            .get(channel_name)
            .and_then(|entries| entries.get(raw))
            .map(|record| record.score(now))
    }

    pub fn get_channel_scores(&self, channel_name: &str) -> FrecencyScores {
        let now = current_timestamp();
        let data = self.data.read();
        let scores = data
            .channels
            .get(channel_name)
            .map(|entries| {
                entries
                    .iter()
                    .map(|(key, record)| (key.clone(), record.score(now)))
                    .collect()
            })
            .unwrap_or_default();
        FrecencyScores {
            scores: Arc::new(scores),
        }
    }

    pub fn create_cache(&self, channel_name: String) -> FrecencyCacheHandle {
        let cache = Arc::new(FrecencyCache::new(channel_name));
        cache.refresh(self);
        cache
    }

    /// Prune the oldest entries from a channel to stay within limits.
    fn prune_channel_entries(
        &self,
        entries: &mut FxHashMap<String, FrecencyRecord>,
    ) {
        // Calculate how many to remove
        let excess =
            entries.len().saturating_sub(self.max_entries_per_channel);
        if excess == 0 {
            return;
        }

        // Find the entries with the lowest scores
        let now = current_timestamp();
        let mut scores: Vec<_> = entries
            .iter()
            .map(|(key, record)| (key.clone(), record.score(now)))
            .collect();

        // Sort by score ascending (lowest first)
        scores.sort_by_key(|(_, score)| *score);

        // Remove the lowest-scored entries
        for (key, _) in scores.into_iter().take(excess) {
            entries.remove(&key);
        }

        debug!(
            "Pruned {} frecency entries, {} remaining",
            excess,
            entries.len()
        );
    }
}

/// Get the current Unix timestamp in seconds.
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_frecency_record_new() {
        let record = FrecencyRecord::new("test".to_string());
        assert_eq!(record.raw, "test");
        assert_eq!(record.access_count, 1);
        assert!(record.last_access > 0);
    }

    #[test]
    fn test_frecency_record_access() {
        let mut record = FrecencyRecord::new("test".to_string());
        let initial_access = record.last_access;

        // Simulate a small delay
        std::thread::sleep(std::time::Duration::from_millis(10));
        record.record_access();

        assert_eq!(record.access_count, 2);
        assert!(record.last_access >= initial_access);
    }

    #[test]
    fn test_frecency_score_recent() {
        let now = current_timestamp();
        let record = FrecencyRecord {
            raw: "test".to_string(),
            last_access: now,
            access_count: 5,
        };

        // Recent (within 4 hours): 100 * 5 = 500
        let score = record.score(now);
        assert_eq!(score, 500);
    }

    #[test]
    fn test_frecency_score_old() {
        let now = current_timestamp();
        let one_week_ago = now - (7 * 24 * 3600);
        let record = FrecencyRecord {
            raw: "test".to_string(),
            last_access: one_week_ago,
            access_count: 10,
        };

        // Within last week: 50 * 10 = 500
        let score = record.score(now);
        assert_eq!(score, 500);
    }

    #[test]
    fn test_frecency_score_very_old() {
        let now = current_timestamp();
        let one_year_ago = now - (365 * 24 * 3600);
        let record = FrecencyRecord {
            raw: "test".to_string(),
            last_access: one_year_ago,
            access_count: 20,
        };

        // Very old: 10 * 20 = 200
        let score = record.score(now);
        assert_eq!(score, 200);
    }

    #[test]
    fn test_frecency_score_capped_count() {
        let now = current_timestamp();
        let record = FrecencyRecord {
            raw: "test".to_string(),
            last_access: now,
            access_count: 100, // Should be capped to 20
        };

        // Recent with capped count: 100 * 20 = 2000
        let score = record.score(now);
        assert_eq!(score, 2000);
    }

    #[test]
    fn test_frecency_manager_record_and_get() {
        let dir = tempdir().unwrap();
        let frecency = Frecency::new(100, dir.path());

        frecency.record_access("files", "/home/test/file.rs");

        let now = current_timestamp();
        let score = frecency.get_score("files", "/home/test/file.rs", now);
        assert!(score.is_some());

        // First access: 100 * 1 = 100
        assert_eq!(score.unwrap(), 100);
    }

    #[test]
    fn test_frecency_manager_multiple_accesses() {
        let dir = tempdir().unwrap();
        let frecency = Frecency::new(100, dir.path());

        frecency.record_access("files", "/home/test/file.rs");
        frecency.record_access("files", "/home/test/file.rs");
        frecency.record_access("files", "/home/test/file.rs");

        let now = current_timestamp();
        let score = frecency.get_score("files", "/home/test/file.rs", now);

        // Three accesses: 100 * 3 = 300
        assert_eq!(score.unwrap(), 300);
    }

    #[test]
    fn test_frecency_manager_per_channel() {
        let dir = tempdir().unwrap();
        let frecency = Frecency::new(100, dir.path());

        frecency.record_access("files", "/home/test/file.rs");
        frecency.record_access("git-repos", "/home/test/repo");

        let now = current_timestamp();

        // Each channel has its own entries
        assert!(
            frecency
                .get_score("files", "/home/test/file.rs", now)
                .is_some()
        );
        assert!(
            frecency
                .get_score("git-repos", "/home/test/repo", now)
                .is_some()
        );

        // Cross-channel lookup should return None
        assert!(
            frecency
                .get_score("files", "/home/test/repo", now)
                .is_none()
        );
        assert!(
            frecency
                .get_score("git-repos", "/home/test/file.rs", now)
                .is_none()
        );
    }

    #[test]
    fn test_frecency_file_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();

        // Create and populate
        {
            let frecency = Frecency::new(100, &path);
            frecency.record_access("files", "/home/test/a.rs");
            frecency.record_access("files", "/home/test/b.rs");
            frecency.record_access("git-repos", "/home/test/repo");
            frecency.save_to_file().unwrap();
        }

        // Load and verify
        {
            let frecency = Frecency::new(100, &path);
            frecency.init().unwrap();

            let now = current_timestamp();
            assert!(
                frecency
                    .get_score("files", "/home/test/a.rs", now)
                    .is_some()
            );
            assert!(
                frecency
                    .get_score("files", "/home/test/b.rs", now)
                    .is_some()
            );
            assert!(
                frecency
                    .get_score("git-repos", "/home/test/repo", now)
                    .is_some()
            );
        }
    }

    #[test]
    fn test_frecency_pruning() {
        let dir = tempdir().unwrap();
        let frecency = Frecency::new(3, dir.path());

        // Add 5 entries, should prune to 3
        frecency.record_access("files", "a");
        frecency.record_access("files", "b");
        frecency.record_access("files", "c");
        frecency.record_access("files", "d");
        frecency.record_access("files", "e");

        let data = frecency.data.read();
        let entries = data.channels.get("files").unwrap();
        assert!(entries.len() <= 3);
    }
}
