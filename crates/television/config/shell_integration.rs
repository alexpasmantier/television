use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct ShellIntegrationConfig {
    pub commands: HashMap<String, String>,
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
