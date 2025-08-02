use anyhow::Result;
use colored::Colorize;
use std::path::Path;
use tracing::debug;
use ureq::get;

use crate::{
    cable::{CABLE_DIR_NAME, CHANNEL_FILE_FORMAT},
    channels::prototypes::ChannelPrototype,
    config::get_config_dir,
};

#[derive(Debug, Clone, serde::Deserialize)]
struct GhNode {
    name: String,
    #[serde(rename = "type")]
    kind: NodeType,
    download_url: Option<String>,
}

impl GhNode {
    pub fn is_file(&self) -> bool {
        matches!(self.kind, NodeType::File)
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
enum NodeType {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "dir")]
    Directory,
}

const GITHUB_API_BASE_URL: &str =
    "https://api.github.com/repos/alexpasmantier/television/contents/";

fn make_gh_content_request(gh_dir: &Path) -> Result<Vec<GhNode>> {
    let url = format!("{}{}", GITHUB_API_BASE_URL, gh_dir.to_str().unwrap());
    debug!("Making GitHub API request to: {}", url);
    get(&url)
        .header("User-Agent", "television-client")
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| {
            anyhow::anyhow!("Request to '{}' failed with: {}", url, e)
        })
        .map(|response| {
            if response.status().is_success() {
                serde_json::from_str::<Vec<GhNode>>(
                    &response.into_body().read_to_string()?,
                )
                .map_err(|e| anyhow::anyhow!("Failed to parse JSON: {}", e))
            } else {
                Err(anyhow::anyhow!("Failed to fetch data from GitHub API"))
            }
        })?
}

fn fetch_raw_content_from_url(url: &str) -> Result<String> {
    let response =
        get(url).header("User-Agent", "television-client").call()?;

    if response.status().is_success() {
        Ok(response.into_body().read_to_string()?)
    } else {
        Err(anyhow::anyhow!(
            "Failed to fetch raw content from URL: {}",
            url
        ))
    }
}

struct DownloadedPrototype {
    pub name: String,
    pub content: String,
}

impl DownloadedPrototype {
    pub fn new(name: String, content: String) -> Self {
        Self { name, content }
    }
}

#[cfg(unix)]
const DEFAULT_CABLE_DIR_PATH: &str = "cable/unix";
#[cfg(windows)]
const DEFAULT_CABLE_DIR_PATH: &str = "cable/windows";

fn get_default_prototypes_from_repo() -> Result<Vec<DownloadedPrototype>> {
    let nodes = make_gh_content_request(Path::new(DEFAULT_CABLE_DIR_PATH))?;
    for node in &nodes {
        println!(
            "  Discovered channel: {}\t\tdownload url: {}",
            node.name.blue().bold(),
            node.download_url.as_deref().unwrap_or("N/A").blue().bold()
        );
    }
    Ok(nodes
        .iter()
        .filter_map(|node| {
            if node.is_file() {
                node.download_url.clone()
            } else {
                None
            }
        })
        .filter_map(|url| fetch_raw_content_from_url(&url).ok())
        .filter_map(|content| {
            let name = toml::from_str::<ChannelPrototype>(&content)
                .map(|p| p.metadata.name)
                .ok()?;
            Some(DownloadedPrototype::new(name, content))
        })
        .collect())
}

pub fn update_local_channels(force: &bool) -> Result<()> {
    println!("{}", "Fetching latest cable channels...".bold());
    let default_prototypes = get_default_prototypes_from_repo()?;
    println!("{}", "\nSaving channels locally...".bold());
    let cable_path = get_config_dir().join(CABLE_DIR_NAME);
    if !cable_path.exists() {
        println!("  Creating cable directory at {}", cable_path.display());
        std::fs::create_dir_all(&cable_path)?;
    }
    for p in default_prototypes {
        let file_path =
            cable_path.join(&p.name).with_extension(CHANNEL_FILE_FORMAT);
        // if the file already exists, don't overwrite it
        if file_path.exists() && !force {
            println!(
                "  Channel {} already exists at {}, SKIPPING...",
                p.name.blue().bold(),
                file_path.display().to_string().yellow().bold()
            );
            continue;
        }
        std::fs::write(&file_path, p.content)?;
        println!(
            "  Saved channel {} to {}",
            p.name.blue().bold(),
            file_path.display().to_string().yellow().bold()
        );
    }
    println!(
        "{}",
        "\nCable channels updated successfully.".green().bold()
    );
    Ok(())
}
