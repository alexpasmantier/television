use crate::{
    action::Action,
    event::{Key, convert_raw_event_to_key},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

// Legacy binding structure for backward compatibility with shell integration
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Hash)]
#[serde(untagged)]
pub enum Binding {
    SingleKey(Key),
    MultipleKeys(Vec<Key>),
}

impl Display for Binding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Binding::SingleKey(key) => write!(f, "{key}"),
            Binding::MultipleKeys(keys) => {
                let keys_str: Vec<String> = keys
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect();
                write!(f, "{}", keys_str.join(", "))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// A set of keybindings that maps keys directly to actions.
///
/// This struct represents the new architecture where keybindings are structured as
/// Key -> Action mappings in the configuration file. This eliminates the need for
/// runtime inversion and provides better discoverability.
pub struct KeyBindings {
    #[serde(
        flatten,
        serialize_with = "serialize_key_bindings",
        deserialize_with = "deserialize_key_bindings"
    )]
    pub bindings: FxHashMap<Key, Action>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Types of events that can be bound to actions
pub enum EventType {
    MouseClick,
    MouseScrollUp,
    MouseScrollDown,
    Resize,
    Custom(String),
}

impl<'de> serde::Deserialize<'de> for EventType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "mouse-click" => Ok(EventType::MouseClick),
            "mouse-scroll-up" => Ok(EventType::MouseScrollUp),
            "mouse-scroll-down" => Ok(EventType::MouseScrollDown),
            "resize" => Ok(EventType::Resize),
            custom => Ok(EventType::Custom(custom.to_string())),
        }
    }
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::MouseClick => write!(f, "mouse-click"),
            EventType::MouseScrollUp => write!(f, "mouse-scroll-up"),
            EventType::MouseScrollDown => write!(f, "mouse-scroll-down"),
            EventType::Resize => write!(f, "resize"),
            EventType::Custom(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// A set of event bindings that maps events to actions.
pub struct EventBindings {
    #[serde(
        flatten,
        serialize_with = "serialize_event_bindings",
        deserialize_with = "deserialize_event_bindings"
    )]
    pub bindings: FxHashMap<EventType, Action>,
}

impl<I> From<I> for KeyBindings
where
    I: IntoIterator<Item = (Key, Action)>,
{
    fn from(iter: I) -> Self {
        KeyBindings {
            bindings: iter.into_iter().collect(),
        }
    }
}

impl<I> From<I> for EventBindings
where
    I: IntoIterator<Item = (EventType, Action)>,
{
    fn from(iter: I) -> Self {
        EventBindings {
            bindings: iter.into_iter().collect(),
        }
    }
}

impl Hash for KeyBindings {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash based on the bindings map
        for (key, action) in &self.bindings {
            key.hash(state);
            action.hash(state);
        }
    }
}

impl Hash for EventBindings {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash based on the bindings map
        for (event, action) in &self.bindings {
            event.hash(state);
            action.hash(state);
        }
    }
}

impl Deref for KeyBindings {
    type Target = FxHashMap<Key, Action>;
    fn deref(&self) -> &Self::Target {
        &self.bindings
    }
}

impl DerefMut for KeyBindings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bindings
    }
}

impl Deref for EventBindings {
    type Target = FxHashMap<EventType, Action>;
    fn deref(&self) -> &Self::Target {
        &self.bindings
    }
}

impl DerefMut for EventBindings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bindings
    }
}

/// Merge two sets of keybindings together.
///
/// The new keybindings will overwrite any existing ones for the same keys.
pub fn merge_keybindings(
    mut keybindings: KeyBindings,
    new_keybindings: &KeyBindings,
) -> KeyBindings {
    for (key, action) in &new_keybindings.bindings {
        keybindings.bindings.insert(*key, action.clone());
    }
    keybindings
}

/// Merge two sets of event bindings together.
///
/// The new event bindings will overwrite any existing ones for the same event types.
pub fn merge_event_bindings(
    mut event_bindings: EventBindings,
    new_event_bindings: &EventBindings,
) -> EventBindings {
    for (event_type, action) in &new_event_bindings.bindings {
        event_bindings
            .bindings
            .insert(event_type.clone(), action.clone());
    }
    event_bindings
}

