use std::fs::read;
use std::io::Result;
use std::path::Path;

use crate::fs::read_lines;

#[derive(Debug, Serialize, Clone)]
pub struct SearchResult {
    pub line_number: u64,
    pub line: String,
    pub line_start: u64,
    pub line_end: u64,
    pub matches: Vec<MatchRange>,
}

pub struct Searcher {}

impl Searcher {
    pub fn new() -> Self {
        Searcher {}
    }

    pub fn search(
        &self,
        file_path: &Path,
        pattern: &str,
        scoring_function: fn(&str, &str) -> f64,
        threshold: f64,
    ) -> Result<Vec<String>> {
        let lines = read_lines(file_path)?;
        let mut results = Vec::new();
        for line in lines {
            if let Ok(line) = line {
                if scoring_function(&line, pattern) > threshold {
                    results.push(line);
                }
            }
        }
        Ok(results)
    }
}
