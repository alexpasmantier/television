//! Install and remove channels from the television repository on GitHub.
//!
//! Remote channels are taken from the git tag matching this version of tv and
//! cached in the data directory on first use.

use anyhow::{Result, bail};
use colored::Colorize;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    io::{Write, stdout},
    path::{Path, PathBuf},
};
use tracing::debug;

use crate::{
    cable::{CHANNEL_FILE_FORMAT, is_default_cable_file},
    channels::prototypes::Metadata,
    cli::args::CableChannelStatus,
    config::get_data_dir,
    gh::fetch_channel_files,
};

const TV_VERSION: &str = env!("CARGO_PKG_VERSION");
const CACHE_DIR_NAME: &str = "gh-cable-cache";
const CHANNEL_DESCRIPTION_MAX_LEN: usize = 80;

pub fn list(
    cable_dir: &Path,
    color: bool,
    statuses: &[CableChannelStatus],
) -> Result<()> {
    colored::control::set_override(color);
    let mut rows = remote_channels()?
        .into_iter()
        .map(|(name, content)| {
            let status = status(cable_dir, &name, &content)?;
            Ok((name, status, parse_metadata(&content)))
        })
        .collect::<Result<Vec<_>>>()?;
    if !statuses.is_empty() {
        debug!("Filtering channels by status: {statuses:?}");
        rows.retain(|r| statuses.contains(&r.1));
    }

    // formatting
    let name_width = rows.iter().map(|r| r.0.len()).max().unwrap_or(0);
    let status_width = rows
        .iter()
        .map(|r| r.1.to_string().len())
        .max()
        .unwrap_or(0);

    let mut out = stdout().lock();
    for (name, status, metadata) in rows {
        let padded = format!("{status:<status_width$}");
        let mut columns = vec![
            format!("{name:<name_width$}"),
            match status {
                CableChannelStatus::Installed => padded.green(),
                CableChannelStatus::Modified => padded.yellow(),
                CableChannelStatus::BuiltIn => padded.cyan(),
                CableChannelStatus::Available => padded.blue(),
            }
            .to_string(),
        ];
        if let Some(description) = metadata
            .as_ref()
            .and_then(|m| m.description.as_ref()?.lines().next())
        {
            let description =
                if description.len() > CHANNEL_DESCRIPTION_MAX_LEN {
                    format!(
                        "{}...",
                        &description[..CHANNEL_DESCRIPTION_MAX_LEN - 3]
                    )
                } else {
                    description.to_string()
                };
            columns.push(description.dimmed().to_string());
        }
        let missing = metadata
            .as_ref()
            .map(missing_requirements)
            .unwrap_or_default();
        if !missing.is_empty() {
            let needs = format!("(needs {})", missing.join(", "));
            columns.push(needs.red().to_string());
        }
        if writeln!(out, "{}", columns.join("  ")).is_err() {
            break;
        }
    }
    Ok(())
}

/// Prints the remote definition of a channel.
pub fn show(name: &str) -> Result<()> {
    let Some(content) = remote_channels()?.remove(name) else {
        bail!("Unknown channel: {name}");
    };
    // ignore write errors: the reader may exit early (e.g. a preview
    // command piping this into a missing program)
    let _ = stdout().write_all(content.as_bytes());
    Ok(())
}

/// Installs the given channels, overwriting any local version.
pub fn install(cable_dir: &Path, names: &[String]) -> Result<()> {
    let remote = remote_channels()?;
    fs::create_dir_all(cable_dir)?;
    for name in names {
        let Some(content) = remote.get(name) else {
            bail!("Unknown channel: {name}");
        };
        fs::write(local_path(cable_dir, name)?, content)?;
        println!(
            "Installed {name} to {}",
            cable_dir
                .join(format!("{name}.{CHANNEL_FILE_FORMAT}"))
                .display()
        );
    }
    Ok(())
}

