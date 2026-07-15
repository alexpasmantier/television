//! Migration of machine-written configuration files.
//!
//! Older tv versions auto-wrote the full default config to the user's config
//! directory on first run, spelling out every default explicitly. Those
//! values look like user intent to the config merger and pin users to the
//! defaults of the version that wrote them.
//!
//! `legacy_config_templates.json` holds the leaf (path, value) pairs of
//! every default config tv ever shipped (extracted from git history by
//! `scripts/generate_legacy_config_pairs.py`). A user config is matched
//! against each template individually: the one it descends from is the one
//! sharing most of its pairs, and only pairs from that template count as
//! boilerplate — a value like `ui_scale = 80` was the default of one era
//! and a deliberate choice in any other.

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::path::{Path, PathBuf};

use super::CONFIG_FILE_NAME;

const LEGACY_TEMPLATES: &str = include_str!("legacy_config_templates.json");
const NOTICE_STAMP_FILE_NAME: &str = ".migrate-config-notice";

/// Minimum fraction of a template's pairs that must appear in the user's
/// config for the file to count as descending from that template.
const TEMPLATE_MATCH_THRESHOLD: f64 = 0.5;

type Leaf = (Vec<String>, toml::Value);

#[derive(Deserialize)]
struct LegacyPair {
    path: Vec<String>,
    value: serde_json::Value,
}

#[derive(Deserialize)]
struct LegacyTemplate {
    pairs: Vec<LegacyPair>,
}

impl LegacyTemplate {
    fn contains(&self, (path, value): &Leaf) -> bool {
        let Ok(json_value) = serde_json::to_value(value) else {
            return false;
        };
        self.pairs
            .iter()
            .any(|pair| &pair.path == path && pair.value == json_value)
    }
}

fn legacy_templates() -> Vec<LegacyTemplate> {
    serde_json::from_str(LEGACY_TEMPLATES)
        .expect("embedded legacy config templates should be valid JSON")
}

/// Flatten a TOML table into leaf (path, value) pairs. Arrays count as
/// leaves and are compared wholesale.
fn flatten_leaves(table: &toml::Table) -> Vec<Leaf> {
    fn walk(
        prefix: &mut Vec<String>,
        value: &toml::Value,
        out: &mut Vec<Leaf>,
    ) {
        if let toml::Value::Table(table) = value {
            for (key, child) in table {
                prefix.push(key.clone());
                walk(prefix, child, out);
                prefix.pop();
            }
        } else {
            out.push((prefix.clone(), value.clone()));
        }
    }
    let mut leaves = Vec::new();
    for (key, child) in table {
        walk(&mut vec![key.clone()], child, &mut leaves);
    }
    leaves
}

/// The shipped template the user's config descends from, if any.
fn best_matching_template(
    templates: &[LegacyTemplate],
    leaves: &[Leaf],
) -> Option<usize> {
    let (best, score) = templates
        .iter()
        .map(|template| {
            leaves.iter().filter(|leaf| template.contains(leaf)).count()
        })
        .enumerate()
        .max_by_key(|(_, score)| *score)?;
    #[allow(clippy::cast_precision_loss)]
    let fraction = score as f64 / templates[best].pairs.len() as f64;
    (fraction >= TEMPLATE_MATCH_THRESHOLD).then_some(best)
}

/// Rebuild a nested TOML table from leaf (path, value) pairs.
fn table_from_leaves(leaves: &[Leaf]) -> toml::Table {
    let mut root = toml::Table::new();
    for (path, value) in leaves {
        let mut node = &mut root;
        for segment in &path[..path.len() - 1] {
            node = node
                .entry(segment.clone())
                .or_insert_with(|| toml::Value::Table(toml::Table::new()))
                .as_table_mut()
                .expect("intermediate config nodes should be tables");
        }
        node.insert(
            path.last().expect("leaf paths are never empty").clone(),
            value.clone(),
        );
    }
    root
}

fn display_leaf((path, value): &Leaf) -> String {
    format!("{} = {}", path.join("."), value)
}

pub struct MigrationReport {
    pub backup: PathBuf,
    /// Settings that survived the migration, as `path = value` lines.
    pub kept: Vec<String>,
}

/// Back up the user's config file and rewrite it with only the settings
/// that aren't machine-written boilerplate.
pub fn migrate_config(config_dir: &Path) -> Result<MigrationReport> {
    let config_file = config_dir.join(CONFIG_FILE_NAME);
    if !config_file.is_file() {
        bail!(
            "no config file found at {}, nothing to migrate",
            config_file.display()
        );
    }
    let backup = config_file.with_extension("toml.bak");
    if backup.exists() {
        bail!(
            "backup file {} already exists, move it out of the way first",
            backup.display()
        );
    }

    let contents = std::fs::read_to_string(&config_file)?;
    let table: toml::Table = toml::from_str(&contents).context(format!(
        "Error parsing configuration file: {}",
        config_file.display()
    ))?;

    let templates = legacy_templates();
    let leaves = flatten_leaves(&table);
    let Some(template) = best_matching_template(&templates, &leaves) else {
        bail!(
            "{} doesn't look like it was machine-written by an older tv \
             version, nothing to migrate",
            config_file.display()
        );
    };
    let template = &templates[template];
    let kept: Vec<Leaf> = leaves
        .into_iter()
        .filter(|leaf| !template.contains(leaf))
        .collect();

    // serialize before touching anything on disk
    let new_contents = if kept.is_empty() {
        None
    } else {
        let serialized = toml::to_string_pretty(&table_from_leaves(&kept))?;
        Some(format!(
            "# generated by `tv migrate-config`, original backed up as \
             config.toml.bak\n\n{serialized}"
        ))
    };

    std::fs::rename(&config_file, &backup)?;
    if let Some(contents) = &new_contents {
        std::fs::write(&config_file, contents)?;
    }

    Ok(MigrationReport {
        backup,
        kept: kept.iter().map(display_leaf).collect(),
    })
}

