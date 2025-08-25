use crate::channels::{entry::Entry, prototypes::Template};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use shlex::try_quote;
use std::{borrow::Cow, cmp::Ordering};
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

/// Extract placeholder count from a template, with caching for efficiency
fn get_template_placeholder_count(template: &Template) -> usize {
    let count = template.template_section_count();
    debug!("Template has {} sections", count);
    count
}

/// Generate warning message for argument mapping mismatches
fn generate_argument_mapping_warning(
    entries_count: usize,
    placeholder_count: usize,
    mode: &SelectorMode,
) -> Option<String> {
    match mode {
        SelectorMode::Single => {
            // No warnings for single mode - only first entry is used
            None
        }
        SelectorMode::Concatenate => {
            // No warnings for concatenate mode - all entries go into one section
            None
        }
        SelectorMode::OneToOne => {
            match entries_count.cmp(&placeholder_count) {
                Ordering::Greater => Some(format!(
                    "WARNING: Excess entries ignored (using {} of {} selected)",
                    placeholder_count, entries_count
                )),
                Ordering::Less => Some(format!(
                    "WARNING: Empty placeholders detected ({} entries for {} placeholders)",
                    entries_count, placeholder_count
                )),
                Ordering::Equal => None, // Perfect match, no warning needed
            }
        }
    }
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
/// * `config` - Selector configuration for this context
///
/// # Returns
/// * `Ok((formatted_string, optional_warning))` - The formatted result and any warnings
/// * `Err(anyhow::Error)` - If processing fails
pub fn process_entries(
    entries: &[&Entry],
    template: &Template,
) -> Result<(String, Option<String>)> {
    if entries.is_empty() {
        return Err(anyhow::anyhow!("Cannot process empty entries"));
    }

    let _is_single_entry = entries.len() == 1;

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

    // Generate warning for template mismatches (for debugging)
    let placeholder_count = get_template_placeholder_count(template);
    debug!(
        "Template analysis: {} placeholders detected, {} entries provided, mode: {:?}",
        placeholder_count,
        entries_processed.len(),
        template.mode
    );

    let warning = generate_argument_mapping_warning(
        entries_processed.len(),
        placeholder_count,
        &template.mode,
    );

    if let Some(ref warning_msg) = warning {
        debug!("Template mismatch detected: {}", warning_msg);
        debug!("Mode: {:?} will handle mismatch gracefully", template.mode);
    }

    // Use centralized template processing - Template handles all modes internally
    let entries_refs: Vec<&str> =
        entries_processed.iter().map(AsRef::as_ref).collect();
    let formatted = template
        .format_with_inputs(&entries_refs, &template.separator)
        .context("Failed to format template with entries")?;

    Ok((formatted, warning))
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
        let (result, warning) =
            process_entries(&entry_refs, &template).unwrap();
        assert_eq!(result, "cat test.txt");
        assert!(warning.is_none());
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
        let (result, warning) =
            process_entries(&entry_refs, &template).unwrap();
        // Result should contain both files
        assert!(result.contains("file1.txt"));
        assert!(result.contains("'file 2.txt'")); // quoted due to space
        assert!(warning.is_none());
    }

    #[test]
    fn test_shell_escaping_disabled() {
        let entries = [Entry::new("file with spaces.txt".to_string())];

        let mut template = Template::parse("cat {}").unwrap();
        template.shell_escaping = false;

        let entry_refs: Vec<&Entry> = entries.iter().collect();
        let (result, warning) =
            process_entries(&entry_refs, &template).unwrap();
        assert!(result.contains("file with spaces.txt"));
        assert!(!result.contains("'file with spaces.txt'"));
        assert!(warning.is_none());
    }
}
