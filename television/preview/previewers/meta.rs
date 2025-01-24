use crate::preview::{Preview, PreviewContent};
use std::sync::Arc;

pub fn not_supported(title: &str) -> Arc<Preview> {
    Arc::new(Preview::new(
        title.to_string(),
        PreviewContent::NotSupported,
        None,
        None,
        1,
    ))
}

pub fn file_too_large(title: &str) -> Arc<Preview> {
    Arc::new(Preview::new(
        title.to_string(),
        PreviewContent::FileTooLarge,
        None,
        None,
        1,
    ))
}

#[allow(dead_code)]
pub fn loading(title: &str) -> Arc<Preview> {
    Arc::new(Preview::new(
        title.to_string(),
        PreviewContent::Loading,
        None,
        None,
        1,
    ))
}

pub fn timeout(title: &str) -> Arc<Preview> {
    Arc::new(Preview::new(
        title.to_string(),
        PreviewContent::Timeout,
        None,
        None,
        1,
    ))
}
