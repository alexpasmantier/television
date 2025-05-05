use crate::preview::{Preview, PreviewContent};
use std::sync::Arc;

#[allow(dead_code)]
pub fn loading(title: &str) -> Arc<Preview> {
    Arc::new(Preview::new(
        title.to_string(),
        PreviewContent::Loading,
        None,
        1,
    ))
}

pub fn timeout(title: &str) -> Arc<Preview> {
    Arc::new(Preview::new(
        title.to_string(),
        PreviewContent::Timeout,
        None,
        1,
    ))
}
