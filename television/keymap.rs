use std::hash::Hash;

use crate::{
    action::{Action, Actions},
    config::{Keybindings, merge_keybindings},
    event::Key,
    television::Mode,
    utils::hashmaps::invert_hashmap,
};
use rustc_hash::FxHashMap;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct InputMap {
    pub global_keybindings: Keybindings,
    pub channel_keybindings: Keybindings,

    /// This is a reverse mapping of global and channel keybindings
    /// to facilitate lookups from actions to keys.
    actions_keys: FxHashMap<Actions, Key>,
}

impl Hash for InputMap {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (a, k) in &self.actions_keys {
            (a, k).hash(state);
        }
    }
}

impl InputMap {
    pub fn new(global: Keybindings, channel: Keybindings) -> Self {
        let merged = merge_keybindings(global.clone(), &channel);
        let actions_keys = invert_hashmap(&merged);
        Self {
            global_keybindings: global,
            channel_keybindings: channel,
            actions_keys,
        }
    }

    /// Gets all actions bound to a specific key for the current mode.
    ///
    /// - `Mode::Channel` checks both global and channel-specific keybindings.
    /// - `Mode::RemoteControl` only checks global keybindings.
    pub fn get_actions_for_key(
        &self,
        key: &Key,
        mode: &Mode,
    ) -> Option<&Actions> {
        match mode {
            Mode::RemoteControl => self.global_keybindings.get(key),
            Mode::Channel => self
                .channel_keybindings
                .get(key)
                .or_else(|| self.global_keybindings.get(key)),
        }
    }

    /// Gets the key associated with a specific action.
    pub fn get_key_for_action(&self, action: &Action) -> Option<Key> {
        self.actions_keys
            .get(&Actions::single(action.clone()))
            .copied()
    }

    /// Merges another set of global keybindings into the existing ones.
    ///
    /// This won't override channel specific keybindings.
    pub fn merge_globals_with(&mut self, keybindings: &Keybindings) {
        for (key, action) in keybindings.iter() {
            self.global_keybindings.insert(*key, action.clone());
        }
        // Update actions_keys to reflect the merged actions
        // but only if they aren't already mapped in channel specific keybindings
        for (key, actions) in keybindings.iter() {
            if self.channel_keybindings.contains_key(key) {
                continue;
            }
            self.actions_keys.insert(actions.clone(), *key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Key;

    #[test]
    fn test_input_map_multiple_actions_per_key() {
        let mut keybindings = Keybindings::default();
        keybindings.insert(
            Key::Ctrl('s'),
            Actions::multiple(vec![
                Action::ReloadSource,
                Action::CopyEntryToClipboard,
            ]),
        );
        keybindings.insert(Key::Esc, Actions::single(Action::Quit));

        let input_map = InputMap::new(keybindings, Keybindings::default());

        // Test getting all actions for multiple action binding
        let ctrl_s_actions = input_map
            .get_actions_for_key(&Key::Ctrl('s'), &Mode::Channel)
            .unwrap();
        assert_eq!(
            ctrl_s_actions.as_slice(),
            &[Action::ReloadSource, Action::CopyEntryToClipboard]
        );
    }

    #[test]
    fn test_input_map_from_keybindings_with_multiple_actions() {
        let mut bindings = Keybindings::default();
        bindings.insert(
            Key::Ctrl('r'),
            Actions::multiple(vec![Action::ReloadSource, Action::ClearScreen]),
        );
        bindings.insert(Key::Esc, Actions::single(Action::Quit));

        let input_map: InputMap =
            InputMap::new(bindings, Keybindings::default());

        // Test multiple actions are preserved
        let ctrl_r_actions = input_map
            .get_actions_for_key(&Key::Ctrl('r'), &Mode::Channel)
            .unwrap();
        assert_eq!(
            ctrl_r_actions.as_slice(),
            &[Action::ReloadSource, Action::ClearScreen]
        );

        // Test single actions still work
        let esc_actions = input_map
            .get_actions_for_key(&Key::Esc, &Mode::Channel)
            .unwrap();
        assert_eq!(esc_actions.as_slice(), &[Action::Quit]);
    }

    #[test]
    fn test_input_map_constructor_no_intersection() {
        let mut global_bindings = Keybindings::default();
        global_bindings
            .insert(Key::Enter, Actions::single(Action::ConfirmSelection));

        let mut channel_bindings = Keybindings::default();
        channel_bindings
            .insert(Key::Ctrl('x'), Actions::single(Action::DeletePrevChar));

        let input_map =
            InputMap::new(global_bindings.clone(), channel_bindings.clone());

        // Test global keybindings
        let enter_actions = input_map
            .get_actions_for_key(&Key::Enter, &Mode::Channel)
            .unwrap();
        assert_eq!(enter_actions.as_slice(), &[Action::ConfirmSelection]);

        // Test channel keybindings
        let esc_actions = input_map
            .get_actions_for_key(&Key::Ctrl('x'), &Mode::Channel)
            .unwrap();
        assert_eq!(esc_actions.as_slice(), &[Action::DeletePrevChar]);
    }

    #[test]
    fn test_input_map_constructor_with_intersection() {
        let mut global_bindings = Keybindings::default();
        global_bindings
            .insert(Key::Enter, Actions::single(Action::ConfirmSelection));
        global_bindings.insert(Key::Esc, Actions::single(Action::Quit));

        let mut channel_bindings = Keybindings::default();
        channel_bindings.insert(
            Key::Enter,
            Actions::single(Action::ExternalAction(String::from(
                "custom_enter",
            ))),
        );

        let input_map =
            InputMap::new(global_bindings.clone(), channel_bindings.clone());

        let channel_action = input_map
            .get_actions_for_key(&Key::Enter, &Mode::Channel)
            .unwrap();
        assert_eq!(
            channel_action.as_slice(),
            &[Action::ExternalAction(String::from("custom_enter"))]
        );

        let remote_action = input_map
            .get_actions_for_key(&Key::Enter, &Mode::RemoteControl)
            .unwrap();
        assert_eq!(remote_action.as_slice(), &[Action::ConfirmSelection]);
    }

    #[test]
    fn test_input_map_get_actions_for_key() {
        let mut global_bindings = Keybindings::default();
        global_bindings
            .insert(Key::Enter, Actions::single(Action::ConfirmSelection));

        let mut channel_bindings = Keybindings::default();
        channel_bindings.insert(
            Key::Enter,
            Actions::single(Action::ExternalAction(String::from(
                "custom_enter",
            ))),
        );

        let input_map =
            InputMap::new(global_bindings.clone(), channel_bindings.clone());

        let global_action = input_map
            .get_actions_for_key(&Key::Enter, &Mode::RemoteControl)
            .unwrap();
        assert_eq!(global_action.as_slice(), &[Action::ConfirmSelection]);

        let channel_action = input_map
            .get_actions_for_key(&Key::Enter, &Mode::Channel)
            .unwrap();
        assert_eq!(
            channel_action.as_slice(),
            &[Action::ExternalAction(String::from("custom_enter"))]
        );
    }
}
