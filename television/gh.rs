use anyhow::Result;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tracing::debug;
use ureq::{RequestBuilder, get, http::HeaderValue, typestate::WithoutBody};

#[derive(Debug, Clone, serde::Deserialize)]
struct GhNode {
    name: String,
    #[serde(rename = "type")]
    kind: NodeType,
    download_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
enum NodeType {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "dir")]
    Directory,
}

const GITHUB_API_BASE_URL: &str =
    "https://api.github.com/repos/alexpasmantier/television/contents/";

#[cfg(unix)]
const REMOTE_CABLE_DIR: &str = "cable/unix";
#[cfg(windows)]
const REMOTE_CABLE_DIR: &str = "cable/windows";

fn make_gh_request(url: &str) -> Result<RequestBuilder<WithoutBody>> {
    let mut request = get(url).header("User-Agent", "television-client");

    let headers = request.headers_mut().unwrap();
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        debug!("Using GITHUB_TOKEN environment variable");
        headers.insert("Authorization", HeaderValue::try_from(token)?);
    }

    Ok(request)
}

fn fetch(url: &str) -> Result<String> {
    let response = make_gh_request(url)?
        .call()
        .map_err(|e| anyhow::anyhow!("Request to '{url}' failed with: {e}"))?;
    Ok(response.into_body().read_to_string()?)
}

fn list_remote_cable_dir(git_ref: &str) -> Result<Vec<GhNode>> {
    let url = format!("{GITHUB_API_BASE_URL}{REMOTE_CABLE_DIR}");
    debug!("Listing remote cable directory: {url} (ref {git_ref})");
    let response = make_gh_request(&url)?
        .header("Accept", "application/vnd.github+json")
        .query("ref", git_ref)
        .call()
        .map_err(|e| anyhow::anyhow!("Request to '{url}' failed with: {e}"))?;
    serde_json::from_str(&response.into_body().read_to_string()?)
        .map_err(|e| anyhow::anyhow!("Failed to parse JSON: {e}"))
}

/// Downloads the channel files of the repository's cable directory at the
/// given git ref, as `(file name, content)` pairs.
pub fn fetch_channel_files(git_ref: &str) -> Result<Vec<(String, String)>> {
    list_remote_cable_dir(git_ref)?
        .into_par_iter()
        .filter(|node| node.kind == NodeType::File)
        .filter_map(|node| Some((node.name, node.download_url?)))
        .map(|(name, url)| {
            debug!("Downloading channel file {name} from {url}");
            Ok((name, fetch(&url)?))
        })
        .collect()
}
