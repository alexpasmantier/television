use crate::{
    action::{Action, Actions},
    event::{Key, convert_raw_event_to_key},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use tracing::{debug, trace};

/// Generic bindings structure that maps any key type to actions.
///
/// This is the core structure for storing key/event bindings in Television.
/// It provides a flexible mapping system that can work with different key types
/// (keyboard keys, mouse events, etc.) and supports serialization to/from TOML.
///
/// # Type Parameters
///
/// * `K` - The key type (must implement `Display`, `FromStr`, `Eq`, `Hash`)
///
/// # Examples
///
/// ```rust
/// use television::config::keybindings::{Bindings, KeyBindings};
/// use television::event::Key;
/// use television::action::Action;
///
/// // Create new empty bindings
/// let mut bindings = KeyBindings::new();
///
/// // Add a binding
/// bindings.insert(Key::Enter, Action::ConfirmSelection.into());
/// assert_eq!(bindings.len(), 1);
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bindings<K>
where
    K: Display + FromStr + Eq + Hash,
    K::Err: Display,
{
    #[serde(
        flatten,
        serialize_with = "serialize_bindings",
        deserialize_with = "deserialize_bindings"
    )]
    pub inner: FxHashMap<K, Actions>,
}

impl<K> Bindings<K>
where
    K: Display + FromStr + Eq + Hash,
    K::Err: Display,
{
    /// Creates a new empty bindings collection.
    ///
    /// # Returns
    ///
    /// A new `Bindings` instance with no key mappings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use television::config::keybindings::KeyBindings;
    ///
    /// let bindings = KeyBindings::new();
    /// assert!(bindings.is_empty());
    /// ```
    pub fn new() -> Self {
        Bindings {
            inner: FxHashMap::default(),
        }
    }
}

/// A set of keybindings that maps keyboard keys directly to actions.
///
/// This type alias represents the primary keybinding system in Television, where
/// keyboard keys are mapped directly to actions.
///
/// # Features
///
/// - Direct key-to-action mapping
/// - Support for single and multiple actions per key
/// - Key unbinding support (setting to `false`)
/// - Modifier key combinations (Ctrl, Alt, Shift, Super/Cmd)
///
/// # Configuration Format
///
/// ```toml
/// # Single action
/// esc = "quit"
/// enter = "confirm_selection"
///
/// # Multiple actions
/// "ctrl-s" = ["reload_source", "copy_entry_to_clipboard"]
///
/// # Unbind a key
/// "ctrl-c" = false
/// ```
///
/// # Examples
///
/// ```rust
/// use television::config::keybindings::KeyBindings;
/// use television::event::Key;
/// use television::action::Action;
///
/// let mut bindings = KeyBindings::new();
/// bindings.insert(Key::Enter, Action::ConfirmSelection.into());
/// bindings.insert(Key::Esc, Action::Quit.into());
/// assert_eq!(bindings.len(), 2);
/// ```
pub type KeyBindings = Bindings<Key>;

/// Types of events that can be bound to actions.
///
/// This enum defines the various non-keyboard events that can trigger actions
/// in Television, such as mouse events and terminal resize events.
///
/// # Variants
///
/// - `MouseClick` - Mouse button click events
/// - `MouseScrollUp` - Mouse wheel scroll up
/// - `MouseScrollDown` - Mouse wheel scroll down
/// - `Resize` - Terminal window resize events
/// - `Custom(String)` - Custom event types for extensibility
///
/// # Configuration
///
/// Events use kebab-case naming in TOML configuration:
///
/// ```toml
/// mouse-click = "confirm_selection"
/// mouse-scroll-up = "scroll_preview_up"
/// resize = "refresh_layout"
/// ```
///
/// # Examples
///
/// ```rust
/// use television::config::keybindings::EventType;
/// use std::str::FromStr;
///
/// let event = EventType::from_str("mouse-click").unwrap();
/// assert_eq!(event, EventType::MouseClick);
///
/// let custom = EventType::Custom("my-event".to_string());
/// assert_eq!(custom.to_string(), "my-event");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EventType {
    MouseClick,
    MouseScrollUp,
    MouseScrollDown,
    Resize,
    Custom(String),
}

