use devicons::FileIcon;

use crate::previewers::PreviewType;

/// NOTE: having an enum for entry types would be nice since it would allow
/// having a nicer implementation for transitions between channels. This would
/// permit implementing `From<EntryType>` for channels which would make the
/// channel convertible from any other that yields `EntryType`.
/// This needs pondering since it does bring another level of abstraction and
/// adds a layer of complexity.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Entry {
    pub name: String,
    display_name: Option<String>,
    pub value: Option<String>,
    pub name_match_ranges: Option<Vec<(u32, u32)>>,
    pub value_match_ranges: Option<Vec<(u32, u32)>>,
    pub icon: Option<FileIcon>,
    pub line_number: Option<usize>,
    pub preview_type: PreviewType,
}

impl Entry {
    pub fn new(name: String, preview_type: PreviewType) -> Self {
        Self {
            name,
            display_name: None,
            value: None,
            name_match_ranges: None,
            value_match_ranges: None,
            icon: None,
            line_number: None,
            preview_type,
        }
    }

    pub fn with_display_name(mut self, display_name: String) -> Self {
        self.display_name = Some(display_name);
        self
    }

    pub fn with_value(mut self, value: String) -> Self {
        self.value = Some(value);
        self
    }

    pub fn with_name_match_ranges(
        mut self,
        name_match_ranges: Vec<(u32, u32)>,
    ) -> Self {
        self.name_match_ranges = Some(name_match_ranges);
        self
    }

    pub fn with_value_match_ranges(
        mut self,
        value_match_ranges: Vec<(u32, u32)>,
    ) -> Self {
        self.value_match_ranges = Some(value_match_ranges);
        self
    }

    pub fn with_icon(mut self, icon: FileIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn with_line_number(mut self, line_number: usize) -> Self {
        self.line_number = Some(line_number);
        self
    }

    pub fn display_name(&self) -> &str {
        self.display_name.as_ref().unwrap_or(&self.name)
    }

    pub fn stdout_repr(&self) -> String {
        let mut repr = self.name.clone();
        if let Some(line_number) = self.line_number {
            repr.push_str(&format!(":{line_number}"));
        }
        if let Some(preview) = &self.value {
            repr.push_str(&format!("\n{preview}"));
        }
        repr
    }
}

pub const ENTRY_PLACEHOLDER: Entry = Entry {
    name: String::new(),
    display_name: None,
    value: None,
    name_match_ranges: None,
    value_match_ranges: None,
    icon: None,
    line_number: None,
    preview_type: PreviewType::EnvVar,
};
