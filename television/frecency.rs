use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::debug;

const FRECENCY_FILE_NAME: &str = "frecency.json";
const SECONDS_PER_DAY: f64 = 86400.0; // 24 * 60 * 60
pub const DEFAULT_FRECENCY_SIZE: usize = 1000;
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

        Self {
            entry,
            channel,
            access_count: 1,
            last_access: timestamp,
        }
    }

    /// Calculate frecency score based on frequency and recency
    ///
    /// Uses a hybrid algorithm that balances:
    /// - Recency: `1/days_since_access` (hyperbolic decay)
    /// - Frequency: ln(1 + `access_count`) (logarithmic to prevent runaway growth)
    ///
    /// Score = `recency_weight` × `frequency_weight`
    pub fn calculate_score(&self, now: u64) -> f64 {
        // Limit unreasonably high scores when access is very recent (min 0.1 days)
        let days_since_access =
            ((now - self.last_access) as f64 / SECONDS_PER_DAY).max(0.1);

        // Recency weight: more recent = higher score
        let recency_weight = 1.0 / days_since_access;

        // Frequency weight: logarithmic scaling to avoid runaway scores
        let frequency_weight = f64::from(self.access_count).ln_1p();

        // Combined score with recency having stronger influence for recent items
        recency_weight * frequency_weight
    }

    /// Update access information
    pub fn update_access(&mut self) {
        self.access_count += 1;
        self.last_access = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
}

#[derive(Debug, Clone)]
pub struct Frecency {
    entries: Vec<FrecencyEntry>,
    max_size: usize,
    file_path: PathBuf,
    current_channel: String,
    global_mode: bool,
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
        if self.max_size == 0 {
            return Ok(());
        }

        // Don't add empty entries
        if entry.trim().is_empty() {
            return Ok(());
        }

        // Check if entry already exists
        if let Some(existing) = self
            .entries
            .iter_mut()
            .find(|e| e.entry == entry && e.channel == channel)
        {
            existing.update_access();
        } else {
            // Add new entry
            let frecency_entry = FrecencyEntry::new(entry, channel);
            self.entries.push(frecency_entry);

            // Trim if exceeding max size - remove oldest entries
            if self.entries.len() > self.max_size {
                // Sort by last_access and remove the oldest
                self.entries.sort_by_key(|e| e.last_access);
                self.entries.drain(0..self.entries.len() - self.max_size);
            }
        }

        Ok(())
    }

    /// Get frecency score for a specific entry (immutable version for sorting)
    pub fn get_score(&self, entry: &str) -> Option<f64> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Cache channel filter calculation
        let channel_filter = if self.global_mode {
            None
        } else {
            Some(self.current_channel.as_str())
        };

        self.entries
            .iter()
            .find(|e| {
                e.entry == entry
                    && channel_filter.is_none_or(|ch| e.channel == ch)
            })
            .map(|e| e.calculate_score(now))
    }

    /// Check if an entry has frecency data (was previously selected)
    pub fn has_entry(&self, entry: &str) -> bool {
        // Cache channel filter calculation
        let channel_filter = if self.global_mode {
            None
        } else {
            Some(self.current_channel.as_str())
        };

        self.entries.iter().any(|e| {
            e.entry == entry && channel_filter.is_none_or(|ch| e.channel == ch)
        })
    }

    /// Get all frecency entries sorted by score (highest first)
    pub fn get_sorted_entries(&self) -> Vec<(String, f64)> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Cache channel filter calculation
        let channel_filter = if self.global_mode {
            None
        } else {
            Some(self.current_channel.as_str())
        };

        let mut entries = Vec::with_capacity(self.entries.len());
        entries.extend(
            self.entries
                .iter()
                .filter(|e| channel_filter.is_none_or(|ch| e.channel == ch))
                .map(|e| (e.entry.clone(), e.calculate_score(now))),
        );

        entries.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        entries
    }

    /// Get the most frecent items for priority matching
    /// Returns up to `FRECENT_ITEMS_PRIORITY_COUNT` items sorted by recency (newest first)
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
                b.last_access.cmp(&a.last_access)
            });
            // Sort only the selected top items for consistent ordering
            frecent_entries[..limit]
                .sort_by(|a, b| b.last_access.cmp(&a.last_access));
        } else {
            // Full sort when we need all items anyway
            frecent_entries.sort_by(|a, b| b.last_access.cmp(&a.last_access));
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

        // Keep only the most frecent entries if file is too large
        if loaded_entries.len() > self.max_size {
            loaded_entries.sort_by_key(|e| e.last_access);
            loaded_entries.drain(0..loaded_entries.len() - self.max_size);
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

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn max_size(&self) -> usize {
        self.max_size
    }

    pub fn current_channel(&self) -> &str {
        &self.current_channel
    }

    pub fn global_mode(&self) -> bool {
        self.global_mode
    }

    /// Get all entries in the frecency store.
    pub fn get_entries(&self) -> &[FrecencyEntry] {
        &self.entries
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
