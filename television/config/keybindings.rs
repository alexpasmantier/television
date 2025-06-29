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

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
/// A set of keybindings for various actions in the application.
///
/// This struct is a wrapper around a `FxHashMap` that maps `Action`s to their corresponding
/// `Binding`s. It's main use is to provide a convenient way to manage and serialize/deserialize
/// keybindings from the configuration file as well as channel prototypes.
pub struct KeyBindings(pub FxHashMap<Action, Binding>);

impl<I> From<I> for KeyBindings
where
    I: IntoIterator<Item = (Action, Binding)>,
{
    fn from(iter: I) -> Self {
        KeyBindings(iter.into_iter().collect())
    }
}

impl Hash for KeyBindings {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // we're not actually using this for hashing, so this really only is a placeholder
        state.write_u8(0);
    }
}

impl Deref for KeyBindings {
    type Target = FxHashMap<Action, Binding>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KeyBindings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Merge two sets of keybindings together.
///
/// Note that this function won't "meld", for a given action, the bindings from the first set
/// with the bindings from the second set. Instead, it will simply overwrite them with the second
/// set's keys.
/// This is because it is assumed that the second set will be the user's custom keybindings, and
/// they should take precedence over the default ones, effectively replacing them to avoid
/// conflicts.
pub fn merge_keybindings(
    mut keybindings: KeyBindings,
    new_keybindings: &KeyBindings,
) -> KeyBindings {
    for (action, binding) in new_keybindings.iter() {
        keybindings.insert(action.clone(), binding.clone());
    }
    keybindings
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

    if key_event.modifiers.intersects(KeyModifiers::SUPER) {
        modifiers.push("cmd");
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

    // Handle mouse scroll keys as special cases
    match raw.to_ascii_lowercase().as_str() {
        "mousescrollup" => return Ok(Key::MouseScrollUp),
        "mousescrolldown" => return Ok(Key::MouseScrollDown),
        _ => {}
    }

    let key_event = parse_key_event(raw)?;
    Ok(convert_raw_event_to_key(key_event))
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

        assert_eq!(
            parse_key_event("cmd-alt-a").unwrap(),
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
                # Quit the application
                quit = ["esc", "ctrl-c"]
                # Scrolling through entries
                select_next_entry = ["down", "ctrl-n", "ctrl-j"]
                select_prev_entry = ["up", "ctrl-p", "ctrl-k"]
                select_next_page = "pagedown"
                select_prev_page = "pageup"
                # Scrolling the preview pane
                scroll_preview_half_page_down = "ctrl-d"
                scroll_preview_half_page_up = "ctrl-u"
                # Add entry to selection and move to the next entry
                toggle_selection_down = "tab"
                # Add entry to selection and move to the previous entry
                toggle_selection_up = "backtab"
                # Confirm selection
                confirm_selection = "enter"
                # Copy the selected entry to the clipboard
                copy_entry_to_clipboard = "ctrl-y"
                # Toggle the remote control mode
                toggle_remote_control = "ctrl-r"
                # Toggle the preview panel
                toggle_preview = "ctrl-o"
            "#,
        )
        .unwrap();

        assert_eq!(
            keybindings,
            KeyBindings::from(vec![
                (
                    Action::Quit,
                    Binding::MultipleKeys(vec![Key::Esc, Key::Ctrl('c'),])
                ),
                (
                    Action::SelectNextEntry,
                    Binding::MultipleKeys(vec![
                        Key::Down,
                        Key::Ctrl('n'),
                        Key::Ctrl('j'),
                    ])
                ),
                (
                    Action::SelectPrevEntry,
                    Binding::MultipleKeys(vec![
                        Key::Up,
                        Key::Ctrl('p'),
                        Key::Ctrl('k'),
                    ])
                ),
                (Action::SelectNextPage, Binding::SingleKey(Key::PageDown)),
                (Action::SelectPrevPage, Binding::SingleKey(Key::PageUp)),
                (
                    Action::ScrollPreviewHalfPageDown,
                    Binding::SingleKey(Key::Ctrl('d'))
                ),
                (
                    Action::ScrollPreviewHalfPageUp,
                    Binding::SingleKey(Key::Ctrl('u'))
                ),
                (Action::ToggleSelectionDown, Binding::SingleKey(Key::Tab)),
                (Action::ToggleSelectionUp, Binding::SingleKey(Key::BackTab)),
                (Action::ConfirmSelection, Binding::SingleKey(Key::Enter)),
                (
                    Action::CopyEntryToClipboard,
                    Binding::SingleKey(Key::Ctrl('y'))
                ),
                (
                    Action::ToggleRemoteControl,
                    Binding::SingleKey(Key::Ctrl('r'))
                ),
                (Action::TogglePreview, Binding::SingleKey(Key::Ctrl('o'))),
            ])
        );
    }

    #[test]
    fn test_merge_keybindings() {
        let base_keybindings = KeyBindings::from(vec![
            (Action::Quit, Binding::SingleKey(Key::Esc)),
            (
                Action::SelectNextEntry,
                Binding::MultipleKeys(vec![Key::Down, Key::Ctrl('n')]),
            ),
            (Action::SelectPrevEntry, Binding::SingleKey(Key::Up)),
        ]);
        let custom_keybindings = KeyBindings::from(vec![
            (Action::SelectNextEntry, Binding::SingleKey(Key::Ctrl('j'))),
            (
                Action::SelectPrevEntry,
                Binding::MultipleKeys(vec![Key::Up, Key::Ctrl('k')]),
            ),
            (Action::SelectNextPage, Binding::SingleKey(Key::PageDown)),
        ]);

        let merged = merge_keybindings(base_keybindings, &custom_keybindings);

        assert_eq!(
            merged,
            KeyBindings::from(vec![
                (Action::Quit, Binding::SingleKey(Key::Esc)),
                (Action::SelectNextEntry, Binding::SingleKey(Key::Ctrl('j'))),
                (
                    Action::SelectPrevEntry,
                    Binding::MultipleKeys(vec![Key::Up, Key::Ctrl('k')]),
                ),
                (Action::SelectNextPage, Binding::SingleKey(Key::PageDown)),
            ])
        );
    }
}
