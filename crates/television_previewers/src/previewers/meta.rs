use crate::previewers::{Preview, PreviewContent};
use std::sync::Arc;

pub fn not_supported(title: &str) -> Arc<Preview> {
    Arc::new(Preview::new(
        title.to_string(),
        PreviewContent::NotSupported,
    ))
}

pub fn file_too_large(title: &str) -> Arc<Preview> {
    Arc::new(Preview::new(
        title.to_string(),
        PreviewContent::FileTooLarge,
    ))
}

#[allow(dead_code)]
pub fn loading(title: &str) -> Arc<Preview> {
    Arc::new(Preview::new(title.to_string(), PreviewContent::Loading))
}
