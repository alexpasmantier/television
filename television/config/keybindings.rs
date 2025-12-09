use crate::{
    action::{Action, Actions},
    event::{Key, convert_raw_event_to_key},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rustc_hash::FxHashMap;
use serde::Deserialize;
use std::ops::Deref;
use std::ops::DerefMut;
use std::str::FromStr;
use tracing::debug;

/// A hashmap of keyboard key bindings to actions.
#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Keybindings(pub FxHashMap<Key, Actions>);

impl Deref for Keybindings {
    type Target = FxHashMap<Key, Actions>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Keybindings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<(Key, Action)>> for Keybindings {
    fn from(bindings: Vec<(Key, Action)>) -> Self {
        let mut map = FxHashMap::default();
        for (key, action) in bindings {
            map.insert(key, action.into());
        }
        Keybindings(map)
    }
}

impl Keybindings {
    pub fn new() -> Self {
        Keybindings(FxHashMap::default())
    }

    pub fn merge(self, new: &Keybindings) -> Keybindings {
        merge_keybindings(self, new)
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
/// use television::config::keybindings::{Keybindings, merge_keybindings};
/// use television::event::Key;
/// use television::action::Action;
///
/// let base = Keybindings::from(vec![
///     (Key::Enter, Action::ConfirmSelection),
///     (Key::Esc, Action::Quit),
/// ]);
///
/// let custom = Keybindings::from(vec![
///     (Key::Esc, Action::NoOp), // Override quit with no-op
///     (Key::Tab, Action::ToggleSelectionDown), // Add new binding
/// ]);
///
/// let merged = merge_keybindings(base, &custom);
/// assert_eq!(merged.get(&Key::Enter), Some(&Action::ConfirmSelection.into()));
/// assert_eq!(merged.get(&Key::Esc), Some(&Action::NoOp.into()));
/// assert_eq!(merged.get(&Key::Tab), Some(&Action::ToggleSelectionDown.into()));
/// ```
pub fn merge_keybindings(
    mut base: Keybindings,
    new: &Keybindings,
) -> Keybindings {
    debug!("bindings before: {:?}", base.0);

    // Merge new bindings - they take precedence over existing ones
    for (key, actions) in &new.0 {
        base.0.insert(*key, actions.clone());
    }

    debug!("bindings after: {:?}", base.0);

    base
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
        let keybindings: Keybindings = toml::from_str(
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
            Keybindings::from(vec![
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
        let base = Keybindings::from(vec![
            (Key::Esc, Action::Quit),
            (Key::Down, Action::SelectNextEntry),
            (Key::Ctrl('n'), Action::SelectNextEntry),
            (Key::Up, Action::SelectPrevEntry),
        ]);
        let new = Keybindings::from(vec![
            (Key::Ctrl('j'), Action::SelectNextEntry),
            (Key::Ctrl('k'), Action::SelectPrevEntry),
            (Key::PageDown, Action::SelectNextPage),
        ]);

        let merged = merge_keybindings(base, &new);

        // Should contain both base and custom keybindings
        assert!(merged.0.contains_key(&Key::Esc));
        assert_eq!(merged.0.get(&Key::Esc), Some(&Action::Quit.into()));
        assert!(merged.0.contains_key(&Key::Down));
        assert_eq!(
            merged.0.get(&Key::Down),
            Some(&Action::SelectNextEntry.into())
        );
        assert!(merged.0.contains_key(&Key::Ctrl('j')));
        assert_eq!(
            merged.0.get(&Key::Ctrl('j')),
            Some(&Action::SelectNextEntry.into())
        );
        assert!(merged.0.contains_key(&Key::PageDown));
        assert_eq!(
            merged.0.get(&Key::PageDown),
            Some(&Action::SelectNextPage.into())
        );
    }

    #[test]
    fn test_deserialize_unbinding() {
        let keybindings: Keybindings = toml::from_str(
            r#"
                "esc" = "quit"
                "ctrl-c" = "no_op"
                "down" = "select_next_entry"
            "#,
        )
        .unwrap();

        // Normal action binding should work
        assert_eq!(keybindings.0.get(&Key::Esc), Some(&Action::Quit.into()));
        assert_eq!(
            keybindings.0.get(&Key::Down),
            Some(&Action::SelectNextEntry.into())
        );

        // false should bind to NoOp (unbinding)
        assert_eq!(
            keybindings.0.get(&Key::Ctrl('c')),
            Some(&Action::NoOp.into())
        );
    }

    #[test]
    fn test_deserialize_multiple_actions_per_key() {
        let keybindings: Keybindings = toml::from_str(
            r#"
                "esc" = "quit"
                "ctrl-s" = ["reload_source", "copy_entry_to_clipboard"]
                "f1" = ["toggle_help", "toggle_preview", "toggle_status_bar"]
            "#,
        )
        .unwrap();

        // Single action should work
        assert_eq!(keybindings.0.get(&Key::Esc), Some(&Action::Quit.into()));

        // Multiple actions should work
        assert_eq!(
            keybindings.0.get(&Key::Ctrl('s')),
            Some(&Actions::multiple(vec![
                Action::ReloadSource,
                Action::CopyEntryToClipboard
            ]))
        );

        // Three actions should work
        assert_eq!(
            keybindings.0.get(&Key::F(1)),
            Some(&Actions::multiple(vec![
                Action::ToggleHelp,
                Action::TogglePreview,
                Action::ToggleStatusBar
            ]))
        );
    }

    #[test]
    fn test_merge_keybindings_with_multiple_actions() {
        let base_keybindings = Keybindings::from(vec![
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
        let custom_keybindings = Keybindings(custom_bindings);

        let merged = merge_keybindings(base_keybindings, &custom_keybindings);

        // Custom multiple actions should be present
        assert_eq!(
            merged.0.get(&Key::Ctrl('s')),
            Some(&Actions::multiple(vec![
                Action::ReloadSource,
                Action::CopyEntryToClipboard
            ]))
        );

        // Override should work
        assert_eq!(merged.0.get(&Key::Esc), Some(&Action::NoOp.into()));

        // Original binding should be preserved
        assert_eq!(
            merged.0.get(&Key::Enter),
            Some(&Action::ConfirmSelection.into())
        );
    }

    #[test]
    fn test_complex_configuration_with_all_features() {
        let keybindings: Keybindings = toml::from_str(
            r#"
                # Single actions
                esc = "quit"
                enter = "confirm_selection"

                # Multiple actions
                ctrl-s = ["reload_source", "copy_entry_to_clipboard"]
                f1 = ["toggle_help", "toggle_preview", "toggle_status_bar"]

                # Unbinding
                ctrl-c = "no_op"

                # Single action in array format (should work)
                tab = ["toggle_selection_down"]
            "#,
        )
        .unwrap();

        assert_eq!(keybindings.0.len(), 6);

        // Verify all binding types work correctly
        assert_eq!(
            keybindings.0.get(&Key::Esc),
            Some(&Actions::single(Action::Quit))
        );
        assert_eq!(
            keybindings.0.get(&Key::Enter),
            Some(&Action::ConfirmSelection.into())
        );
        assert_eq!(
            keybindings.0.get(&Key::Ctrl('s')),
            Some(&Actions::multiple(vec![
                Action::ReloadSource,
                Action::CopyEntryToClipboard
            ]))
        );
        assert_eq!(
            keybindings.0.get(&Key::F(1)),
            Some(&Actions::multiple(vec![
                Action::ToggleHelp,
                Action::TogglePreview,
                Action::ToggleStatusBar
            ]))
        );
        assert_eq!(
            keybindings.0.get(&Key::Ctrl('c')),
            Some(&Actions::single(Action::NoOp))
        );
        assert_eq!(
            keybindings.0.get(&Key::Tab),
            Some(&Actions::multiple(vec![Action::ToggleSelectionDown]))
        );
    }
}
