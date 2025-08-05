use std::hash::Hash;

use crate::{
    action::{Action, Actions},
    config::{EventBindings, EventType, KeyBindings},
    event::{InputEvent, Key},
    utils::hashmaps::invert_flat_hashmap,
};
use crossterm::event::MouseEventKind;
use rustc_hash::FxHashMap;

/// An input map that handles both keyboard and non-keyboard input events.
///
/// This structure supports multiple-actions-per-key and handles various
/// input types including keyboard keys, mouse events, resize events, and
/// custom events.
///
/// # Fields
///
/// - `key_actions` - Maps keyboard keys to actions
/// - `event_actions` - Maps non-keyboard events to actions
///
/// # Examples
///
/// ```rust
/// use television::keymap::InputMap;
/// use television::event::{Key, InputEvent};
/// use television::action::{Action, Actions};
/// use television::config::{KeyBindings, EventType};
///
/// // Create from key bindings
/// let keybindings = KeyBindings::from(vec![
///     (Key::Char('j'), Action::SelectNextEntry),
///     (Key::Char('k'), Action::SelectPrevEntry),
/// ]);
/// let input_map: InputMap = (&keybindings).into();
///
/// // Query actions for input
/// let key_input = InputEvent::Key(Key::Char('j'));
/// let actions = input_map.get_actions_for_input(&key_input);
/// assert_eq!(actions, Some(vec![Action::SelectNextEntry]));
/// ```
/// ```
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct InputMap {
    /// Maps keyboard keys to their associated actions
    pub key_actions: FxHashMap<Key, Actions>,
    /// Maps non-keyboard events to their associated actions
    pub event_actions: FxHashMap<EventType, Actions>,

    /// Used to query key based on an `Actions` instance.
    actions_keys: FxHashMap<Actions, Key>,
}

impl Hash for InputMap {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (key, actions) in &self.key_actions {
            (key, actions).hash(state);
        }
        for (event, actions) in &self.event_actions {
            (event, actions).hash(state);
        }
    }
}

