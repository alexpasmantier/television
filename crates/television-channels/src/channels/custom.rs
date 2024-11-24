use crate::channels::OnAir;
use crate::entry::{Entry, PreviewType};
use television_fuzzy::{
    matcher::{config::Config, injector::Injector},
    Matcher,
};

pub struct Channel {
    matcher: Matcher<String>,
    entries_command: String,
    preview_command: String,
    name: String,
}

impl Default for Channel {
    fn default() -> Self {
        Self::new("find . -type f", "bat -n --color=always {}", "Files")
    }
}

impl Channel {
    pub fn new(
        entries_command: &str,
        preview_command: &str,
        name: &str,
    ) -> Self {
        let matcher = Matcher::new(Config::default().n_threads(2));
        let injector = matcher.injector();
        tokio::spawn(load_candidates(entries_command.to_string(), injector));
        Self {
            matcher,
            entries_command: entries_command.to_string(),
            preview_command: preview_command.to_string(),
            name: name.to_string(),
        }
    }
}

#[allow(clippy::unused_async)]
async fn load_candidates(command: String, injector: Injector<String>) {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process");

    let branches = String::from_utf8(output.stdout).unwrap();

    for line in branches.lines() {
        let () = injector.push(line.to_string(), |e, cols| {
            cols[0] = e.clone().into();
        });
    }
}

const PREVIEW_COMMAND: &str = "bat -n --color=always {}";

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
                let path = item.matched_string;
                Entry::new(
                    path.clone(),
                    PreviewType::Command(PREVIEW_COMMAND.to_string()),
                )
                .with_name_match_ranges(item.match_indices)
            })
            .collect()
    }

    fn get_result(&self, index: u32) -> Option<Entry> {
        self.matcher.get_result(index).map(|item| {
            let path = item.matched_string;
            Entry::new(
                path.clone(),
                PreviewType::Command(PREVIEW_COMMAND.to_string()),
            )
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