impl Default for KeyBindings {
    fn default() -> Self {
        let mut bindings = FxHashMap::default();

        // Quit actions
        bindings.insert(Key::Esc, Action::Quit);
        bindings.insert(Key::Ctrl('c'), Action::Quit);

        // Navigation
        bindings.insert(Key::Down, Action::SelectNextEntry);
        bindings.insert(Key::Ctrl('n'), Action::SelectNextEntry);
        bindings.insert(Key::Ctrl('j'), Action::SelectNextEntry);
        bindings.insert(Key::Up, Action::SelectPrevEntry);
        bindings.insert(Key::Ctrl('p'), Action::SelectPrevEntry);
        bindings.insert(Key::Ctrl('k'), Action::SelectPrevEntry);

        // History navigation
        bindings.insert(Key::CtrlUp, Action::SelectPrevHistory);
        bindings.insert(Key::CtrlDown, Action::SelectNextHistory);

        // Selection actions
        bindings.insert(Key::Enter, Action::ConfirmSelection);
        bindings.insert(Key::Tab, Action::ToggleSelectionDown);
        bindings.insert(Key::BackTab, Action::ToggleSelectionUp);

        // Preview actions
        bindings.insert(Key::PageDown, Action::ScrollPreviewHalfPageDown);
        bindings.insert(Key::PageUp, Action::ScrollPreviewHalfPageUp);

        // Clipboard and toggles
        bindings.insert(Key::Ctrl('y'), Action::CopyEntryToClipboard);
        bindings.insert(Key::Ctrl('r'), Action::ReloadSource);
        bindings.insert(Key::Ctrl('s'), Action::CycleSources);

        // UI Features
        bindings.insert(Key::Ctrl('t'), Action::ToggleRemoteControl);
        bindings.insert(Key::Ctrl('o'), Action::TogglePreview);
        bindings.insert(Key::Ctrl('h'), Action::ToggleHelp);
        bindings.insert(Key::F(12), Action::ToggleStatusBar);

        // Input field actions
        bindings.insert(Key::Backspace, Action::DeletePrevChar);
        bindings.insert(Key::Ctrl('w'), Action::DeletePrevWord);
        bindings.insert(Key::Ctrl('u'), Action::DeleteLine);
        bindings.insert(Key::Delete, Action::DeleteNextChar);
        bindings.insert(Key::Left, Action::GoToPrevChar);
        bindings.insert(Key::Right, Action::GoToNextChar);
        bindings.insert(Key::Home, Action::GoToInputStart);
        bindings.insert(Key::Ctrl('a'), Action::GoToInputStart);
        bindings.insert(Key::End, Action::GoToInputEnd);
        bindings.insert(Key::Ctrl('e'), Action::GoToInputEnd);

        Self { bindings }
    }
}

impl Default for EventBindings {
    fn default() -> Self {
        let mut bindings = FxHashMap::default();

        // Mouse events
        bindings.insert(EventType::MouseScrollUp, Action::ScrollPreviewUp);
        bindings.insert(EventType::MouseScrollDown, Action::ScrollPreviewDown);

        Self { bindings }
    }
}

pub fn parse_key_event(raw: &str) -> anyhow::Result<KeyEvent, String> {
    let raw_lower = raw.to_ascii_lowercase();
    let (remaining, modifiers) = extract_modifiers(&raw_lower);
    parse_key_code_with_modifiers(remaining, modifiers)
}

fn extract_modifiers(raw: &str) -> (&str, KeyModifiers) {
    let mut modifiers = KeyModifiers::empty();
    let mut current = raw;

    loop {
        if let Some(rest) = current.strip_prefix("ctrl-") {
            modifiers.insert(KeyModifiers::CONTROL);
            current = rest;
            continue;
        }
        if let Some(rest) = current.strip_prefix("shift-") {
            modifiers.insert(KeyModifiers::SHIFT);
            current = rest;
            continue;
        }
        if let Some(rest) = current.strip_prefix("alt-") {
            modifiers.insert(KeyModifiers::ALT);
            current = rest;
            continue;
        }
        if let Some(rest) = current.strip_prefix("cmd-") {
            modifiers.insert(KeyModifiers::SUPER);
            current = rest;
            continue;
        }
        if let Some(rest) = current.strip_prefix("super-") {
            modifiers.insert(KeyModifiers::SUPER);
            current = rest;
            continue;
        }
        break;
    }

    (current, modifiers)
}

