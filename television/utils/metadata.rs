pub struct BuildMetadata {
    pub rustc_version: String,
    pub build_date: String,
    pub target_triple: String,
}

impl BuildMetadata {
    pub fn new(
        rustc_version: String,
        build_date: String,
        target_triple: String,
    ) -> Self {
        Self {
            rustc_version,
            build_date,
            target_triple,
        }
    }
}

pub struct AppMetadata {
    pub version: String,
    pub build: BuildMetadata,
    pub current_directory: String,
}

impl AppMetadata {
    pub fn new(
        version: String,
        build: BuildMetadata,
        current_directory: String,
    ) -> Self {
        Self {
            version,
            build,
            current_directory,
        }
    }
}
