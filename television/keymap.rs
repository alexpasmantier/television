use crate::{
    action::Action,
    config::{Binding, KeyBindings},
    event::Key,
};
use rustc_hash::FxHashMap;
use std::ops::Deref;

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

impl Keymap {
    /// Merge another keymap into this one.
    ///
    /// This will overwrite any existing keys in `self` with the keys from `other`.
    ///
    /// # Example:
    /// ```ignore
    /// let mut keymap1 = Keymap::default();
    /// keymap1.0.insert(Key::Char('a'), Action::SelectNextEntry);
    ///
    /// let keymap2 = Keymap({
    ///     let mut h = FxHashMap::default();
    ///     h.insert(Key::Char('b'), Action::SelectPrevEntry);
    ///     h.insert(Key::Char('a'), Action::Quit); // This will overwrite the previous 'a' action
    ///     h
    /// });
    ///
    /// keymap1.merge(&keymap2);
    ///
    /// assert_eq!(keymap1.0.get(&Key::Char('a')), Some(&Action::Quit));
    /// assert_eq!(keymap1.0.get(&Key::Char('b')), Some(&Action::SelectPrevEntry));
    /// ```
    pub fn merge(&mut self, other: &Keymap) {
        for (key, action) in other.iter() {
            self.0.insert(*key, action.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Binding, KeyBindings};
    use crate::event::Key;

    #[test]
    fn test_keymap_from_keybindings() {
        let keybindings = KeyBindings({
            let mut h = FxHashMap::default();
            for (action, binding) in &[
                (Action::SelectNextEntry, Binding::SingleKey(Key::Char('j'))),
                (Action::SelectPrevEntry, Binding::SingleKey(Key::Char('k'))),
                (Action::Quit, Binding::SingleKey(Key::Char('q'))),
            ] {
                h.insert(action.clone(), binding.clone());
            }
            h
        });

        let keymap: Keymap = (&keybindings).into();
        assert_eq!(
            keymap.0.get(&Key::Char('j')),
            Some(&Action::SelectNextEntry)
        );
        assert_eq!(
            keymap.0.get(&Key::Char('k')),
            Some(&Action::SelectPrevEntry)
        );
        assert_eq!(keymap.0.get(&Key::Char('q')), Some(&Action::Quit));
    }

    #[test]
    fn test_keymap_merge_into() {
        let mut keymap1 = Keymap(FxHashMap::default());
        keymap1.0.insert(Key::Char('a'), Action::SelectNextEntry);
        keymap1.0.insert(Key::Char('b'), Action::SelectPrevEntry);

        let keymap2 = Keymap({
            let mut h = FxHashMap::default();
            h.insert(Key::Char('c'), Action::Quit);
            h.insert(Key::Char('a'), Action::Quit); // This should overwrite the
            // previous 'a' action
            h
        });

        keymap1.merge(&keymap2);
        assert_eq!(keymap1.0.get(&Key::Char('a')), Some(&Action::Quit));
        assert_eq!(
            keymap1.0.get(&Key::Char('b')),
            Some(&Action::SelectPrevEntry)
        );
        assert_eq!(keymap1.0.get(&Key::Char('c')), Some(&Action::Quit));
    }
}
