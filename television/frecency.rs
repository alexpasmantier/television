use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::debug;

const FRECENCY_FILE_NAME: &str = "frecency.json";
const SECONDS_PER_DAY: f64 = 86400.0; // 24 * 60 * 60
/// Maximum number of frecent items to prioritize in search results
/// This ensures frequently-used items get higher priority in nucleo matching
pub const FRECENT_ITEMS_PRIORITY_COUNT: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrecencyEntry {
    /// The actual selected entry (file path, command, etc.)
    pub entry: String,
    /// The channel that the entry belongs to
    pub channel: String,
    /// Number of times this entry was selected
    pub access_count: u32,
    /// Timestamp of the last access
    pub last_access: u64,
    /// Frecency score
    #[serde(skip)]
    score: Option<f64>,
}

impl PartialEq for FrecencyEntry {
    fn eq(&self, other: &Self) -> bool {
        self.entry == other.entry && self.channel == other.channel
    }
}

impl FrecencyEntry {
    pub fn new(entry: String, channel: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut new_entry = Self {
            entry,
            channel,
            access_count: 1,
            last_access: timestamp,
            score: None,
        };

        // Pre-calculate score when creating new entry
        new_entry.score = Some(new_entry.calculate_score(timestamp));
        new_entry
    }

    /// Calculate frecency score based on frequency and recency
    ///
    /// Uses a hybrid algorithm that balances:
    /// - Recency: `1/sqrt(days_since_access + 1)` (gentler decay than linear)
    /// - Frequency: ln(1 + `access_count`) (logarithmic to prevent runaway growth)
    ///
    /// Score = `recency_weight` × `frequency_weight`
    pub fn calculate_score(&self, now: u64) -> f64 {
        // Handle future timestamps or invalid time calculations
        let days_since_access = if self.last_access > now {
            // Future timestamp - treat as very recent (0.1 days)
            0.1
        } else {
            // Normal case: calculate days since access, minimum 0.1 days to prevent division issues
            ((now - self.last_access) as f64 / SECONDS_PER_DAY).max(0.1)
        };

        // Recency weight: square root decay is gentler than linear decay
        // This allows frequently accessed older items to maintain higher scores
        let recency_weight = 1.0 / (days_since_access + 1.0).sqrt();

        // Frequency weight: logarithmic scaling to avoid runaway scores
        let frequency_weight = f64::from(self.access_count).ln_1p();

        // Combined score with better balance between frequency and recency
        recency_weight * frequency_weight
    }

    /// Get the cached frecency score, calculating it if not available
    pub fn get_score(&self) -> f64 {
        // Get current timestamp for score calculation
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.score.unwrap_or_else(|| self.calculate_score(now))
    }
}

#[derive(Debug, Clone)]
pub struct Frecency {
    pub entries: Vec<FrecencyEntry>,
    pub max_size: usize,
    file_path: PathBuf,
    pub current_channel: String,
    pub global_mode: bool,
}

impl Frecency {
    pub fn new(
        max_size: usize,
        channel_name: &str,
        global_mode: bool,
        data_dir: &Path,
    ) -> Self {
        let file_path = data_dir.join(FRECENCY_FILE_NAME);

        Self {
            entries: Vec::with_capacity(max_size),
            max_size,
            file_path,
            current_channel: channel_name.to_string(),
            global_mode,
        }
    }

    /// Initialize the frecency by loading previously persisted entries from disk.
    pub fn init(&mut self) -> Result<()> {
        // if max_size is 0, frecency is disabled
        if self.max_size > 0 {
            self.load_from_file()?;
        }
        Ok(())
    }

    /// Add or update a frecency entry for a selected item.
    pub fn add_entry(&mut self, entry: String, channel: String) -> Result<()> {
        if self.max_size == 0 || entry.trim().is_empty() {
            return Ok(());
        }

        let key = (&entry, &channel);

        if let Some(existing) = self
            .entries
            .iter_mut()
            .find(|e| (&e.entry, &e.channel) == key)
        {
            existing.access_count += 1;
            existing.last_access = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            existing.score =
                Some(existing.calculate_score(existing.last_access));
        } else {
            self.entries.push(FrecencyEntry::new(entry, channel));
        }

        Ok(())
    }