impl FromStr for EventType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "mouse-click" => EventType::MouseClick,
            "mouse-scroll-up" => EventType::MouseScrollUp,
            "mouse-scroll-down" => EventType::MouseScrollDown,
            "resize" => EventType::Resize,
            custom => EventType::Custom(custom.to_string()),
        })
    }
}

impl<'de> serde::Deserialize<'de> for EventType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(EventType::from_str(&s).unwrap())
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

/// A set of event bindings that maps non-keyboard events to actions.
///
/// This type alias provides bindings for mouse events, resize events,
/// and other non-keyboard interactions. It uses the same underlying
/// `Bindings` structure as `KeyBindings` but for `EventType` instead of `Key`.
///
/// # Default Bindings
///
/// - `mouse-scroll-up` → `scroll_preview_up`
/// - `mouse-scroll-down` → `scroll_preview_down`
///
/// # Configuration Example
///
/// ```toml
/// [event-bindings]
/// mouse-click = "confirm_selection"
/// mouse-scroll-up = "scroll_preview_up"
/// resize = "refresh_layout"
/// ```
///
/// # Examples
///
/// ```rust
/// use television::config::keybindings::{EventBindings, EventType};
/// use television::action::Action;
///
/// let mut bindings = EventBindings::new();
/// bindings.insert(EventType::MouseClick, Action::ConfirmSelection.into());
/// assert_eq!(bindings.len(), 1);
/// ```
pub type EventBindings = Bindings<EventType>;

impl<K, I> From<I> for Bindings<K>
where
    K: Display + FromStr + Eq + Hash,
    K::Err: Display,
    I: IntoIterator<Item = (K, Action)>,
{
    fn from(iter: I) -> Self {
        Bindings {
            inner: iter
                .into_iter()
                .map(|(k, a)| (k, Actions::from(a)))
                .collect(),
        }
    }
}

impl<K> Hash for Bindings<K>
where
    K: Display + FromStr + Eq + Hash,
    K::Err: Display,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash based on the bindings map
        for (key, actions) in &self.inner {
            key.hash(state);
            actions.hash(state);
        }
    }
}