impl InputMap {
    pub fn new(
        key_actions: FxHashMap<Key, Actions>,
        event_actions: FxHashMap<EventType, Actions>,
    ) -> Self {
        let actions_keys = invert_flat_hashmap(&key_actions);
        Self {
            key_actions,
            event_actions,
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

    /// Gets all actions bound to a specific event type.
    ///
    /// Returns a reference to the `Actions` (single or multiple) bound to
    /// the given event type, or `None` if no binding exists.
    ///
    /// # Arguments
    ///
    /// * `event` - The event type to look up
    ///
    /// # Returns
    ///
    /// - `Some(&Actions)` - The actions bound to the event
    /// - `None` - No binding exists for this event type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use television::keymap::InputMap;
    /// use television::config::EventType;
    /// use television::action::{Action, Actions};
    ///
    /// let mut input_map = InputMap::default();
    /// input_map.event_actions.insert(
    ///     EventType::MouseClick,
    ///     Actions::single(Action::ConfirmSelection)
    /// );
    ///
    /// let actions = input_map.get_actions_for_event(&EventType::MouseClick).unwrap();
    /// assert_eq!(actions.as_slice(), &[Action::ConfirmSelection]);
    /// ```
    pub fn get_actions_for_event(
        &self,
        event: &EventType,
    ) -> Option<&Actions> {
        self.event_actions.get(event)
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

    /// Gets the first action bound to a specific event type (backward compatibility).
    ///
    /// This method provides backward compatibility with the old single-action
    /// binding system. For events with multiple actions, it returns only the
    /// first action. Use `get_actions_for_event()` to get all actions.
    ///
    /// # Arguments
    ///
    /// * `event` - The event type to look up
    ///
    /// # Returns
    ///
    /// - `Some(Action)` - The first action bound to the event
    /// - `None` - No binding exists for this event type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use television::keymap::InputMap;
    /// use television::config::EventType;
    /// use television::action::{Action, Actions};
    ///
    /// let mut input_map = InputMap::default();
    /// input_map.event_actions.insert(
    ///     EventType::MouseClick,
    ///     Actions::multiple(vec![Action::ConfirmSelection, Action::TogglePreview])
    /// );
    ///
    /// // Returns only the first action
    /// assert_eq!(
    ///     input_map.get_action_for_event(&EventType::MouseClick),
    ///     Some(Action::ConfirmSelection)
    /// );
    /// ```
    pub fn get_action_for_event(&self, event: &EventType) -> Option<Action> {
        self.event_actions
            .get(event)
            .and_then(|actions| actions.first().cloned())
    }

    /// Gets all actions for any input event.
    ///
    /// This is the primary method for querying actions in the new input system.
    /// It handles all types of input events and returns a vector of all actions
    /// that should be executed in response to the input.
    ///
    /// # Arguments
    ///
    /// * `input` - The input event to look up
    ///
    /// # Returns
    ///
    /// - `Some(Vec<Action>)` - All actions bound to the input event
    /// - `None` - No binding exists for this input
    ///
    /// # Supported Input Types
    ///
    /// - `InputEvent::Key` - Keyboard input
    /// - `InputEvent::Mouse` - Mouse clicks and scrolling
    /// - `InputEvent::Resize` - Terminal resize events
    /// - `InputEvent::Custom` - Custom event types
    ///
    /// # Examples
    ///
    /// ```rust
    /// use television::keymap::InputMap;
    /// use television::event::{Key, InputEvent, MouseInputEvent};
    /// use television::config::EventType;
    /// use television::action::{Action, Actions};
    /// use crossterm::event::MouseEventKind;
    ///
    /// let mut input_map = InputMap::default();
    /// input_map.key_actions.insert(
    ///     Key::Ctrl('s'),
    ///     Actions::multiple(vec![Action::ReloadSource, Action::CopyEntryToClipboard])
    /// );
    ///
    /// let key_input = InputEvent::Key(Key::Ctrl('s'));
    /// let actions = input_map.get_actions_for_input(&key_input).unwrap();
    /// assert_eq!(actions, vec![Action::ReloadSource, Action::CopyEntryToClipboard]);
    /// ```
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

    /// Gets the first action for any input event (backward compatibility).
    ///
    /// This method provides backward compatibility with the old single-action
    /// system. It handles all input types and returns only the first action
    /// bound to the input. Use `get_actions_for_input()` to get all actions.
    ///
    /// # Arguments
    ///
    /// * `input` - The input event to look up
    ///
    /// # Returns
    ///
    /// - `Some(Action)` - The first action bound to the input
    /// - `None` - No binding exists for this input
    ///
    /// # Supported Input Types
    ///
    /// - `InputEvent::Key` - Returns first action for the key
    /// - `InputEvent::Mouse` - Maps to appropriate event type and returns first action
    /// - `InputEvent::Resize` - Returns first action for resize events
    /// - `InputEvent::Custom` - Returns first action for the custom event
    ///
    /// # Examples
    ///
    /// ```rust
    /// use television::keymap::InputMap;
    /// use television::event::{Key, InputEvent};
    /// use television::action::{Action, Actions};
    ///
    /// let mut input_map = InputMap::default();
    /// input_map.key_actions.insert(
    ///     Key::Enter,
    ///     Actions::multiple(vec![Action::ConfirmSelection, Action::Quit])
    /// );
    ///
    /// let key_input = InputEvent::Key(Key::Enter);
    /// assert_eq!(
    ///     input_map.get_action_for_input(&key_input),
    ///     Some(Action::ConfirmSelection) // Only first action
    /// );
    /// ```
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

    /// Gets the key associated with a specific action.
    pub fn get_key_for_action(&self, action: &Action) -> Option<Key> {
        self.actions_keys
            .get(&Actions::single(action.clone()))
            .copied()
    }
}

impl From<&KeyBindings> for InputMap {
    /// Converts `KeyBindings` into an `InputMap`.
    ///
    /// This conversion creates an input map containing only keyboard bindings.
    /// The event bindings will be empty. Since the new `KeyBindings` structure
    /// already stores Key → Actions mappings, we can directly copy the bindings
    /// without the inversion that was needed in the old system.
    ///
    /// # Arguments
    ///
    /// * `keybindings` - The key bindings to convert
    ///
    /// # Returns
    ///
    /// An `InputMap` with the key bindings and empty event bindings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use television::keymap::InputMap;
    /// use television::config::KeyBindings;
    /// use television::event::Key;
    /// use television::action::Action;
    ///
    /// let keybindings = KeyBindings::from(vec![
    ///     (Key::Enter, Action::ConfirmSelection),
    ///     (Key::Esc, Action::Quit),
    /// ]);
    ///
    /// let input_map: InputMap = (&keybindings).into();
    /// assert_eq!(input_map.get_action_for_key(&Key::Enter), Some(Action::ConfirmSelection));
    /// assert!(input_map.event_actions.is_empty());
    /// ```
    fn from(keybindings: &KeyBindings) -> Self {
        Self {
            key_actions: keybindings.inner.clone(),
            event_actions: FxHashMap::default(),
            actions_keys: invert_flat_hashmap(&keybindings.inner),
        }
    }
}

impl From<&EventBindings> for InputMap {
    /// Converts `EventBindings` into an `InputMap`.
    ///
    /// This conversion creates an input map containing only event bindings.
    /// The key bindings will be empty. This is useful when you want to
    /// handle only non-keyboard events.
    ///
    /// # Arguments
    ///
    /// * `event_bindings` - The event bindings to convert
    ///
    /// # Returns
    ///
    /// An `InputMap` with the event bindings and empty key bindings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use television::keymap::InputMap;
    /// use television::config::{EventBindings, EventType};
    /// use television::action::Action;
    ///
    /// let event_bindings = EventBindings::from(vec![
    ///     (EventType::MouseClick, Action::ConfirmSelection),
    ///     (EventType::Resize, Action::ClearScreen),
    /// ]);
    ///
    /// let input_map: InputMap = (&event_bindings).into();
    /// assert_eq!(
    ///     input_map.get_action_for_event(&EventType::MouseClick),
    ///     Some(Action::ConfirmSelection)
    /// );
    /// assert!(input_map.key_actions.is_empty());
    /// ```
    fn from(event_bindings: &EventBindings) -> Self {
        Self {
            key_actions: FxHashMap::default(),
            event_actions: event_bindings.inner.clone(),
            actions_keys: FxHashMap::default(),
        }
    }
}

impl From<(&KeyBindings, &EventBindings)> for InputMap {
    /// Converts both `KeyBindings` and `EventBindings` into an `InputMap`.
    ///
    /// This conversion creates a complete input map with both keyboard and
    /// event bindings. This is the most common way to create a fully
    /// configured input map that handles all types of input events.
    ///
    /// # Arguments
    ///
    /// * `keybindings` - The keyboard bindings to include
    /// * `event_bindings` - The event bindings to include
    ///
    /// # Returns
    ///
    /// An `InputMap` containing both key and event bindings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use television::keymap::InputMap;
    /// use television::config::{KeyBindings, EventBindings, EventType};
    /// use television::event::Key;
    /// use television::action::Action;
    ///
    /// let keybindings = KeyBindings::from(vec![
    ///     (Key::Enter, Action::ConfirmSelection),
    /// ]);
    /// let event_bindings = EventBindings::from(vec![
    ///     (EventType::MouseClick, Action::ConfirmSelection),
    /// ]);
    ///
    /// let input_map: InputMap = (&keybindings, &event_bindings).into();
    /// assert_eq!(input_map.get_action_for_key(&Key::Enter), Some(Action::ConfirmSelection));
    /// assert_eq!(
    ///     input_map.get_action_for_event(&EventType::MouseClick),
    ///     Some(Action::ConfirmSelection)
    /// );
    /// ```
    fn from(
        (keybindings, event_bindings): (&KeyBindings, &EventBindings),
    ) -> Self {
        Self {
            key_actions: keybindings.inner.clone(),
            event_actions: event_bindings.inner.clone(),
            actions_keys: invert_flat_hashmap(&keybindings.inner),
        }
    }
}

impl InputMap {
    /// Merges another `InputMap` into this one.
    ///
    /// This method combines the bindings from another `InputMap` into this one,
    /// with bindings from `other` taking precedence over existing bindings.
    /// This is useful for applying configuration hierarchies and user customizations.
    ///
    /// # Arguments
    ///
    /// * `other` - The input map to merge into this one
    ///
    /// # Behavior
    ///
    /// - Existing keys/events in `self` are overwritten by those in `other`
    /// - New keys/events from `other` are added to `self`
    /// - Both key bindings and event bindings are merged
    ///
    /// # Examples
    ///
    /// ```rust
    /// use television::keymap::InputMap;
    /// use television::event::Key;
    /// use television::action::{Action, Actions};
    ///
    /// let mut base_map = InputMap::default();
    /// base_map.key_actions.insert(Key::Enter, Actions::single(Action::ConfirmSelection));
    ///
    /// let mut custom_map = InputMap::default();
    /// custom_map.key_actions.insert(Key::Enter, Actions::single(Action::Quit)); // Override
    /// custom_map.key_actions.insert(Key::Esc, Actions::single(Action::Quit));   // New binding
    ///
    /// base_map.merge(&custom_map);
    /// assert_eq!(base_map.get_action_for_key(&Key::Enter), Some(Action::Quit));
    /// assert_eq!(base_map.get_action_for_key(&Key::Esc), Some(Action::Quit));
    /// ```
    pub fn merge(&mut self, other: &InputMap) {
        for (key, action) in &other.key_actions {
            self.key_actions.insert(*key, action.clone());
        }
        for (event, action) in &other.event_actions {
            self.event_actions.insert(event.clone(), action.clone());
        }
        // Update actions_keys to reflect the merged actions
        for (key, actions) in &other.key_actions {
            self.actions_keys.insert(actions.clone(), *key);
        }
    }

    /// Merges key bindings into this `InputMap`.
    ///
    /// This method adds all key bindings from a `KeyBindings` configuration
    /// into this input map, overwriting any existing bindings for the same keys.
    ///
    /// # Arguments
    ///
    /// * `keybindings` - The key bindings to merge
    ///
    /// # Examples
    ///
    /// ```rust
    /// use television::keymap::InputMap;
    /// use television::config::KeyBindings;
    /// use television::event::Key;
    /// use television::action::Action;
    ///
    /// let mut input_map = InputMap::default();
    /// let keybindings = KeyBindings::from(vec![
    ///     (Key::Enter, Action::ConfirmSelection),
    ///     (Key::Esc, Action::Quit),
    /// ]);
    ///
    /// input_map.merge_key_bindings(&keybindings);
    /// assert_eq!(input_map.get_action_for_key(&Key::Enter), Some(Action::ConfirmSelection));
    /// ```
    pub fn merge_key_bindings(&mut self, keybindings: &KeyBindings) {
        for (key, action) in &keybindings.inner {
            self.key_actions.insert(*key, action.clone());
        }
        // Update actions_keys to reflect the merged actions
        for (key, actions) in &keybindings.inner {
            self.actions_keys.insert(actions.clone(), *key);
        }
    }

    /// Merges event bindings into this `InputMap`.
    ///
    /// This method adds all event bindings from an `EventBindings` configuration
    /// into this input map, overwriting any existing bindings for the same events.
    ///
    /// # Arguments
    ///
    /// * `event_bindings` - The event bindings to merge
    ///
    /// # Examples
    ///
    /// ```rust
    /// use television::keymap::InputMap;
    /// use television::config::{EventBindings, EventType};
    /// use television::action::Action;
    ///
    /// let mut input_map = InputMap::default();
    /// let event_bindings = EventBindings::from(vec![
    ///     (EventType::MouseClick, Action::ConfirmSelection),
    ///     (EventType::Resize, Action::ClearScreen),
    /// ]);
    ///
    /// input_map.merge_event_bindings(&event_bindings);
    /// assert_eq!(
    ///     input_map.get_action_for_event(&EventType::MouseClick),
    ///     Some(Action::ConfirmSelection)
    /// );
    /// ```
    pub fn merge_event_bindings(&mut self, event_bindings: &EventBindings) {
        for (event, action) in &event_bindings.inner {
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
            Actions::multiple(vec![
                Action::ReloadSource,
                Action::CopyEntryToClipboard,
            ]),
        );
        key_actions.insert(Key::Esc, Actions::single(Action::Quit));

        let input_map = InputMap {
            key_actions,
            event_actions: FxHashMap::default(),
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
    fn test_input_map_multiple_actions_for_input_event() {
        let mut input_map = InputMap::default();
        input_map.key_actions.insert(
            Key::Char('j'),
            Actions::multiple(vec![
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
            Actions::multiple(vec![
                Action::ConfirmSelection,
                Action::TogglePreview,
            ]),
        );

        let input_map = InputMap {
            key_actions: FxHashMap::default(),
            event_actions,
            actions_keys: FxHashMap::default(),
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

        let keybindings = KeyBindings { inner: bindings };
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