/// One-time notice inviting users whose config still contains machine-written
/// boilerplate to run `tv migrate-config`. The data directory is stamped
/// after the first completed check — whatever its outcome — so the template
/// matching doesn't tax every exit.
pub fn maybe_print_migration_notice(config_dir: &Path, data_dir: &Path) {
    let stamp = data_dir.join(NOTICE_STAMP_FILE_NAME);
    if stamp.exists() {
        return;
    }
    let Ok(contents) =
        std::fs::read_to_string(config_dir.join(CONFIG_FILE_NAME))
    else {
        return;
    };
    let Ok(table) = toml::from_str::<toml::Table>(&contents) else {
        return;
    };
    let leaves = flatten_leaves(&table);
    if best_matching_template(&legacy_templates(), &leaves).is_some() {
        eprintln!(
            "\ntip: your config file was generated by an older tv version \
             and pins its old defaults.\nRun `tv migrate-config` to \
             automatically migrate it."
        );
    }
    let _ = std::fs::write(&stamp, b"");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // real machine-written config from the 0.13-0.15 era (comments stripped)
    const OLD_GENERATED_CONFIG: &str =
        include_str!("../../tests/fixtures/machine_written_config_0_15.toml");

    fn write_config(dir: &Path, contents: &str) -> PathBuf {
        let config_file = dir.join(CONFIG_FILE_NAME);
        std::fs::write(&config_file, contents).unwrap();
        config_file
    }

    #[test]
    fn migrate_untouched_config_only_backs_up() {
        let dir = tempdir().unwrap();
        let config_file = write_config(dir.path(), OLD_GENERATED_CONFIG);

        let report = migrate_config(dir.path()).unwrap();

        assert!(report.kept.is_empty());
        assert!(!config_file.exists());
        assert!(report.backup.is_file());
    }

    #[test]
    fn migrate_keeps_user_settings() {
        let dir = tempdir().unwrap();
        let contents = OLD_GENERATED_CONFIG
            .replace(r#"theme = "default""#, r#"theme = "tokyonight""#)
            .replace(
                r#""command_history" = "ctrl-r""#,
                r#""command_history" = "ctrl-h""#,
            );
        write_config(dir.path(), &contents);

        let report = migrate_config(dir.path()).unwrap();

        assert_eq!(
            report.kept,
            vec![
                "shell_integration.keybindings.command_history = \"ctrl-h\""
                    .to_string(),
                "ui.theme = \"tokyonight\"".to_string()
            ]
        );

        let new_config: toml::Table = toml::from_str(
            &std::fs::read_to_string(dir.path().join(CONFIG_FILE_NAME))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            new_config["ui"]["theme"],
            toml::Value::String("tokyonight".into())
        );
        assert!(new_config["ui"].get("input_bar").is_none());
    }

    #[test]
    fn migrate_leaves_handwritten_configs_alone() {
        let dir = tempdir().unwrap();
        // coincides with some era's defaults, but was written by hand
        let config_file = write_config(
            dir.path(),
            "[ui]\nui_scale = 80\ntheme = \"gruvbox-dark\"\n",
        );

        assert!(migrate_config(dir.path()).is_err());
        assert!(config_file.is_file());
    }

    #[test]
    fn notice_check_runs_once() {
        let config_dir = tempdir().unwrap();
        let data_dir = tempdir().unwrap();
        let stamp = data_dir.path().join(NOTICE_STAMP_FILE_NAME);

        // no config file: nothing to check, no stamp
        maybe_print_migration_notice(config_dir.path(), data_dir.path());
        assert!(!stamp.exists());

        // any completed check stamps, even when no notice is warranted
        write_config(config_dir.path(), "[ui]\ntheme = \"gruvbox-dark\"\n");
        maybe_print_migration_notice(config_dir.path(), data_dir.path());
        assert!(stamp.exists());
    }

    #[test]
    fn migrate_refuses_to_overwrite_backup() {
        let dir = tempdir().unwrap();
        let config_file = write_config(dir.path(), OLD_GENERATED_CONFIG);
        std::fs::write(config_file.with_extension("toml.bak"), "").unwrap();

        assert!(migrate_config(dir.path()).is_err());
        assert!(config_file.is_file());
    }
}
