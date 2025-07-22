use crate::{
    action::Action,
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
    pub key_actions: FxHashMap<Key, Action>,
    pub event_actions: FxHashMap<EventType, Action>,
}

impl InputMap {
    /// Create a new empty `InputMap`
    pub fn new() -> Self {
        Self {
            key_actions: FxHashMap::default(),
            event_actions: FxHashMap::default(),
        }
    }

    /// Get the action for a given key
    pub fn get_action_for_key(&self, key: &Key) -> Option<&Action> {
        self.key_actions.get(key)
    }

    /// Get the action for a given event type
    pub fn get_action_for_event(&self, event: &EventType) -> Option<&Action> {
        self.event_actions.get(event)
    }

    /// Get an action for any input event
    pub fn get_action_for_input(&self, input: &InputEvent) -> Option<Action> {
        match input {
            InputEvent::Key(key) => self.get_action_for_key(key).cloned(),
            InputEvent::Mouse(mouse_event) => {
                let event_type = match mouse_event.kind {
                    MouseEventKind::Down(_) => EventType::MouseClick,
                    MouseEventKind::ScrollUp => EventType::MouseScrollUp,
                    MouseEventKind::ScrollDown => EventType::MouseScrollDown,
                    _ => return None,
                };
                self.get_action_for_event(&event_type).cloned()
            }
            InputEvent::Resize(_, _) => {
                self.get_action_for_event(&EventType::Resize).cloned()
            }
            InputEvent::Custom(name) => self
                .get_action_for_event(&EventType::Custom(name.clone()))
                .cloned(),
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
            Some(&Action::SelectNextEntry)
        );
        assert_eq!(
            input_map.get_action_for_key(&Key::Char('k')),
            Some(&Action::SelectPrevEntry)
        );
        assert_eq!(
            input_map.get_action_for_key(&Key::Char('q')),
            Some(&Action::Quit)
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
            Some(&Action::ConfirmSelection)
        );
        assert_eq!(
            input_map.get_action_for_event(&EventType::Resize),
            Some(&Action::ClearScreen)
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
            .insert(Key::Char('a'), Action::SelectNextEntry);
        input_map1
            .key_actions
            .insert(Key::Char('b'), Action::SelectPrevEntry);

        let mut input_map2 = InputMap::new();
        input_map2.key_actions.insert(Key::Char('c'), Action::Quit);
        input_map2.key_actions.insert(Key::Char('a'), Action::Quit); // This should overwrite
        input_map2
            .event_actions
            .insert(EventType::MouseClick, Action::ConfirmSelection);

        input_map1.merge(&input_map2);
        assert_eq!(
            input_map1.get_action_for_key(&Key::Char('a')),
            Some(&Action::Quit)
        );
        assert_eq!(
            input_map1.get_action_for_key(&Key::Char('b')),
            Some(&Action::SelectPrevEntry)
        );
        assert_eq!(
            input_map1.get_action_for_key(&Key::Char('c')),
            Some(&Action::Quit)
        );
        assert_eq!(
            input_map1.get_action_for_event(&EventType::MouseClick),
            Some(&Action::ConfirmSelection)
        );
    }
}
