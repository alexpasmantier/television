use rustc_hash::FxHashMap;
use std::ops::Deref;

use crate::television::Mode;

use crate::action::Action;
use crate::config::{Binding, KeyBindings};
use crate::event::Key;

#[derive(Default, Debug)]
/// A keymap is a set of mappings of keys to actions for every mode.
///
/// # Example:
/// ```ignore
///     Keymap {
///         Mode::Channel => {
///             Key::Char('j') => Action::MoveDown,
///             Key::Char('k') => Action::MoveUp,
///             Key::Char('q') => Action::Quit,
///         },
///         Mode::Insert => {
///             Key::Ctrl('a') => Action::MoveToStart,
///         },
///     }
/// ```
pub struct Keymap(pub FxHashMap<Mode, FxHashMap<Key, Action>>);

impl Deref for Keymap {
    type Target = FxHashMap<Mode, FxHashMap<Key, Action>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&KeyBindings> for Keymap {
    /// Convert a `KeyBindings` into a `Keymap`.
    ///
    /// This essentially "reverses" the inner `KeyBindings` structure, so that each mode keymap is
    /// indexed by its keys instead of the actions so as to be used as a routing table for incoming
    /// key events.
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
    /// For a provided `Mode`, merge the given `mappings` into the keymap.
    pub fn with_mode_mappings(
        mut self,
        mode: Mode,
        mappings: Vec<(Key, Action)>,
    ) -> Self {
        let mode_keymap = self.0.entry(mode).or_default();

        for (key, action) in mappings {
            mode_keymap.insert(key, action);
        }
        self
    }
}
