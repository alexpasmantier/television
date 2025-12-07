use std::hash::Hash;

use crate::{
    action::{Action, Actions},
    config::Keybindings,
    event::Key,
    utils::hashmaps::invert_hashmap,
};
use rustc_hash::FxHashMap;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct InputMap {
    /// Maps keyboard keys to their associated actions
    pub key_actions: FxHashMap<Key, Actions>,

    /// Used to query key based on an `Actions` instance.
    actions_keys: FxHashMap<Actions, Key>,
}

impl Hash for InputMap {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (key, actions) in &self.key_actions {
            (key, actions).hash(state);
        }
    }
}

impl InputMap {
    pub fn new(key_actions: FxHashMap<Key, Actions>) -> Self {
        let actions_keys = invert_hashmap(&key_actions);
        Self {
            key_actions,
            actions_keys,
        }
    }

    /// Gets all actions bound to a specific key.
    ///
    /// Returns a reference to the `Actions` (single or multiple) bound to
    /// the given key, or `None` if no binding exists.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// - `Some(&Actions)` - The actions bound to the key
    /// - `None` - No binding exists for this key
    ///
    /// # Examples
    ///
    /// ```rust
    /// use television::keymap::InputMap;
    /// use television::event::Key;
    /// use television::action::{Action, Actions};
    ///
    /// let mut input_map = InputMap::default();
    /// input_map.key_actions.insert(Key::Enter, Actions::single(Action::ConfirmSelection));
    ///
    /// let actions = input_map.get_actions_for_key(&Key::Enter).unwrap();
    /// assert_eq!(actions.as_slice(), &[Action::ConfirmSelection]);
    /// ```
    pub fn get_actions_for_key(&self, key: &Key) -> Option<&Actions> {
        self.key_actions.get(key)
    }

    /// Gets the first action bound to a specific key (backward compatibility).
    ///
    /// This method provides backward compatibility with the old single-action
    /// binding system. For keys with multiple actions, it returns only the
    /// first action. Use `get_actions_for_key()` to get all actions.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// - `Some(Action)` - The first action bound to the key
    /// - `None` - No binding exists for this key
    ///
    /// # Examples
    ///
    /// ```rust
    /// use television::keymap::InputMap;
    /// use television::event::Key;
    /// use television::action::{Action, Actions};
    ///
    /// let mut input_map = InputMap::default();
    /// input_map.key_actions.insert(
    ///     Key::Ctrl('r'),
    ///     Actions::multiple(vec![Action::ReloadSource, Action::ClearScreen])
    /// );
    ///
    /// // Returns only the first action
    /// assert_eq!(input_map.get_action_for_key(&Key::Ctrl('r')), Some(Action::ReloadSource));
    /// ```
    pub fn get_action_for_key(&self, key: &Key) -> Option<Action> {
        self.key_actions
            .get(key)
            .and_then(|actions| actions.first().cloned())
    }

    /// Gets the key associated with a specific action.
    pub fn get_key_for_action(&self, action: &Action) -> Option<Key> {
        self.actions_keys
            .get(&Actions::single(action.clone()))
            .copied()
    }
}

impl From<Keybindings> for InputMap {
    fn from(keybindings: Keybindings) -> Self {
        Self {
            actions_keys: invert_hashmap(&keybindings),
            key_actions: keybindings.0,
        }
    }
}

impl From<&Keybindings> for InputMap {
    fn from(keybindings: &Keybindings) -> Self {
        Self {
            key_actions: keybindings.0.clone(),
            actions_keys: invert_hashmap(keybindings),
        }
    }
}

impl InputMap {
    pub fn merge(&mut self, other: &InputMap) {
        for (key, action) in &other.key_actions {
            self.key_actions.insert(*key, action.clone());
        }
        // Update actions_keys to reflect the merged actions
        for (key, actions) in &other.key_actions {
            self.actions_keys.insert(actions.clone(), *key);
        }
    }

