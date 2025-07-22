use crate::{
    action::{Action, Actions},
    config::{EventBindings, EventType, KeyBindings},
    event::{InputEvent, Key},
};
use crossterm::event::MouseEventKind;
use rustc_hash::FxHashMap;

#[derive(Default, Debug, Clone)]
/// An input map that handles both keyboard and non-keyboard input events.
///
/// This replaces the old Keymap structure and provides unified access to
/// both key bindings and event bindings through a single interface.
///
/// # Example:
/// ```ignore
///     InputMap {
///         Key::Char('j') => Action::MoveDown,
///         Key::Char('k') => Action::MoveUp,
///         EventType::MouseClick => Action::ConfirmSelection,
///     }
/// ```
pub struct InputMap {
    pub key_actions: FxHashMap<Key, Actions>,
    pub event_actions: FxHashMap<EventType, Actions>,
}

impl InputMap {
    /// Create a new empty `InputMap`
    pub fn new() -> Self {
        Self {
            key_actions: FxHashMap::default(),
            event_actions: FxHashMap::default(),
        }
    }

    /// Get the actions for a given key
    pub fn get_actions_for_key(&self, key: &Key) -> Option<&Actions> {
        self.key_actions.get(key)
    }

    /// Get the actions for a given event type
    pub fn get_actions_for_event(
        &self,
        event: &EventType,
    ) -> Option<&Actions> {
        self.event_actions.get(event)
    }

    /// Get the action for a given key (backward compatibility)
    pub fn get_action_for_key(&self, key: &Key) -> Option<Action> {
        self.key_actions.get(key).and_then(|actions| match actions {
            Actions::Single(action) => Some(action.clone()),
            Actions::Multiple(actions_vec) => actions_vec.first().cloned(),
        })
    }

    /// Get the action for a given event type (backward compatibility)
    pub fn get_action_for_event(&self, event: &EventType) -> Option<Action> {
        self.event_actions
            .get(event)
            .and_then(|actions| match actions {
                Actions::Single(action) => Some(action.clone()),
                Actions::Multiple(actions_vec) => actions_vec.first().cloned(),
            })
    }

    /// Get all actions for any input event
    pub fn get_actions_for_input(
        &self,
        input: &InputEvent,
    ) -> Option<Vec<Action>> {
        match input {
            InputEvent::Key(key) => self
                .get_actions_for_key(key)
                .map(|actions| actions.as_slice().to_vec()),
            InputEvent::Mouse(mouse_event) => {
                let event_type = match mouse_event.kind {
                    MouseEventKind::Down(_) => EventType::MouseClick,
                    MouseEventKind::ScrollUp => EventType::MouseScrollUp,
                    MouseEventKind::ScrollDown => EventType::MouseScrollDown,
                    _ => return None,
                };
                self.get_actions_for_event(&event_type)
                    .map(|actions| actions.as_slice().to_vec())
            }
            _ => None,
        }
    }

    /// Get an action for any input event (backward compatibility)
    pub fn get_action_for_input(&self, input: &InputEvent) -> Option<Action> {
        match input {
            InputEvent::Key(key) => self.get_action_for_key(key),
            InputEvent::Mouse(mouse_event) => {
                let event_type = match mouse_event.kind {
                    MouseEventKind::Down(_) => EventType::MouseClick,
                    MouseEventKind::ScrollUp => EventType::MouseScrollUp,
                    MouseEventKind::ScrollDown => EventType::MouseScrollDown,
                    _ => return None,
                };
                self.get_action_for_event(&event_type)
            }
            InputEvent::Resize(_, _) => {
                self.get_action_for_event(&EventType::Resize)
            }
            InputEvent::Custom(name) => {
                self.get_action_for_event(&EventType::Custom(name.clone()))
            }
        }
    }
}

impl From<&KeyBindings> for InputMap {
    /// Convert a `KeyBindings` into an `InputMap`.
    ///
    /// Since the new `KeyBindings` already store Key -> Action mappings,
    /// we can directly copy the bindings without inversion.
    fn from(keybindings: &KeyBindings) -> Self {
        Self {
            key_actions: keybindings.bindings.clone(),
            event_actions: FxHashMap::default(),
        }
    }
}

impl From<&EventBindings> for InputMap {
    /// Convert `EventBindings` into an `InputMap`.
    fn from(event_bindings: &EventBindings) -> Self {
        Self {
            key_actions: FxHashMap::default(),
            event_actions: event_bindings.bindings.clone(),
        }
    }
}

impl From<(&KeyBindings, &EventBindings)> for InputMap {
    /// Convert both `KeyBindings` and `EventBindings` into an `InputMap`.
    fn from(
        (keybindings, event_bindings): (&KeyBindings, &EventBindings),
    ) -> Self {
        Self {
            key_actions: keybindings.bindings.clone(),
            event_actions: event_bindings.bindings.clone(),
        }
    }
}

impl InputMap {
    /// Merge another `InputMap` into this one.
    ///
    /// This will overwrite any existing keys/events in `self` with the mappings from `other`.
    pub fn merge(&mut self, other: &InputMap) {
        for (key, action) in &other.key_actions {
            self.key_actions.insert(*key, action.clone());
        }
        for (event, action) in &other.event_actions {
            self.event_actions.insert(event.clone(), action.clone());
        }
    }

    /// Merge key bindings into this `InputMap`
    pub fn merge_key_bindings(&mut self, keybindings: &KeyBindings) {
        for (key, action) in &keybindings.bindings {
            self.key_actions.insert(*key, action.clone());
        }
    }

