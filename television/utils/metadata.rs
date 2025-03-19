#[derive(Debug, Clone, PartialEq, Hash, Eq)]
/// Global application metadata like version and current directory.
pub struct AppMetadata {
    pub version: String,
    pub current_directory: String,
}

impl AppMetadata {
    pub fn new(version: String, current_directory: String) -> Self {
        Self {
            version,
            current_directory,
        }
    }
}