    pub fn merge_key_bindings(&mut self, keybindings: &Keybindings) {
        for (key, action) in keybindings.iter() {
            self.key_actions.insert(*key, action.clone());
        }
        // Update actions_keys to reflect the merged actions
        for (key, actions) in keybindings.iter() {
            self.actions_keys.insert(actions.clone(), *key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Key;

    #[test]
    fn test_input_map_from_keybindings() {
        let keybindings = Keybindings::from(vec![
            (Key::Char('j'), Action::SelectNextEntry),
            (Key::Char('k'), Action::SelectPrevEntry),
            (Key::Char('q'), Action::Quit),
        ]);

        let input_map: InputMap = (&keybindings).into();
        assert_eq!(
            input_map.get_action_for_key(&Key::Char('j')),
            Some(Action::SelectNextEntry)
        );
        assert_eq!(
            input_map.get_action_for_key(&Key::Char('k')),
            Some(Action::SelectPrevEntry)
        );
        assert_eq!(
            input_map.get_action_for_key(&Key::Char('q')),
            Some(Action::Quit)
        );
    }

    #[test]
    fn test_input_map_merge() {
        let mut input_map1 = InputMap::default();
        input_map1
            .key_actions
            .insert(Key::Char('a'), Action::SelectNextEntry.into());
        input_map1
            .key_actions
            .insert(Key::Char('b'), Action::SelectPrevEntry.into());

        let mut input_map2 = InputMap::default();
        input_map2
            .key_actions
            .insert(Key::Char('c'), Action::Quit.into());
        input_map2
            .key_actions
            .insert(Key::Char('a'), Action::Quit.into()); // This should overwrite

        input_map1.merge(&input_map2);
        assert_eq!(
            input_map1.get_action_for_key(&Key::Char('a')),
            Some(Action::Quit)
        );
        assert_eq!(
            input_map1.get_action_for_key(&Key::Char('b')),
            Some(Action::SelectPrevEntry)
        );
        assert_eq!(
            input_map1.get_action_for_key(&Key::Char('c')),
            Some(Action::Quit)
        );
    }

    #[test]
    fn test_input_map_multiple_actions_per_key() {
        let mut key_actions = FxHashMap::default();
        key_actions.insert(
            Key::Ctrl('s'),
            Actions::multiple(vec![
                Action::ReloadSource,
                Action::CopyEntryToClipboard,
            ]),
        );
        key_actions.insert(Key::Esc, Actions::single(Action::Quit));

        let input_map = InputMap {
            key_actions,
            actions_keys: FxHashMap::default(),
        };

        // Test getting all actions for multiple action binding
        let ctrl_s_actions =
            input_map.get_actions_for_key(&Key::Ctrl('s')).unwrap();
        assert_eq!(
            ctrl_s_actions.as_slice(),
            &[Action::ReloadSource, Action::CopyEntryToClipboard]
        );

        // Test backward compatibility method returns first action
        assert_eq!(
            input_map.get_action_for_key(&Key::Ctrl('s')),
            Some(Action::ReloadSource)
        );

        // Test single action still works
        assert_eq!(
            input_map.get_action_for_key(&Key::Esc),
            Some(Action::Quit)
        );
    }

    #[test]
    fn test_input_map_merge_multiple_actions() {
        let mut input_map1 = InputMap::default();
        input_map1
            .key_actions
            .insert(Key::Char('a'), Actions::single(Action::SelectNextEntry));

        let mut input_map2 = InputMap::default();
        input_map2.key_actions.insert(
            Key::Char('a'),
            Actions::multiple(vec![Action::ReloadSource, Action::Quit]), // This should overwrite
        );
        input_map2.key_actions.insert(
            Key::Char('b'),
            Actions::multiple(vec![Action::TogglePreview, Action::ToggleHelp]),
        );

        input_map1.merge(&input_map2);

        // Verify the multiple actions overwrite worked
        let actions_a =
            input_map1.get_actions_for_key(&Key::Char('a')).unwrap();
        assert_eq!(
            actions_a.as_slice(),
            &[Action::ReloadSource, Action::Quit]
        );

        // Verify the new multiple actions were added
        let actions_b =
            input_map1.get_actions_for_key(&Key::Char('b')).unwrap();
        assert_eq!(
            actions_b.as_slice(),
            &[Action::TogglePreview, Action::ToggleHelp]
        );
    }

    #[test]
    fn test_input_map_from_keybindings_with_multiple_actions() {
        let mut bindings = FxHashMap::default();
        bindings.insert(
            Key::Ctrl('r'),
            Actions::multiple(vec![Action::ReloadSource, Action::ClearScreen]),
        );
        bindings.insert(Key::Esc, Actions::single(Action::Quit));

        let input_map: InputMap = Keybindings(bindings).into();

        // Test multiple actions are preserved
        let ctrl_r_actions =
            input_map.get_actions_for_key(&Key::Ctrl('r')).unwrap();
        assert_eq!(
            ctrl_r_actions.as_slice(),
            &[Action::ReloadSource, Action::ClearScreen]
        );

        // Test single actions still work
        let esc_actions = input_map.get_actions_for_key(&Key::Esc).unwrap();
        assert_eq!(esc_actions.as_slice(), &[Action::Quit]);
    }
}
