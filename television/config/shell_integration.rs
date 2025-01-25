use rustc_hash::FxHashMap;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct ShellIntegrationConfig {
    pub commands: FxHashMap<String, String>,
}