fn parse_key_code_with_modifiers(
    raw: &str,
    mut modifiers: KeyModifiers,
) -> anyhow::Result<KeyEvent, String> {
    let c = match raw {
        "esc" => KeyCode::Esc,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "backtab" => {
            modifiers.insert(KeyModifiers::SHIFT);
            KeyCode::BackTab
        }
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        "space" | " " => KeyCode::Char(' '),
        "hyphen" | "minus" => KeyCode::Char('-'),
        "tab" => KeyCode::Tab,
        c if c.len() == 1 => {
            let mut c = c.chars().next().unwrap();
            if modifiers.contains(KeyModifiers::SHIFT) {
                c = c.to_ascii_uppercase();
            }
            KeyCode::Char(c)
        }
        _ => return Err(format!("Unable to parse {raw}")),
    };
    Ok(KeyEvent::new(c, modifiers))
}

#[allow(dead_code)]
pub fn key_event_to_string(key_event: &KeyEvent) -> String {
    let char;
    let key_code = match key_event.code {
        KeyCode::Backspace => "backspace",
        KeyCode::Enter => "enter",
        KeyCode::Left => "left",
        KeyCode::Right => "right",
        KeyCode::Up => "up",
        KeyCode::Down => "down",
        KeyCode::Home => "home",
        KeyCode::End => "end",
        KeyCode::PageUp => "pageup",
        KeyCode::PageDown => "pagedown",
        KeyCode::Tab => "tab",
        KeyCode::BackTab => "backtab",
        KeyCode::Delete => "delete",
        KeyCode::Insert => "insert",
        KeyCode::F(c) => {
            char = format!("f({c})");
            &char
        }
        KeyCode::Char(' ') => "space",
        KeyCode::Char(c) => {
            char = c.to_string();
            &char
        }
        KeyCode::Esc => "esc",
        KeyCode::Null
        | KeyCode::CapsLock
        | KeyCode::Menu
        | KeyCode::ScrollLock
        | KeyCode::Media(_)
        | KeyCode::NumLock
        | KeyCode::PrintScreen
        | KeyCode::Pause
        | KeyCode::KeypadBegin
        | KeyCode::Modifier(_) => "",
    };

    let mut modifiers = Vec::with_capacity(3);

    if key_event.modifiers.intersects(KeyModifiers::CONTROL) {
        modifiers.push("ctrl");
    }

    if key_event.modifiers.intersects(KeyModifiers::SHIFT) {
        modifiers.push("shift");
    }

    #[cfg(target_os = "macos")]
    if key_event.modifiers.intersects(KeyModifiers::SUPER) {
        modifiers.push("cmd");
    }

    #[cfg(not(target_os = "macos"))]
    if key_event.modifiers.intersects(KeyModifiers::SUPER) {
        modifiers.push("super");
    }

    if key_event.modifiers.intersects(KeyModifiers::ALT) {
        modifiers.push("alt");
    }

    let mut key = modifiers.join("-");

    if !key.is_empty() {
        key.push('-');
    }
    key.push_str(key_code);

    key
}

pub fn parse_key(raw: &str) -> anyhow::Result<Key, String> {
    if raw.chars().filter(|c| *c == '>').count()
        != raw.chars().filter(|c| *c == '<').count()
    {
        return Err(format!("Unable to parse `{raw}`"));
    }
    let raw = if raw.contains("><") {
        raw
    } else {
        let raw = raw.strip_prefix('<').unwrap_or(raw);
        raw.strip_suffix('>').unwrap_or(raw)
    };

    let key_event = parse_key_event(raw)?;
    Ok(convert_raw_event_to_key(key_event))
}

/// Custom serializer for `KeyBindings` that converts `Key` enum to string for TOML compatibility
fn serialize_key_bindings<S>(
    bindings: &FxHashMap<Key, Action>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeMap;
    let mut map = serializer.serialize_map(Some(bindings.len()))?;
    for (key, action) in bindings {
        map.serialize_entry(&key.to_string(), action)?;
    }
    map.end()
}

