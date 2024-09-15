use std::env::vars;

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::display::{StyleGroup, StyledString};
use crate::finders::{Entry, Finder};

struct EnvVar {
    name: String,
    value: String,
}

#[derive(Clone)]
pub struct EnvVarEntry {
    pub name: String,
    pub value: String,
    pub score: i64,
    pub indices: Vec<usize>,
}

impl Entry for EnvVarEntry {
    fn display(&self) -> Vec<StyledString> {
        vec![
            StyledString {
                text: self.name.clone(),
                style: StyleGroup::Entry,
            },
            StyledString {
                text: ": ".to_string(),
                style: StyleGroup::Default,
            },
            StyledString {
                text: self.value.clone(),
                style: StyleGroup::EntryContent,
            },
        ]
    }
}

pub struct EnvVarFinder {
    env_vars: Vec<EnvVar>,
    matcher: SkimMatcherV2,
}

impl EnvVarFinder {
    pub fn new() -> Self {
        let mut env_vars = Vec::new();
        for (name, value) in vars() {
            env_vars.push(EnvVar { name, value });
        }
        EnvVarFinder {
            env_vars,
            matcher: SkimMatcherV2::default(),
        }
    }
}

impl Finder<EnvVarEntry> for EnvVarFinder {
    type I = std::vec::IntoIter<EnvVarEntry>;

    fn find(&self, pattern: &str) -> Self::I {
        let mut results = Vec::new();
        for env_var in &self.env_vars {
            if pattern.is_empty() {
                results.push(EnvVarEntry {
                    name: env_var.name.clone(),
                    value: env_var.value.clone(),
                    score: 0,
                    indices: Vec::new(),
                });
            } else if let Some((score, indices)) = self.matcher.fuzzy(&env_var.name, pattern, true)
            {
                results.push(EnvVarEntry {
                    name: env_var.name.clone(),
                    value: env_var.value.clone(),
                    score,
                    indices,
                });
            }
        }
        results.into_iter()
    }
}
