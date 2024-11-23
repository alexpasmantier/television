use bollard::{image::ListImagesOptions, Docker};
use devicons::FileIcon;

use crate::channels::OnAir;
use crate::entry::{Entry, PreviewType};

use std::collections::HashMap;
use std::default::Default;

use television_fuzzy::matcher::{config::Config, injector::Injector, Matcher};

pub struct Channel {
    matcher: Matcher<DockerImage>,
    icon: FileIcon,
}

#[derive(Debug, Clone)]
struct DockerImage {
    repository: String,
    tag: String,
    image_id: String,
}

impl Channel {
    pub fn new() -> Self {
        let matcher = Matcher::new(Config::default().n_threads(1));
        let injector = matcher.injector();
        tokio::spawn(load_docker_images(injector));
        Channel {
            matcher,
            icon: FileIcon::from("docker"),
        }
    }
}

impl Default for Channel {
    fn default() -> Self {
        Self::new()
    }
}

impl OnAir for Channel {
    fn find(&mut self, pattern: &str) {
        self.matcher.find(pattern);
    }

    fn results(&mut self, num_entries: u32, offset: u32) -> Vec<Entry> {
        self.matcher.tick();
        self.matcher
            .results(num_entries, offset)
            .into_iter()
            .map(|item| {
                let entry =
                    Entry::new(item.inner.repository, PreviewType::EnvVar)
                        .with_value(item.inner.tag.clone())
                        .with_icon(self.icon);

                entry
            })
            .collect()
    }

    fn get_result(&self, index: u32) -> Option<Entry> {
        self.matcher.get_result(index).map(|item| {
            Entry::new(item.inner.repository, PreviewType::EnvVar)
                .with_value(item.inner.tag.clone())
                .with_icon(self.icon)
        })
    }
    fn result_count(&self) -> u32 {
        self.matcher.matched_item_count
    }

    fn total_count(&self) -> u32 {
        self.matcher.total_item_count
    }

    fn running(&self) -> bool {
        self.matcher.status.running
    }
    fn shutdown(&self) {}
}

async fn load_docker_images(injector: Injector<DockerImage>) {
    let docker = match Docker::connect_with_local_defaults() {
        Ok(d) => d,
        Err(e) => {
            panic!("{:?}", e)
        }
    };
    let mut filters = HashMap::new();
    filters.insert("dangling", vec!["false"]);
    // List images

    let result = docker
        .list_images(Some(ListImagesOptions {
            all: true,
            filters,
            ..Default::default()
        }))
        .await;
    let summaries = match result {
        Ok(image_summaries) => image_summaries,
        Err(e) => {
            panic!("{:?}", e)
        }
    };
    summaries
        .iter()
        .filter(|f| f.repo_tags.len() > 0 && f.repo_digests.len() > 0)
        .for_each(|f| {
            injector.push(
                DockerImage {
                    repository: f.repo_digests[0].clone(),
                    tag: f.repo_tags[0].clone(),
                    image_id: (f.id).clone(),
                },
                |_, cols| {
                    cols[0] = (f.repo_digests[0]).clone().into();
                },
            )
        })

    // push the images to the injector, so that the matcher sees them
}
