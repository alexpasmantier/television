use crate::action::Action;
use crate::event::{convert_raw_event_to_key, Key};
use crate::screen::mode::Mode;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Deserializer};
use std::fmt::Display;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Default)]
pub struct KeyBindings(pub FxHashMap<Mode, FxHashMap<Action, Binding>>);

impl Deref for KeyBindings {
    type Target = FxHashMap<Mode, FxHashMap<Action, Binding>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KeyBindings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn merge_keybindings(
    mut keybindings: KeyBindings,
    new_keybindings: &KeyBindings,
) -> KeyBindings {
    for (mode, bindings) in new_keybindings.iter() {
        for (action, binding) in bindings {
            match keybindings.get_mut(mode) {
                Some(mode_bindings) => {
                    mode_bindings.insert(action.clone(), binding.clone());
                }
                None => {
                    keybindings.insert(
                        *mode,
                        [(action.clone(), binding.clone())]
                            .iter()
                            .cloned()
                            .collect(),
                    );
                }
            }
        }
    }
    keybindings
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum SerializedBinding {
    SingleKey(String),
    MultipleKeys(Vec<String>),
}

impl<'de> Deserialize<'de> for KeyBindings {
    fn deserialize<D>(deserializer: D) -> color_eyre::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let parsed_map = FxHashMap::<
            Mode,
            FxHashMap<Action, SerializedBinding>,
        >::deserialize(deserializer)?;

        let keybindings: FxHashMap<Mode, FxHashMap<Action, Binding>> =
            parsed_map
                .into_iter()
                .map(|(mode, inner_map)| {
                    let converted_inner_map = inner_map
                        .into_iter()
                        .map(|(cmd, binding)| {
                            (
                                cmd,
                                match binding {
                                    SerializedBinding::SingleKey(key_str) => {
                                        Binding::SingleKey(
                                            parse_key(&key_str).unwrap(),
                                        )
                                    }
                                    SerializedBinding::MultipleKeys(
                                        keys_str,
                                    ) => Binding::MultipleKeys(
                                        keys_str
                                            .iter()
                                            .map(|key_str| {
                                                parse_key(key_str).unwrap()
                                            })
                                            .collect(),
                                    ),
                                },
                            )
                        })
                        .collect();
                    (mode, converted_inner_map)
                })
                .collect();

        Ok(KeyBindings(keybindings))
    }
}

pub fn parse_key_event(raw: &str) -> color_eyre::Result<KeyEvent, String> {
    let raw_lower = raw.to_ascii_lowercase();
    let (remaining, modifiers) = extract_modifiers(&raw_lower);
    parse_key_code_with_modifiers(remaining, modifiers)
}

fn extract_modifiers(raw: &str) -> (&str, KeyModifiers) {
    let mut modifiers = KeyModifiers::empty();
    let mut current = raw;

    loop {
        match current {
            rest if rest.starts_with("ctrl-") => {
                modifiers.insert(KeyModifiers::CONTROL);
                current = &rest[5..];
            }
            rest if rest.starts_with("alt-") => {
                modifiers.insert(KeyModifiers::ALT);
                current = &rest[4..];
            }
            rest if rest.starts_with("shift-") => {
                modifiers.insert(KeyModifiers::SHIFT);
                current = &rest[6..];
            }
            _ => break, // break out of the loop if no known prefix is detected
        };
    }

    (current, modifiers)
}

fn parse_key_code_with_modifiers(
    raw: &str,
    mut modifiers: KeyModifiers,
) -> color_eyre::Result<KeyEvent, String> {
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
        "space" => KeyCode::Char(' '),
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

pub fn parse_key(raw: &str) -> color_eyre::Result<Key, String> {
    if raw.chars().filter(|c| *c == '>').count()
        != raw.chars().filter(|c| *c == '<').count()
    {
        return Err(format!("Unable to parse `{raw}`"));
    }
    let raw = if raw.contains("><") {
        raw
    } else {
        let raw = raw.strip_prefix('<').unwrap_or(raw);
        let raw = raw.strip_suffix('>').unwrap_or(raw);
        raw
    };
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
}
