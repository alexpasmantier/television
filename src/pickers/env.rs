use anyhow::Result;

use crate::finders::{self, Finder};
use crate::previewers::{self, Previewer};

pub struct EnvVarPicker {
    finder: finders::EnvVarFinder,
    entries: Vec<finders::EnvVarEntry>,
    selected_entry: Option<finders::EnvVarEntry>,
    previewer: previewers::EnvVarPreviewer,
}

impl EnvVarPicker {
    pub fn new() -> Self {
        EnvVarPicker {
            finder: finders::EnvVarFinder::new(),
            entries: Vec::new(),
            selected_entry: None,
            previewer: previewers::EnvVarPreviewer::new(),
        }
    }
}

impl super::Picker<finders::EnvVarFinder, finders::EnvVarEntry, previewers::EnvVarPreviewer>
    for EnvVarPicker
{
    fn load_entries(&mut self, pattern: &str) -> Result<()> {
        self.entries = self.finder.find(pattern).collect();
        Ok(())
    }

    fn select_entry(&mut self, index: usize) {
        self.selected_entry = self.entries.get(index).cloned();
    }

    fn selected_entry(&self) -> Option<finders::EnvVarEntry> {
        self.selected_entry.clone()
    }

    fn entries(&self) -> &Vec<finders::EnvVarEntry> {
        &self.entries
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.selected_entry = None;
    }

    fn get_preview(&self) -> Option<previewers::Preview> {
        self.selected_entry
            .as_ref()
            .map(|e| self.previewer.preview(e.clone()))
    }
}
