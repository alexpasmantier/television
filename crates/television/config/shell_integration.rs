use std::collections::HashMap;

use rustc_hash::FxHashMap;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct ShellIntegrationConfig {
    pub commands: FxHashMap<String, String>,
}

impl From<ShellIntegrationConfig> for config::ValueKind {
    fn from(val: ShellIntegrationConfig) -> Self {
        let mut m = HashMap::new();
        m.insert(
            String::from("commands"),
            config::ValueKind::Table(
                val.commands
                    .into_iter()
                    .map(|(k, v)| (k, config::ValueKind::String(v).into()))
                    .collect(),
            )
            .into(),
        );
        config::ValueKind::Table(m)
    }
}