/// Custom deserializer for `KeyBindings` that parses string keys back to `Key` enum
fn deserialize_key_bindings<'de, D>(
    deserializer: D,
) -> Result<FxHashMap<Key, Action>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{MapAccess, Visitor};
    use std::fmt;

    struct KeyBindingsVisitor;

    impl<'de> Visitor<'de> for KeyBindingsVisitor {
        type Value = FxHashMap<Key, Action>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map with string keys and action values")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            use serde::de::Error;
            use toml::Value;

            let mut bindings = FxHashMap::default();
            while let Some((key_str, raw_value)) =
                map.next_entry::<String, Value>()?
            {
                let key = parse_key(&key_str).map_err(Error::custom)?;

                match raw_value {
                    Value::Boolean(false) => {
                        // Explicitly unbind key
                        bindings.insert(key, Action::NoOp);
                    }
                    Value::Boolean(true) => {
                        // True means do nothing (keep current binding or ignore)
                    }
                    action => {
                        // Try to deserialize as Action
                        let action = Action::deserialize(action)
                            .map_err(Error::custom)?;
                        bindings.insert(key, action);
                    }
                }
            }
            Ok(bindings)
        }
    }

    deserializer.deserialize_map(KeyBindingsVisitor)
}

/// Custom serializer for `EventBindings` that converts `EventType` enum to string for TOML compatibility
fn serialize_event_bindings<S>(
    bindings: &FxHashMap<EventType, Action>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeMap;
    let mut map = serializer.serialize_map(Some(bindings.len()))?;
    for (event_type, action) in bindings {
        map.serialize_entry(&event_type.to_string(), action)?;
    }
    map.end()
}

