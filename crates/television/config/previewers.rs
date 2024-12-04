use config::ValueKind;
use serde::Deserialize;
use std::collections::HashMap;
use television_previewers::previewers;
use television_previewers::previewers::PreviewerConfig;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct PreviewersConfig {
    #[serde(default)]
    pub basic: BasicPreviewerConfig,
    pub file: FilePreviewerConfig,
    #[serde(default)]
    pub env_var: EnvVarPreviewerConfig,
}

impl From<PreviewersConfig> for PreviewerConfig {
    fn from(val: PreviewersConfig) -> Self {
        PreviewerConfig::default()
            .file(previewers::FilePreviewerConfig::new(val.file.theme))
    }
}

impl From<PreviewersConfig> for ValueKind {
    fn from(val: PreviewersConfig) -> Self {
        let mut m = HashMap::new();
        m.insert(String::from("basic"), val.basic.into());
        m.insert(String::from("file"), val.file.into());
        m.insert(String::from("env_var"), val.env_var.into());
        ValueKind::Table(m)
    }
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct BasicPreviewerConfig {}

impl From<BasicPreviewerConfig> for ValueKind {
    fn from(_val: BasicPreviewerConfig) -> Self {
        ValueKind::Table(HashMap::new())
    }
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct FilePreviewerConfig {
    //pub max_file_size: u64,
    pub theme: String,
}

impl From<FilePreviewerConfig> for ValueKind {
    fn from(val: FilePreviewerConfig) -> Self {
        let mut m = HashMap::new();
        m.insert(String::from("theme"), ValueKind::String(val.theme).into());
        ValueKind::Table(m)
    }
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct EnvVarPreviewerConfig {}

impl From<EnvVarPreviewerConfig> for ValueKind {
    fn from(_val: EnvVarPreviewerConfig) -> Self {
        ValueKind::Table(HashMap::new())
    }
}