// TODO: progress indicator instead of line by line
/// Installs every channel that isn't installed yet and whose requirements are
/// met. With `force`, installs and overwrites everything.
pub fn install_all(cable_dir: &Path, force: bool) -> Result<()> {
    let remote = remote_channels()?;
    fs::create_dir_all(cable_dir)?;
    for (name, content) in &remote {
        let path = local_path(cable_dir, name)?;
        if !force {
            if path.exists() {
                println!("Skipped {name}: already installed");
                continue;
            }
            let missing = parse_metadata(content)
                .as_ref()
                .map(missing_requirements)
                .unwrap_or_default();
            if !missing.is_empty() {
                println!("Skipped {name}: needs {}", missing.join(", "));
                continue;
            }
        }
        fs::write(path, content)?;
        println!("Installed {name}");
    }
    Ok(())
}

/// Removes the given channels from the local cable directory.
pub fn remove(cable_dir: &Path, names: &[String]) -> Result<()> {
    for name in names {
        let path = local_path(cable_dir, name)?;
        if !path.exists() {
            bail!("Channel {name} is not installed");
        }
        fs::remove_file(path)?;
        println!("Removed {name}");
    }
    Ok(())
}

/// Returns the remote channels as `name → content`, downloading them on
/// first use.
fn remote_channels() -> Result<BTreeMap<String, String>> {
    let dir = get_data_dir().join(CACHE_DIR_NAME).join(TV_VERSION);
    if !dir.exists() {
        download_to(&dir)?;
    }
    debug!("Reading remote channels from {}", dir.display());
    let mut channels = BTreeMap::new();
    for entry in fs::read_dir(&dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == CHANNEL_FILE_FORMAT)
            && let Some(name) = path.file_stem().and_then(|s| s.to_str())
        {
            channels.insert(name.to_string(), fs::read_to_string(&path)?);
        }
    }
    Ok(channels)
}

fn download_to(dir: &Path) -> Result<()> {
    eprintln!("Fetching channels for tv {TV_VERSION}...");
    let files = fetch_channel_files(TV_VERSION)?;
    // download to a tempdir first to avoid half-downloaded files if the process is interrupted
    let mut tmp = OsString::from(dir);
    tmp.push(format!(".tmp-{}", std::process::id()));
    let tmp = PathBuf::from(tmp);
    fs::create_dir_all(&tmp)?;
    for (file_name, content) in files {
        fs::write(tmp.join(file_name), content)?;
    }
    debug!("Saving {} as {}", tmp.display(), dir.display());
    fs::rename(&tmp, dir)?;
    Ok(())
}

fn status(
    cable_dir: &Path,
    name: &str,
    remote: &str,
) -> Result<CableChannelStatus> {
    let path = local_path(cable_dir, name)?;
    Ok(match fs::read_to_string(path) {
        Ok(local) if local == remote => CableChannelStatus::Installed,
        Ok(_) => CableChannelStatus::Modified,
        Err(_)
            if is_default_cable_file(&format!(
                "{name}.{CHANNEL_FILE_FORMAT}"
            )) =>
        {
            CableChannelStatus::BuiltIn
        }
        Err(_) => CableChannelStatus::Available,
    })
}

fn local_path(cable_dir: &Path, name: &str) -> Result<PathBuf> {
    // just to be safe
    if Path::new(name).file_name() != Some(name.as_ref()) {
        bail!("Invalid channel name: {name}");
    }
    Ok(cable_dir.join(format!("{name}.{CHANNEL_FILE_FORMAT}")))
}

fn parse_metadata(content: &str) -> Option<Metadata> {
    #[derive(Deserialize)]
    struct MetadataOnly {
        metadata: Metadata,
    }
    toml::from_str::<MetadataOnly>(content)
        .ok()
        .map(|m| m.metadata)
}

fn missing_requirements(metadata: &Metadata) -> Vec<String> {
    metadata
        .requirements
        .clone()
        .par_iter_mut()
        .filter_map(|r| {
            r.init();
            (!r.is_met()).then_some(r.bin_name.clone())
        })
        .collect()
}