/// Custom deserializer for `EventBindings` that parses string keys back to `EventType` enum
fn deserialize_event_bindings<'de, D>(
    deserializer: D,
) -> Result<FxHashMap<EventType, Action>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{MapAccess, Visitor};
    use std::fmt;

    struct EventBindingsVisitor;

    impl<'de> Visitor<'de> for EventBindingsVisitor {
        type Value = FxHashMap<EventType, Action>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map with string keys and action values")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut bindings = FxHashMap::default();
            while let Some((event_str, action)) =
                map.next_entry::<String, Action>()?
            {
                // Parse the event string back to EventType
                let event_type = match event_str.as_str() {
                    "mouse-click" => EventType::MouseClick,
                    "mouse-scroll-up" => EventType::MouseScrollUp,
                    "mouse-scroll-down" => EventType::MouseScrollDown,
                    "resize" => EventType::Resize,
                    custom => EventType::Custom(custom.to_string()),
                };
                bindings.insert(event_type, action);
            }
            Ok(bindings)
        }
    }

    deserializer.deserialize_map(EventBindingsVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_keys() {
        assert_eq!(
            parse_key_event("a").unwrap(),
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())
        );

        assert_eq!(
            parse_key_event("enter").unwrap(),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty())
        );

        assert_eq!(
            parse_key_event("esc").unwrap(),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::empty())
        );
    }

    #[test]
    fn test_with_modifiers() {
        assert_eq!(
            parse_key_event("ctrl-a").unwrap(),
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL)
        );

        assert_eq!(
            parse_key_event("alt-enter").unwrap(),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT)
        );

        assert_eq!(
            parse_key_event("shift-esc").unwrap(),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::SHIFT)
        );
    }

    #[test]
    fn test_multiple_modifiers() {
        assert_eq!(
            parse_key_event("ctrl-alt-a").unwrap(),
            KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::CONTROL | KeyModifiers::ALT
            )
        );

        #[cfg(target_os = "macos")]
        assert_eq!(
            parse_key_event("cmd-alt-a").unwrap(),
            KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::SUPER | KeyModifiers::ALT
            )
        );

        #[cfg(not(target_os = "macos"))]
        assert_eq!(
            parse_key_event("super-alt-a").unwrap(),
            KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::SUPER | KeyModifiers::ALT
            )
        );

        assert_eq!(
            parse_key_event("ctrl-shift-enter").unwrap(),
            KeyEvent::new(
                KeyCode::Enter,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )
        );
    }

    #[test]
    fn test_reverse_multiple_modifiers() {
        assert_eq!(
            key_event_to_string(&KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::CONTROL | KeyModifiers::ALT
            )),
            "ctrl-alt-a".to_string()
        );
    }

    #[test]
    fn test_invalid_keys() {
        assert!(parse_key_event("invalid-key").is_err());
        assert!(parse_key_event("ctrl-invalid-key").is_err());
    }

    #[test]
    fn test_case_insensitivity() {
        assert_eq!(
            parse_key_event("CTRL-a").unwrap(),
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL)
        );

        assert_eq!(
            parse_key_event("AlT-eNtEr").unwrap(),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT)
        );
    }

    #[test]
    fn test_deserialize_keybindings() {
        let keybindings: KeyBindings = toml::from_str(
            r#"
                "esc" = "quit"
                "ctrl-c" = "quit"
                "down" = "select_next_entry"
                "ctrl-n" = "select_next_entry"
                "ctrl-j" = "select_next_entry"
                "up" = "select_prev_entry"
                "ctrl-p" = "select_prev_entry"
                "ctrl-k" = "select_prev_entry"
                "pagedown" = "select_next_page"
                "pageup" = "select_prev_page"
                "ctrl-d" = "scroll_preview_half_page_down"
                "ctrl-u" = "scroll_preview_half_page_up"
                "tab" = "toggle_selection_down"
                "backtab" = "toggle_selection_up"
                "enter" = "confirm_selection"
                "ctrl-y" = "copy_entry_to_clipboard"
                "ctrl-r" = "toggle_remote_control"
                "ctrl-o" = "toggle_preview"
            "#,
        )
        .unwrap();

        assert_eq!(
            keybindings,
            KeyBindings::from(vec![
                (Key::Esc, Action::Quit),
                (Key::Ctrl('c'), Action::Quit),
                (Key::Down, Action::SelectNextEntry),
                (Key::Ctrl('n'), Action::SelectNextEntry),
                (Key::Ctrl('j'), Action::SelectNextEntry),
                (Key::Up, Action::SelectPrevEntry),
                (Key::Ctrl('p'), Action::SelectPrevEntry),
                (Key::Ctrl('k'), Action::SelectPrevEntry),
                (Key::PageDown, Action::SelectNextPage),
                (Key::PageUp, Action::SelectPrevPage),
                (Key::Ctrl('d'), Action::ScrollPreviewHalfPageDown),
                (Key::Ctrl('u'), Action::ScrollPreviewHalfPageUp),
                (Key::Tab, Action::ToggleSelectionDown),
                (Key::BackTab, Action::ToggleSelectionUp),
                (Key::Enter, Action::ConfirmSelection),
                (Key::Ctrl('y'), Action::CopyEntryToClipboard),
                (Key::Ctrl('r'), Action::ToggleRemoteControl),
                (Key::Ctrl('o'), Action::TogglePreview),
            ])
        );
    }

    #[test]
    fn test_merge_keybindings() {
        let base_keybindings = KeyBindings::from(vec![
            (Key::Esc, Action::Quit),
            (Key::Down, Action::SelectNextEntry),
            (Key::Ctrl('n'), Action::SelectNextEntry),
            (Key::Up, Action::SelectPrevEntry),
        ]);
        let custom_keybindings = KeyBindings::from(vec![
            (Key::Ctrl('j'), Action::SelectNextEntry),
            (Key::Ctrl('k'), Action::SelectPrevEntry),
            (Key::PageDown, Action::SelectNextPage),
        ]);

        let merged = merge_keybindings(base_keybindings, &custom_keybindings);

        // Should contain both base and custom keybindings
        assert!(merged.bindings.contains_key(&Key::Esc));
        assert_eq!(merged.bindings.get(&Key::Esc), Some(&Action::Quit));
        assert!(merged.bindings.contains_key(&Key::Down));
        assert_eq!(
            merged.bindings.get(&Key::Down),
            Some(&Action::SelectNextEntry)
        );
        assert!(merged.bindings.contains_key(&Key::Ctrl('j')));
        assert_eq!(
            merged.bindings.get(&Key::Ctrl('j')),
            Some(&Action::SelectNextEntry)
        );
        assert!(merged.bindings.contains_key(&Key::PageDown));
        assert_eq!(
            merged.bindings.get(&Key::PageDown),
            Some(&Action::SelectNextPage)
        );
    }

    #[test]
    fn test_deserialize_unbinding() {
        let keybindings: KeyBindings = toml::from_str(
            r#"
                "esc" = "quit"
                "ctrl-c" = false
                "down" = "select_next_entry"
                "up" = true
            "#,
        )
        .unwrap();

        // Normal action binding should work
        assert_eq!(keybindings.bindings.get(&Key::Esc), Some(&Action::Quit));
        assert_eq!(
            keybindings.bindings.get(&Key::Down),
            Some(&Action::SelectNextEntry)
        );

        // false should bind to NoOp (unbinding)
        assert_eq!(
            keybindings.bindings.get(&Key::Ctrl('c')),
            Some(&Action::NoOp)
        );

        // true should be ignored (no binding created)
        assert_eq!(keybindings.bindings.get(&Key::Up), None);
    }
}
