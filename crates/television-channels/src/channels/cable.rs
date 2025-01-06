use color_eyre::Result;
use indexmap::IndexSet;
use lazy_static::lazy_static;
use regex::Regex;
use tracing::debug;

use crate::cable::{CableChannelPrototype, DEFAULT_DELIMITER};
use crate::channels::OnAir;
use crate::entry::{Entry, PreviewCommand, PreviewType};
use television_fuzzy::{
    matcher::{config::Config, injector::Injector},
    Matcher,
};
use television_utils::command::shell_command;

#[derive(Debug, Clone)]
enum PreviewKind {
    Command(PreviewCommand),
    Builtin(PreviewType),
    None,
}

#[allow(dead_code)]
pub struct Channel {
    name: String,
    matcher: Matcher<String>,
    entries_command: String,
    preview_kind: PreviewKind,
}

impl Default for Channel {
    fn default() -> Self {
        Self::new(
            "Files",
            "find . -type f",
            Some(PreviewCommand::new("bat -n --color=always {}", ":")),
        )
    }
}

impl From<CableChannelPrototype> for Channel {
    fn from(prototype: CableChannelPrototype) -> Self {
        Self::new(
            &prototype.name,
            &prototype.source_command,
            match prototype.preview_command {
                Some(command) => Some(PreviewCommand::new(
                    &command,
                    &prototype
                        .preview_delimiter
                        .unwrap_or(DEFAULT_DELIMITER.to_string()),
                )),
                None => None,
            },
        )
    }
}

lazy_static! {
    static ref BUILTIN_PREVIEW_RE: Regex = Regex::new(r"^:(\w+):$").unwrap();
}

fn parse_preview_kind(command: &PreviewCommand) -> Result<PreviewKind> {
    debug!("Parsing preview kind for command: {:?}", command);
    if let Some(captures) = BUILTIN_PREVIEW_RE.captures(&command.command) {
        let preview_type = PreviewType::try_from(&captures[1])?;
        Ok(PreviewKind::Builtin(preview_type))
    } else {
        Ok(PreviewKind::Command(command.clone()))
    }
}

impl Channel {
    pub fn new(
        name: &str,
        entries_command: &str,
        preview_command: Option<PreviewCommand>,
    ) -> Self {
        let matcher = Matcher::new(Config::default());
        let injector = matcher.injector();
        tokio::spawn(load_candidates(entries_command.to_string(), injector));
        let preview_kind = match preview_command {
            Some(command) => {
                parse_preview_kind(&command).unwrap_or_else(|_| {
                    panic!("Invalid preview command: {command}")
                })
            }
            None => PreviewKind::None,
        };
        debug!("Preview kind: {:?}", preview_kind);
        Self {
            matcher,
            entries_command: entries_command.to_string(),
            preview_kind,
            name: name.to_string(),
        }
    }
}

#[allow(clippy::unused_async)]
async fn load_candidates(command: String, injector: Injector<String>) {
    debug!("Loading candidates from command: {:?}", command);
    let output = shell_command()
        .arg(command)
        .output()
        .expect("failed to execute process");

    let decoded_output = String::from_utf8(output.stdout).unwrap();
    debug!("Decoded output: {:?}", decoded_output);

    for line in decoded_output.lines().collect::<IndexSet<_>>() {
        if !line.trim().is_empty() {
            let () = injector.push(line.to_string(), |e, cols| {
                cols[0] = e.clone().into();
            });
        }
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
                let path = item.matched_string;
                Entry::new(
                    path.clone(),
                    match &self.preview_kind {
                        PreviewKind::Command(ref preview_command) => {
                            PreviewType::Command(preview_command.clone())
                        }
                        PreviewKind::Builtin(preview_type) => {
                            preview_type.clone()
                        }
                        PreviewKind::None => PreviewType::None,
                    },
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
                match &self.preview_kind {
                    PreviewKind::Command(ref preview_command) => {
                        PreviewType::Command(preview_command.clone())
                    }
                    PreviewKind::Builtin(preview_type) => preview_type.clone(),
                    PreviewKind::None => PreviewType::None,
                },
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
