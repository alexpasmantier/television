use crate::config::get_data_dir;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::error;

const HISTORY_FILE_NAME: &str = "history.json";
pub const DEFAULT_HISTORY_SIZE: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// The search query/pattern that was typed
    pub entry: String,
    /// The channel that the entry belongs to
    pub channel: String,
    /// The timestamp of the entry
    pub timestamp: u64,
}

impl PartialEq for HistoryEntry {
    fn eq(&self, other: &Self) -> bool {
        self.entry == other.entry && self.channel == other.channel
    }
}

impl HistoryEntry {
    pub fn new(entry: String, channel: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            entry,
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
        max_size: Option<usize>,
        channel_name: &str,
        global_mode: bool,
        data_dir: &Path,
    ) -> Self {
        let max_size = max_size.unwrap_or(DEFAULT_HISTORY_SIZE);
        let file_path = data_dir.join(HISTORY_FILE_NAME);

        Self {
            entries: Vec::new(),
            current_index: None,
            max_size,
            file_path,
            current_channel: channel_name.to_string(),
            global_mode,
        }
    }

    /// Initialize the history by loading previously persisted entries from disk.
    pub fn init(&mut self) -> Result<()> {
        self.load_from_file()
    }

    /// Add a new history entry, if it's not a duplicate.
    pub fn add_entry(&mut self, query: String, channel: String) -> Result<()> {
        // Don't add empty queries
        if query.trim().is_empty() {
            return Ok(());
        }

        // Don't add duplicate consecutive queries
        if let Some(last_entry) = self.entries.last() {
            if last_entry.entry == query && last_entry.channel == channel {
                return Ok(());
            }
        }

        let history_entry = HistoryEntry::new(query, channel);
        self.entries.push(history_entry);

        // Reset current index when adding new entry
        self.current_index = None;

        // Trim history if it exceeds max size
        if self.entries.len() > self.max_size {
            self.entries.drain(0..self.entries.len() - self.max_size);
        }

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
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.file_path)?;
        if content.trim().is_empty() {
            return Ok(());
        }

        let loaded_entries: Vec<HistoryEntry> =
            serde_json::from_str(&content)?;

        // Keep only the most recent entries if file is too large
        let mut entries = loaded_entries;
        if entries.len() > self.max_size {
            entries.drain(0..entries.len() - self.max_size);
        }

        self.entries = entries;
        Ok(())
    }

    pub fn save_to_file(&self) -> Result<()> {
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
        max_size: usize,
    ) {
        self.current_channel = channel_name.to_string();
        self.global_mode = global_mode;

        // Update max_size and trim if necessary
        if max_size != self.max_size {
            self.max_size = max_size;
            if self.entries.len() > self.max_size {
                self.entries.drain(0..self.entries.len() - self.max_size);
            }
        }

        // Reset navigation state when switching channels
        self.current_index = None;
    }
}

impl Default for History {
    fn default() -> Self {
        let mut history = Self::new(None, "", false, &get_data_dir());
        if let Err(e) = history.init() {
            error!("Failed to create default history: {}", e);
        }
        history
    }
}
