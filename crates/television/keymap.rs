use rustc_hash::FxHashMap;
use std::ops::Deref;

use color_eyre::Result;
use television_screen::mode::Mode;

use crate::action::Action;
use crate::config::{Binding, KeyBindings};
use crate::event::Key;

#[derive(Default, Debug)]
pub struct Keymap(pub FxHashMap<Mode, FxHashMap<Key, Action>>);

impl Deref for Keymap {
    type Target = FxHashMap<Mode, FxHashMap<Key, Action>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&KeyBindings> for Keymap {
    fn from(keybindings: &KeyBindings) -> Self {
        let mut keymap = FxHashMap::default();
        for (mode, bindings) in keybindings.iter() {
            let mut mode_keymap = FxHashMap::default();
            for (action, binding) in bindings {
                match binding {
                    Binding::SingleKey(key) => {
                        mode_keymap.insert(*key, action.clone());
                    }
                    Binding::MultipleKeys(keys) => {
                        for key in keys {
                            mode_keymap.insert(*key, action.clone());
                        }
                    }
                }
            }
            keymap.insert(*mode, mode_keymap);
        }
        Self(keymap)
    }
}

impl Keymap {
    pub fn with_mode_mappings(
        mut self,
        mode: Mode,
        mappings: Vec<(Key, Action)>,
    ) -> Result<Self> {
        let mode_keymap = self.0.get_mut(&mode).ok_or_else(|| {
            color_eyre::eyre::eyre!("Mode {:?} not found", mode)
        })?;
        for (key, action) in mappings {
            mode_keymap.insert(key, action);
        }
        Ok(self)
    }
}
