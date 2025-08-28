use crate::channels::{entry::Entry, prototypes::Template};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use shlex::try_quote;
use std::borrow::Cow;
use tracing::debug;

/// Defines how multiple selected arguments are distributed to template processing
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum SelectorMode {
    /// Use only the first selected item
    Single,
    /// Concatenate all selected items with separator and provide to all template placeholders (default)
    #[default]
    Concatenate,
    /// Map each selected item to one template placeholder (1:1 mapping)
    OneToOne,
}

/// Process multiple entries through a template using selector configuration
///
/// This function handles both single and multiple entries:
/// - Single entry: Uses standard template formatting
/// - Multiple entries: Uses selector-specific processing based on mode
///
/// # Arguments
/// * `entries` - Collection of entries to process
/// * `template` - Template to format entries with
///
/// # Returns
/// * `Ok(String)` - The formatted result
/// * `Err(anyhow::Error)` - If processing fails
pub fn process_entries(
    entries: &[&Entry],
    template: &Template,
) -> Result<String> {
    if entries.is_empty() {
        return Err(anyhow::anyhow!("Cannot process empty entries"));
    }

    debug!(
        "Processing {} entries with selector mode: {:?}",
        entries.len(),
        template.mode
    );

    // Process entries with shell escaping if enabled
    let entries_processed: Vec<String> = entries
        .iter()
        .map(|&entry| {
            if template.shell_escaping {
                try_quote(&entry.raw)
                    .map(Cow::into_owned)
                    .unwrap_or_else(|_| entry.raw.clone())
            } else {
                entry.raw.clone()
            }
        })
        .collect();

    // Use centralized template processing - Template handles all modes internally
    let entries_refs: Vec<&str> =
        entries_processed.iter().map(AsRef::as_ref).collect();
    let formatted = template
        .format_with_inputs(&entries_refs, &template.separator)
        .context("Failed to format template with entries")?;

    Ok(formatted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector_mode_serialization() {
        assert_eq!(
            serde_json::to_string(&SelectorMode::Single).unwrap(),
            "\"single\""
        );
        assert_eq!(
            serde_json::to_string(&SelectorMode::Concatenate).unwrap(),
            "\"concatenate\""
        );
        assert_eq!(
            serde_json::to_string(&SelectorMode::OneToOne).unwrap(),
            "\"one_to_one\""
        );
    }

    #[test]
    fn test_process_single_entry() {
        let entries = [Entry::new("test.txt".to_string())];

        let template = Template::parse("cat {}").unwrap();

        let entry_refs: Vec<&Entry> = entries.iter().collect();
        let result = process_entries(&entry_refs, &template).unwrap();
        assert_eq!(result, "cat test.txt");
    }

    #[test]
    fn test_process_multiple_entries_concatenate() {
        let entries = vec![
            Entry::new("file1.txt".to_string()),
            Entry::new("file 2.txt".to_string()),
        ];

        let mut template = Template::parse("diff {}").unwrap();
        template.mode = SelectorMode::Concatenate;
        template.separator = " ".to_string();
        template.shell_escaping = true;

        let entry_refs: Vec<&Entry> = entries.iter().collect();
        let result = process_entries(&entry_refs, &template).unwrap();
        // Result should contain both files
        assert!(result.contains("file1.txt"));
        assert!(result.contains("'file 2.txt'")); // quoted due to space
    }

    #[test]
    fn test_shell_escaping_disabled() {
        let entries = [Entry::new("file with spaces.txt".to_string())];

        let mut template = Template::parse("cat {}").unwrap();
        template.shell_escaping = false;

        let entry_refs: Vec<&Entry> = entries.iter().collect();
        let result = process_entries(&entry_refs, &template).unwrap();
        assert!(result.contains("file with spaces.txt"));
        assert!(!result.contains("'file with spaces.txt'"));
    }

    #[test]
    fn test_concatenate_mode_multi_placeholder() {
        let entries = vec![
            Entry::new("file1.txt".to_string()),
            Entry::new("file2.txt".to_string()),
        ];

        let mut template = Template::parse("cmd {} {}").unwrap();
        template.mode = SelectorMode::Concatenate;
        template.separator = " ".to_string();
        template.shell_escaping = false;

        let entry_refs: Vec<&Entry> = entries.iter().collect();
        let result = process_entries(&entry_refs, &template).unwrap();
        // In concatenate mode, all inputs go to each placeholder
        assert_eq!(result, "cmd file1.txt file2.txt file1.txt file2.txt");
    }

    #[test]
    fn test_single_mode_multi_placeholder() {
        let entries = vec![
            Entry::new("file1.txt".to_string()),
            Entry::new("file2.txt".to_string()),
        ];

        let mut template = Template::parse("diff {} {}").unwrap();
        template.mode = SelectorMode::Single;
        template.separator = " ".to_string();
        template.shell_escaping = false;

        let entry_refs: Vec<&Entry> = entries.iter().collect();
        let result = process_entries(&entry_refs, &template).unwrap();
        // Should use only first entry for both placeholders
        assert_eq!(result, "diff file1.txt file1.txt");
    }

    #[test]
    fn test_empty_entries() {
        let entries: Vec<Entry> = vec![];
        let template = Template::parse("cat {}").unwrap();

        let entry_refs: Vec<&Entry> = entries.iter().collect();
        let result = process_entries(&entry_refs, &template);

        // Should return error for empty entries
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Cannot process empty entries")
        );
    }
}
