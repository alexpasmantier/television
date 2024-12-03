use std::{collections::HashMap, ops::Deref};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct CableChannelPrototype {
    pub name: String,
    pub source_command: String,
    pub preview_command: String,
    #[serde(default = "default_delimiter")]
    pub preview_delimiter: String,
}

const DEFAULT_DELIMITER: &str = " ";

fn default_delimiter() -> String {
    DEFAULT_DELIMITER.to_string()
}

impl ToString for CableChannelPrototype {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, serde::Deserialize, Default)]
pub struct CableChannels(pub HashMap<String, CableChannelPrototype>);

impl Deref for CableChannels {
    type Target = HashMap<String, CableChannelPrototype>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
