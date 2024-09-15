use crate::display;
use crate::finders;
use crate::previewers;

pub struct EnvVarPreviewer {}

impl EnvVarPreviewer {
    pub fn new() -> Self {
        EnvVarPreviewer {}
    }
}

impl previewers::Previewer<finders::EnvVarEntry> for EnvVarPreviewer {
    fn preview(&self, e: finders::EnvVarEntry) -> previewers::Preview {
        previewers::Preview {
            title: e.name.clone(),
            content: vec![display::StyledString {
                text: e.value.clone(),
                style: display::StyleGroup::Entry,
            }],
        }
    }
}