    /// Get the most frecent items for priority matching
    /// Returns up to `FRECENT_ITEMS_PRIORITY_COUNT` items sorted by frecency score (highest first)
    pub fn get_frecent_items(&self) -> Vec<String> {
        // Early exit if frecency is disabled or no entries
        if self.max_size == 0 || self.entries.is_empty() {
            return Vec::new();
        }

        // Cache channel filter calculation
        let channel_filter = if self.global_mode {
            None
        } else {
            Some(self.current_channel.as_str())
        };

        let mut frecent_entries: Vec<_> =
            Vec::with_capacity(FRECENT_ITEMS_PRIORITY_COUNT);
        frecent_entries.extend(
            self.entries
                .iter()
                .filter(|e| channel_filter.is_none_or(|ch| e.channel == ch)),
        );

        // Early exit if no entries match the channel filter
        if frecent_entries.is_empty() {
            return Vec::new();
        }

        // Use partial sort for O(n) performance when we only need top items
        // Performance analysis for typical case (1000 entries → 200 items):
        //   select_nth_unstable_by(200) → O(n) ≈ 1000 comparisons
        //   sort top 200 entries → O(200 × log(200)) ≈ 1500 comparisons
        //   Total ≈ 2500 comparisons vs 10,000+ for full sort
        let limit = FRECENT_ITEMS_PRIORITY_COUNT.min(frecent_entries.len());

        if limit < frecent_entries.len() {
            // Partial sort: O(n) to find top items, then sort only those
            frecent_entries.select_nth_unstable_by(limit, |a, b| {
                let score_a = a.get_score();
                let score_b = b.get_score();
                score_b.partial_cmp(&score_a).unwrap_or(Ordering::Equal)
            });
            // Sort only the selected top items for consistent ordering
            frecent_entries[..limit].sort_by(|a, b| {
                let score_a = a.get_score();
                let score_b = b.get_score();
                score_b.partial_cmp(&score_a).unwrap_or(Ordering::Equal)
            });
        } else {
            // Full sort when we need all items anyway
            frecent_entries.sort_by(|a, b| {
                let score_a = a.get_score();
                let score_b = b.get_score();
                score_b
                    .partial_cmp(&score_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        // Take the most frecent items up to the configured limit
        let mut result = Vec::with_capacity(limit);
        result.extend(
            frecent_entries
                .into_iter()
                .take(limit)
                .map(|e| e.entry.clone()),
        );
        result
    }

    fn load_from_file(&mut self) -> Result<()> {
        if !self.file_path.exists() {
            debug!("Frecency file not found: {}", self.file_path.display());
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.file_path)?;
        if content.trim().is_empty() {
            debug!("Frecency file is empty: {}", self.file_path.display());
            return Ok(());
        }

        let mut loaded_entries: Vec<FrecencyEntry> =
            serde_json::from_str(&content)?;

        // Pre-calculate scores for all loaded entries
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        for entry in &mut loaded_entries {
            entry.score = Some(entry.calculate_score(now));
        }

        // Keep only the most frecent entries if file is too large
        if loaded_entries.len() > self.max_size {
            // Partial sort: O(n) to find top items, then sort only those
            loaded_entries.select_nth_unstable_by(self.max_size, |a, b| {
                let score_a = a.get_score();
                let score_b = b.get_score();
                score_b.partial_cmp(&score_a).unwrap_or(Ordering::Equal)
            });
            // Sort only the selected top items for consistent ordering
            loaded_entries[..self.max_size].sort_by(|a, b| {
                let score_a = a.get_score();
                let score_b = b.get_score();
                score_b.partial_cmp(&score_a).unwrap_or(Ordering::Equal)
            });
            loaded_entries.truncate(self.max_size);
        }

        self.entries = loaded_entries;
        Ok(())
    }

    pub fn save_to_file(&self) -> Result<()> {
        if self.max_size == 0 {
            debug!("Frecency is disabled, not saving to file.");
            return Ok(());
        }

        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json_content = serde_json::to_string_pretty(&self.entries)?;
        std::fs::write(&self.file_path, json_content)?;
        Ok(())
    }

    /// Update the current channel context for this frecency instance.
    pub fn update_channel_context(
        &mut self,
        channel_name: &str,
        global_mode: bool,
    ) {
        self.current_channel = channel_name.to_string();
        self.global_mode = global_mode;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    const SECONDS_PER_DAY: u64 = 86400;

    #[test]
    fn test_frecency_score_calculation() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Test recent item with low frequency
        let recent_item = FrecencyEntry {
            entry: "recent.txt".to_string(),
            channel: "files".to_string(),
            access_count: 1,
            last_access: now - SECONDS_PER_DAY, // 1 day ago
            score: None,
        };

        // Test older item with high frequency
        let frequent_item = FrecencyEntry {
            entry: "frequent.txt".to_string(),
            channel: "files".to_string(),
            access_count: 100,
            last_access: now - (30 * SECONDS_PER_DAY), // 30 days ago
            score: None,
        };

        let recent_score = recent_item.calculate_score(now);
        let frequent_score = frequent_item.calculate_score(now);

        // Frequent item should have higher score despite being older
        assert!(
            frequent_score > recent_score,
            "Frequent item (score: {}) should have higher score than recent item (score: {})",
            frequent_score,
            recent_score
        );

        // Scores should be positive
        assert!(recent_score > 0.0);
        assert!(frequent_score > 0.0);
    }

    #[test]
    fn test_get_frecent_items_sorting() {
        let mut frecency = Frecency::new(
            100,
            "files",
            false,
            Path::new("/tmp/test_frecency.json"),
        );

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Add entries with different access patterns
        let high_freq_old = FrecencyEntry {
            entry: "high_freq_old.txt".to_string(),
            channel: "files".to_string(),
            access_count: 50,
            last_access: now - (7 * SECONDS_PER_DAY), // 7 days ago, high frequency
            score: None,
        };

        let low_freq_recent = FrecencyEntry {
            entry: "low_freq_recent.txt".to_string(),
            channel: "files".to_string(),
            access_count: 2,
            last_access: now - SECONDS_PER_DAY, // 1 day ago, low frequency
            score: None,
        };

        let med_freq_med_time = FrecencyEntry {
            entry: "med_freq_med_time.txt".to_string(),
            channel: "files".to_string(),
            access_count: 10,
            last_access: now - (3 * SECONDS_PER_DAY), // 3 days ago, medium frequency
            score: None,
        };

        frecency.entries =
            vec![low_freq_recent, high_freq_old, med_freq_med_time];

        let frecent_items = frecency.get_frecent_items();

        assert_eq!(frecent_items.len(), 3);
        assert_eq!(frecent_items[0], "high_freq_old.txt");
    }

    #[test]
    fn test_score_caching() {
        let entry =
            FrecencyEntry::new("test.txt".to_string(), "files".to_string());
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Entry should have score pre-calculated during creation
        assert!(entry.score.is_some());

        // get_score should use the cached value
        let score1 = entry.get_score();
        let score2 = entry.get_score();
        assert_eq!(score1, score2);

        // Cached score should match calculated score
        let calculated = entry.calculate_score(now);
        assert_eq!(score1, calculated);
    }

    #[test]
    fn test_frecency_entry_access_patterns() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Very recent access has a high recency weight
        let very_recent = FrecencyEntry {
            entry: "very_recent.txt".to_string(),
            channel: "files".to_string(),
            access_count: 1,
            last_access: now - 3600, // 1 hour ago
            score: None,
        };

        // Very old access has a low recency weight
        let very_old = FrecencyEntry {
            entry: "very_old.txt".to_string(),
            channel: "files".to_string(),
            access_count: 1,
            last_access: now - (365 * SECONDS_PER_DAY), // 1 year ago
            score: None,
        };

        let recent_score = very_recent.calculate_score(now);
        let old_score = very_old.calculate_score(now);

        assert!(
            recent_score > old_score,
            "Recent access should have higher score than very old access"
        );
    }
}