impl<K> Deref for Bindings<K>
where
    K: Display + FromStr + Eq + Hash,
    K::Err: Display,
{
    type Target = FxHashMap<K, Actions>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<K> DerefMut for Bindings<K>
where
    K: Display + FromStr + Eq + Hash,
    K::Err: Display,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Merges two binding collections, with new bindings taking precedence.
///
/// # Arguments
///
/// * `bindings` - The base bindings collection (will be consumed)
/// * `new_bindings` - The new bindings to merge in (higher precedence)
///
/// # Returns
///
/// A new `Bindings` collection with merged key mappings.
///
/// # Examples
///
/// ```rust
/// use television::config::keybindings::{KeyBindings, merge_bindings};
/// use television::event::Key;
/// use television::action::Action;
///
/// let base = KeyBindings::from(vec![
///     (Key::Enter, Action::ConfirmSelection),
///     (Key::Esc, Action::Quit),
/// ]);
///
/// let custom = KeyBindings::from(vec![
///     (Key::Esc, Action::NoOp), // Override quit with no-op
///     (Key::Tab, Action::ToggleSelectionDown), // Add new binding
/// ]);
///
/// let merged = merge_bindings(base, &custom);
/// assert_eq!(merged.get(&Key::Enter), Some(&Action::ConfirmSelection.into()));
/// assert_eq!(merged.get(&Key::Esc), Some(&Action::NoOp.into()));
/// assert_eq!(merged.get(&Key::Tab), Some(&Action::ToggleSelectionDown.into()));
/// ```
pub fn merge_bindings<K>(
    mut bindings: Bindings<K>,
    new_bindings: &Bindings<K>,
) -> Bindings<K>
where
    K: Display + FromStr + Clone + Eq + Hash + std::fmt::Debug,
    K::Err: Display,
{
    debug!("bindings before: {:?}", bindings.inner);

    // Merge new bindings - they take precedence over existing ones
    for (key, actions) in &new_bindings.inner {
        bindings.inner.insert(key.clone(), actions.clone());
    }

    debug!("bindings after: {:?}", bindings.inner);

    bindings
}

impl Default for Bindings<Key> {
    fn default() -> Self {
        let mut bindings = FxHashMap::default();

        // Quit actions
        bindings.insert(Key::Esc, Action::Quit.into());
        bindings.insert(Key::Ctrl('c'), Action::Quit.into());

        // Navigation
        bindings.insert(Key::Down, Action::SelectNextEntry.into());
        bindings.insert(Key::Ctrl('n'), Action::SelectNextEntry.into());
        bindings.insert(Key::Ctrl('j'), Action::SelectNextEntry.into());
        bindings.insert(Key::Up, Action::SelectPrevEntry.into());
        bindings.insert(Key::Ctrl('p'), Action::SelectPrevEntry.into());
        bindings.insert(Key::Ctrl('k'), Action::SelectPrevEntry.into());

        // History navigation
        bindings.insert(Key::CtrlUp, Action::SelectPrevHistory.into());
        bindings.insert(Key::CtrlDown, Action::SelectNextHistory.into());

        // Selection actions
        bindings.insert(Key::Enter, Action::ConfirmSelection.into());
        bindings.insert(Key::Tab, Action::ToggleSelectionDown.into());
        bindings.insert(Key::BackTab, Action::ToggleSelectionUp.into());

        // Preview actions
        bindings
            .insert(Key::PageDown, Action::ScrollPreviewHalfPageDown.into());
        bindings.insert(Key::PageUp, Action::ScrollPreviewHalfPageUp.into());

        // Clipboard and toggles
        bindings.insert(Key::Ctrl('y'), Action::CopyEntryToClipboard.into());
        bindings.insert(Key::Ctrl('r'), Action::ReloadSource.into());
        bindings.insert(Key::Ctrl('s'), Action::CycleSources.into());

        // UI Features
        bindings.insert(Key::Ctrl('t'), Action::ToggleRemoteControl.into());
        bindings.insert(Key::Ctrl('o'), Action::TogglePreview.into());
        bindings.insert(Key::Ctrl('h'), Action::ToggleHelp.into());
        bindings.insert(Key::F(12), Action::ToggleStatusBar.into());
        bindings.insert(Key::Ctrl('l'), Action::ToggleOrientation.into());

        // Input field actions
        bindings.insert(Key::Backspace, Action::DeletePrevChar.into());
        bindings.insert(Key::Ctrl('w'), Action::DeletePrevWord.into());
        bindings.insert(Key::Ctrl('u'), Action::DeleteLine.into());
        bindings.insert(Key::Delete, Action::DeleteNextChar.into());
        bindings.insert(Key::Left, Action::GoToPrevChar.into());
        bindings.insert(Key::Right, Action::GoToNextChar.into());
        bindings.insert(Key::Home, Action::GoToInputStart.into());
        bindings.insert(Key::Ctrl('a'), Action::GoToInputStart.into());
        bindings.insert(Key::End, Action::GoToInputEnd.into());
        bindings.insert(Key::Ctrl('e'), Action::GoToInputEnd.into());

        Bindings { inner: bindings }
    }
}

impl Default for Bindings<EventType> {
    fn default() -> Self {
        let mut bindings = FxHashMap::default();

        // Mouse events
        bindings
            .insert(EventType::MouseScrollUp, Action::ScrollPreviewUp.into());
        bindings.insert(
            EventType::MouseScrollDown,
            Action::ScrollPreviewDown.into(),
        );

        Bindings { inner: bindings }
    }
}

/// Parses a string representation of a key event into a `KeyEvent`.
///
/// This function converts human-readable key descriptions (like "ctrl-a", "alt-enter")
/// into crossterm `KeyEvent` structures.
///
/// # Arguments
///
/// * `raw` - String representation of the key (e.g., "ctrl-a", "shift-f1", "esc")
///
/// # Returns
///
/// * `Ok(KeyEvent)` - Successfully parsed key event
/// * `Err(String)` - Parse error with description
///
/// # Supported Modifiers
///
/// - `ctrl-` - Control key
/// - `alt-` - Alt key
/// - `shift-` - Shift key
/// - `cmd-` - Command key (macOS)
/// - `super-` - Super key (Linux/Windows)
///
/// # Examples
///
/// ```rust
/// use television::config::keybindings::parse_key_event;
/// use crossterm::event::{KeyCode, KeyModifiers};
///
/// let event = parse_key_event("ctrl-a").unwrap();
/// assert_eq!(event.code, KeyCode::Char('a'));
/// assert_eq!(event.modifiers, KeyModifiers::CONTROL);
///
/// let event = parse_key_event("alt-enter").unwrap();
/// assert_eq!(event.code, KeyCode::Enter);
/// assert_eq!(event.modifiers, KeyModifiers::ALT);
/// ```
pub fn parse_key_event(raw: &str) -> anyhow::Result<KeyEvent, String> {
    let raw_lower = raw.to_ascii_lowercase();
    let (remaining, modifiers) = extract_modifiers(&raw_lower);
    parse_key_code_with_modifiers(remaining, modifiers)
}

/// Extracts modifier keys from a raw key string.
///
/// This helper function parses modifier prefixes (ctrl-, alt-, shift-, etc.)
/// from a key string and returns the remaining key part along with the
/// extracted modifiers as a `KeyModifiers` bitfield.
///
/// # Arguments
///
/// * `raw` - The raw key string (already lowercased)
///
/// # Returns
///
/// A tuple of (`remaining_key_string`, `extracted_modifiers`)
///
/// # Examples
///
/// ```ignore
/// let (key, mods) = extract_modifiers("ctrl-alt-a");
/// assert_eq!(key, "a");
/// assert!(mods.contains(KeyModifiers::CONTROL | KeyModifiers::ALT));
/// ```
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

/// Parses a key code string with pre-extracted modifiers into a `KeyEvent`.
///
/// This function handles the actual key code parsing after modifiers have
/// been extracted. It supports named keys (like "esc", "enter") and
/// single character keys.
///
/// # Arguments
///
/// * `raw` - The key string with modifiers already removed
/// * `modifiers` - Pre-extracted modifier keys
///
/// # Returns
///
/// * `Ok(KeyEvent)` - Successfully parsed key event
/// * `Err(String)` - Parse error for unrecognized keys
///
/// # Supported Keys
///
/// - Named keys: esc, enter, left, right, up, down, home, end, etc.
/// - Function keys: f1-f12
/// - Special keys: space, tab, backspace, delete, etc.
/// - Single characters: a-z, 0-9, punctuation
fn parse_key_code_with_modifiers(
    raw: &str,
    mut modifiers: KeyModifiers,
) -> anyhow::Result<KeyEvent, String> {
    use rustc_hash::FxHashMap;
    use std::sync::LazyLock;

    static KEY_CODE_MAP: LazyLock<FxHashMap<&'static str, KeyCode>> =
        LazyLock::new(|| {
            [
                ("esc", KeyCode::Esc),
                ("enter", KeyCode::Enter),
                ("left", KeyCode::Left),
                ("right", KeyCode::Right),
                ("up", KeyCode::Up),
                ("down", KeyCode::Down),
                ("home", KeyCode::Home),
                ("end", KeyCode::End),
                ("pageup", KeyCode::PageUp),
                ("pagedown", KeyCode::PageDown),
                ("backspace", KeyCode::Backspace),
                ("delete", KeyCode::Delete),
                ("insert", KeyCode::Insert),
                ("f1", KeyCode::F(1)),
                ("f2", KeyCode::F(2)),
                ("f3", KeyCode::F(3)),
                ("f4", KeyCode::F(4)),
                ("f5", KeyCode::F(5)),
                ("f6", KeyCode::F(6)),
                ("f7", KeyCode::F(7)),
                ("f8", KeyCode::F(8)),
                ("f9", KeyCode::F(9)),
                ("f10", KeyCode::F(10)),
                ("f11", KeyCode::F(11)),
                ("f12", KeyCode::F(12)),
                ("space", KeyCode::Char(' ')),
                (" ", KeyCode::Char(' ')),
                ("hyphen", KeyCode::Char('-')),
                ("minus", KeyCode::Char('-')),
                ("tab", KeyCode::Tab),
            ]
            .into_iter()
            .collect()
        });

    let c = if let Some(&key_code) = KEY_CODE_MAP.get(raw) {
        key_code
    } else if raw == "backtab" {
        modifiers.insert(KeyModifiers::SHIFT);
        KeyCode::BackTab
    } else if raw.len() == 1 {
        let mut c = raw.chars().next().unwrap();
        if modifiers.contains(KeyModifiers::SHIFT) {
            c = c.to_ascii_uppercase();
        }
        KeyCode::Char(c)
    } else {
        return Err(format!("Unable to parse {raw}"));
    };

    Ok(KeyEvent::new(c, modifiers))
}

/// Converts a `KeyEvent` back to its string representation.
///
/// This function performs the reverse operation of `parse_key_event`,
/// converting a crossterm `KeyEvent` back into a human-readable string
/// format that can be used in configuration files.
///
/// # Arguments
///
/// * `key_event` - The key event to convert
///
/// # Returns
///
/// String representation of the key event (e.g., "ctrl-a", "alt-enter")
///
/// # Examples
///
/// ```rust
/// use television::config::keybindings::key_event_to_string;
/// use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
///
/// let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
/// assert_eq!(key_event_to_string(&event), "ctrl-a");
///
/// let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT);
/// assert_eq!(key_event_to_string(&event), "alt-enter");
/// ```
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

impl FromStr for Key {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
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
}

/// Generic serializer that converts any key type to string for TOML compatibility.
///
/// This function enables serialization of the bindings `HashMap` to TOML format
/// by converting keys to their string representation using the `Display` trait.
/// This is used internally by serde when serializing `Bindings` structs.
///
/// # Arguments
///
/// * `bindings` - The bindings `HashMap` to serialize
/// * `serializer` - The serde serializer instance
///
/// # Type Parameters
///
/// * `K` - Key type that implements `Display`
/// * `S` - Serializer type
///
/// # Returns
///
/// Result of the serialization operation
fn serialize_bindings<K, S>(
    bindings: &FxHashMap<K, Actions>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    K: Display,
    S: serde::Serializer,
{
    use serde::ser::SerializeMap;
    let mut map = serializer.serialize_map(Some(bindings.len()))?;
    for (key, actions) in bindings {
        map.serialize_entry(&key.to_string(), actions)?;
    }
    map.end()
}

/// Generic deserializer that parses string keys back to key enum.
///
/// This function enables deserialization from TOML format by parsing string keys
/// back into their appropriate key types using the `FromStr` trait. It also handles
/// special cases like boolean values for key unbinding.
///
/// # Special Value Handling
///
/// - `false` - Binds the key to `Action::NoOp` (effectively unbinding)
/// - `true` - Ignores the binding (preserves existing or default binding)
/// - String/Array - Normal action binding
///
/// # Arguments
///
/// * `deserializer` - The serde deserializer instance
///
/// # Type Parameters
///
/// * `K` - Key type that implements `FromStr`, `Eq`, and `Hash`
/// * `D` - Deserializer type
///
/// # Returns
///
/// Result containing the parsed bindings `HashMap` or deserialization error
fn deserialize_bindings<'de, K, D>(
    deserializer: D,
) -> Result<FxHashMap<K, Actions>, D::Error>
where
    K: FromStr + Eq + std::hash::Hash,
    K::Err: std::fmt::Display,
    D: serde::Deserializer<'de>,
{
    use serde::de::{MapAccess, Visitor};
    use std::fmt;

    struct BindingsVisitor<K>(std::marker::PhantomData<K>);

    impl<'de, K> Visitor<'de> for BindingsVisitor<K>
    where
        K: FromStr + Eq + std::hash::Hash,
        K::Err: std::fmt::Display,
    {
        type Value = FxHashMap<K, Actions>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter
                .write_str("a map with string keys and action/actions values")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            use serde::de::Error;
            use toml::Value;

            debug!("Starting deserialize_bindings for configuration");
            let mut bindings = FxHashMap::default();
            while let Some((key_str, raw_value)) =
                map.next_entry::<String, Value>()?
            {
                trace!(
                    "Processing binding: key='{}', value={:?}",
                    key_str, raw_value
                );
                let key = K::from_str(&key_str).map_err(|e| {
                    debug!("Failed to parse key '{}': {}", key_str, e);
                    Error::custom(e)
                })?;

                match raw_value {
                    Value::Boolean(false) => {
                        debug!("Unbinding key '{}' (set to NoOp)", key_str);
                        bindings.insert(key, Action::NoOp.into());
                    }
                    Value::Boolean(true) => {
                        trace!("Ignoring key '{}' (set to true)", key_str);
                    }
                    actions_value => {
                        // Try to deserialize as Actions (handles both single and multiple)
                        let actions = Actions::deserialize(actions_value)
                            .map_err(|e| {
                                debug!("Failed to deserialize actions for key '{}': {}", key_str, e);
                                Error::custom(e)
                            })?;
                        trace!(
                            "Binding key '{}' to actions: {:?}",
                            key_str, actions
                        );
                        bindings.insert(key, actions);
                    }
                }
            }
            debug!("Deserialized {} key bindings", bindings.len());
            Ok(bindings)
        }
    }

    deserializer.deserialize_map(BindingsVisitor(std::marker::PhantomData))
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

        let merged = merge_bindings(base_keybindings, &custom_keybindings);

        // Should contain both base and custom keybindings
        assert!(merged.inner.contains_key(&Key::Esc));
        assert_eq!(merged.inner.get(&Key::Esc), Some(&Action::Quit.into()));
        assert!(merged.inner.contains_key(&Key::Down));
        assert_eq!(
            merged.inner.get(&Key::Down),
            Some(&Action::SelectNextEntry.into())
        );
        assert!(merged.inner.contains_key(&Key::Ctrl('j')));
        assert_eq!(
            merged.inner.get(&Key::Ctrl('j')),
            Some(&Action::SelectNextEntry.into())
        );
        assert!(merged.inner.contains_key(&Key::PageDown));
        assert_eq!(
            merged.inner.get(&Key::PageDown),
            Some(&Action::SelectNextPage.into())
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
        assert_eq!(
            keybindings.inner.get(&Key::Esc),
            Some(&Action::Quit.into())
        );
        assert_eq!(
            keybindings.inner.get(&Key::Down),
            Some(&Action::SelectNextEntry.into())
        );

        // false should bind to NoOp (unbinding)
        assert_eq!(
            keybindings.inner.get(&Key::Ctrl('c')),
            Some(&Action::NoOp.into())
        );

        // true should be ignored (no binding created)
        assert_eq!(keybindings.inner.get(&Key::Up), None);
    }

    #[test]
    fn test_deserialize_multiple_actions_per_key() {
        let keybindings: KeyBindings = toml::from_str(
            r#"
                "esc" = "quit"
                "ctrl-s" = ["reload_source", "copy_entry_to_clipboard"]
                "f1" = ["toggle_help", "toggle_preview", "toggle_status_bar"]
            "#,
        )
        .unwrap();

        // Single action should work
        assert_eq!(
            keybindings.inner.get(&Key::Esc),
            Some(&Action::Quit.into())
        );

        // Multiple actions should work
        assert_eq!(
            keybindings.inner.get(&Key::Ctrl('s')),
            Some(&Actions::multiple(vec![
                Action::ReloadSource,
                Action::CopyEntryToClipboard
            ]))
        );

        // Three actions should work
        assert_eq!(
            keybindings.inner.get(&Key::F(1)),
            Some(&Actions::multiple(vec![
                Action::ToggleHelp,
                Action::TogglePreview,
                Action::ToggleStatusBar
            ]))
        );
    }

    #[test]
    fn test_merge_keybindings_with_multiple_actions() {
        let base_keybindings = KeyBindings::from(vec![
            (Key::Esc, Action::Quit),
            (Key::Enter, Action::ConfirmSelection),
        ]);

        let mut custom_bindings = FxHashMap::default();
        custom_bindings.insert(
            Key::Ctrl('s'),
            Actions::multiple(vec![
                Action::ReloadSource,
                Action::CopyEntryToClipboard,
            ]),
        );
        custom_bindings.insert(Key::Esc, Action::NoOp.into()); // Override
        let custom_keybindings = KeyBindings {
            inner: custom_bindings,
        };

        let merged = merge_bindings(base_keybindings, &custom_keybindings);

        // Custom multiple actions should be present
        assert_eq!(
            merged.inner.get(&Key::Ctrl('s')),
            Some(&Actions::multiple(vec![
                Action::ReloadSource,
                Action::CopyEntryToClipboard
            ]))
        );

        // Override should work
        assert_eq!(merged.inner.get(&Key::Esc), Some(&Action::NoOp.into()));

        // Original binding should be preserved
        assert_eq!(
            merged.inner.get(&Key::Enter),
            Some(&Action::ConfirmSelection.into())
        );
    }

    #[test]
    fn test_complex_configuration_with_all_features() {
        let keybindings: KeyBindings = toml::from_str(
            r#"
                # Single actions
                esc = "quit"
                enter = "confirm_selection"

                # Multiple actions
                ctrl-s = ["reload_source", "copy_entry_to_clipboard"]
                f1 = ["toggle_help", "toggle_preview", "toggle_status_bar"]

                # Unbinding
                ctrl-c = false

                # Single action in array format (should work)
                tab = ["toggle_selection_down"]
            "#,
        )
        .unwrap();

        assert_eq!(keybindings.inner.len(), 6); // ctrl-c=false creates NoOp binding

        // Verify all binding types work correctly
        assert_eq!(
            keybindings.inner.get(&Key::Esc),
            Some(&Actions::single(Action::Quit))
        );
        assert_eq!(
            keybindings.inner.get(&Key::Enter),
            Some(&Action::ConfirmSelection.into())
        );
        assert_eq!(
            keybindings.inner.get(&Key::Ctrl('s')),
            Some(&Actions::multiple(vec![
                Action::ReloadSource,
                Action::CopyEntryToClipboard
            ]))
        );
        assert_eq!(
            keybindings.inner.get(&Key::F(1)),
            Some(&Actions::multiple(vec![
                Action::ToggleHelp,
                Action::TogglePreview,
                Action::ToggleStatusBar
            ]))
        );
        assert_eq!(
            keybindings.inner.get(&Key::Ctrl('c')),
            Some(&Actions::single(Action::NoOp))
        );
        assert_eq!(
            keybindings.inner.get(&Key::Tab),
            Some(&Actions::multiple(vec![Action::ToggleSelectionDown]))
        );
    }
}
