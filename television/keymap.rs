use rustc_hash::FxHashMap;
use std::ops::Deref;

use crate::action::Action;
use crate::config::{Binding, KeyBindings};
use crate::event::Key;

#[derive(Default, Debug)]
/// A keymap is a set of mappings of keys to actions for every mode.
///
/// # Example:
/// ```ignore
///     Keymap {
///         Key::Char('j') => Action::MoveDown,
///         Key::Char('k') => Action::MoveUp,
///         Key::Char('q') => Action::Quit,
///     }
/// ```
pub struct Keymap(pub FxHashMap<Key, Action>);

impl Deref for Keymap {
    type Target = FxHashMap<Key, Action>;
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
        for (action, binding) in keybindings.iter() {
            match binding {
                Binding::SingleKey(key) => {
                    keymap.insert(*key, action.clone());
                }
                Binding::MultipleKeys(keys) => {
                    for key in keys {
                        keymap.insert(*key, action.clone());
                    }
                }
            }
        }
        Self(keymap)
    }
}