    /// Merge event bindings into this `InputMap`
    pub fn merge_event_bindings(&mut self, event_bindings: &EventBindings) {
        for (event, action) in &event_bindings.bindings {
            self.event_actions.insert(event.clone(), action.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{EventBindings, KeyBindings};
    use crate::event::{Key, MouseInputEvent};
    use crossterm::event::MouseEventKind;

    #[test]
    fn test_input_map_from_keybindings() {
        let keybindings = KeyBindings::from(vec![
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
    fn test_input_map_from_event_bindings() {
        let event_bindings = EventBindings::from(vec![
            (EventType::MouseClick, Action::ConfirmSelection),
            (EventType::Resize, Action::ClearScreen),
        ]);

        let input_map: InputMap = (&event_bindings).into();
        assert_eq!(
            input_map.get_action_for_event(&EventType::MouseClick),
            Some(Action::ConfirmSelection)
        );
        assert_eq!(
            input_map.get_action_for_event(&EventType::Resize),
            Some(Action::ClearScreen)
        );
    }

    #[test]
    fn test_input_map_get_action_for_input() {
        let keybindings =
            KeyBindings::from(vec![(Key::Char('j'), Action::SelectNextEntry)]);
        let event_bindings = EventBindings::from(vec![(
            EventType::MouseClick,
            Action::ConfirmSelection,
        )]);

        let input_map: InputMap = (&keybindings, &event_bindings).into();

        // Test key input
        let key_input = InputEvent::Key(Key::Char('j'));
        assert_eq!(
            input_map.get_action_for_input(&key_input),
            Some(Action::SelectNextEntry)
        );

        // Test mouse input
        let mouse_input = InputEvent::Mouse(MouseInputEvent {
            kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
            position: (10, 10),
        });
        assert_eq!(
            input_map.get_action_for_input(&mouse_input),
            Some(Action::ConfirmSelection)
        );
    }

    #[test]
    fn test_input_map_merge() {
        let mut input_map1 = InputMap::new();
        input_map1
            .key_actions
            .insert(Key::Char('a'), Action::SelectNextEntry.into());
        input_map1
            .key_actions
            .insert(Key::Char('b'), Action::SelectPrevEntry.into());

        let mut input_map2 = InputMap::new();
        input_map2
            .key_actions
            .insert(Key::Char('c'), Action::Quit.into());
        input_map2
            .key_actions
            .insert(Key::Char('a'), Action::Quit.into()); // This should overwrite
        input_map2
            .event_actions
            .insert(EventType::MouseClick, Action::ConfirmSelection.into());

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
        assert_eq!(
            input_map1.get_action_for_event(&EventType::MouseClick),
            Some(Action::ConfirmSelection)
        );
    }

    #[test]
    fn test_input_map_multiple_actions_per_key() {
        let mut key_actions = FxHashMap::default();
        key_actions.insert(
            Key::Ctrl('s'),
            Actions::Multiple(vec![
                Action::ReloadSource,
                Action::CopyEntryToClipboard,
            ]),
        );
        key_actions.insert(Key::Esc, Actions::Single(Action::Quit));

        let input_map = InputMap {
            key_actions,
            event_actions: FxHashMap::default(),
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
    fn test_input_map_multiple_actions_for_input_event() {
        let mut input_map = InputMap::new();
        input_map.key_actions.insert(
            Key::Char('j'),
            Actions::Multiple(vec![
                Action::SelectNextEntry,
                Action::ScrollPreviewDown,
            ]),
        );

        let key_input = InputEvent::Key(Key::Char('j'));
        let actions = input_map.get_actions_for_input(&key_input).unwrap();

        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0], Action::SelectNextEntry);
        assert_eq!(actions[1], Action::ScrollPreviewDown);

        // Test backward compatibility returns first action
        assert_eq!(
            input_map.get_action_for_input(&key_input),
            Some(Action::SelectNextEntry)
        );
    }

    #[test]
    fn test_input_map_multiple_actions_for_events() {
        let mut event_actions = FxHashMap::default();
        event_actions.insert(
            EventType::MouseClick,
            Actions::Multiple(vec![
                Action::ConfirmSelection,
                Action::TogglePreview,
            ]),
        );

        let input_map = InputMap {
            key_actions: FxHashMap::default(),
            event_actions,
        };

        // Test mouse input with multiple actions
        let mouse_input = InputEvent::Mouse(MouseInputEvent {
            kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
            position: (10, 10),
        });

        let actions = input_map.get_actions_for_input(&mouse_input).unwrap();
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0], Action::ConfirmSelection);
        assert_eq!(actions[1], Action::TogglePreview);

        // Test backward compatibility returns first action
        assert_eq!(
            input_map.get_action_for_input(&mouse_input),
            Some(Action::ConfirmSelection)
        );
    }

    #[test]
    fn test_input_map_merge_multiple_actions() {
        let mut input_map1 = InputMap::new();
        input_map1
            .key_actions
            .insert(Key::Char('a'), Actions::Single(Action::SelectNextEntry));

        let mut input_map2 = InputMap::new();
        input_map2.key_actions.insert(
            Key::Char('a'),
            Actions::Multiple(vec![Action::ReloadSource, Action::Quit]), // This should overwrite
        );
        input_map2.key_actions.insert(
            Key::Char('b'),
            Actions::Multiple(vec![Action::TogglePreview, Action::ToggleHelp]),
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
            Actions::Multiple(vec![Action::ReloadSource, Action::ClearScreen]),
        );
        bindings.insert(Key::Esc, Actions::Single(Action::Quit));

        let keybindings = KeyBindings { bindings };
        let input_map: InputMap = (&keybindings).into();

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
