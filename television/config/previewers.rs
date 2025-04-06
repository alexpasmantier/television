use crate::preview::{previewers, PreviewerConfig};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, Hash)]
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
            .file(previewers::files::FilePreviewerConfig::new(val.file.theme))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, Hash)]
pub struct BasicPreviewerConfig {}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Hash)]
#[serde(default)]
pub struct FilePreviewerConfig {
    //pub max_file_size: u64,
    pub theme: String,
}

impl Default for FilePreviewerConfig {
    fn default() -> Self {
        Self {
            //max_file_size: 1024 * 1024,
            theme: String::from("TwoDark"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, Hash)]
pub struct EnvVarPreviewerConfig {}
