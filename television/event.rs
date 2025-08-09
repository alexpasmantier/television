use crossterm::event::{
    KeyCode::{
        BackTab, Backspace, Char, Delete, Down, End, Enter, Esc, F, Home,
        Insert, Left, PageDown, PageUp, Right, Tab, Up,
    },
    KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    future::Future,
    pin::Pin,
    str::FromStr,
    task::{Context, Poll as TaskPoll},
    time::Duration,
};
use tokio::{signal, sync::mpsc};
use tracing::{debug, trace, warn};

#[derive(Debug, Clone, Copy)]
pub enum Event<I> {
    Closed,
    Input(I),
    Mouse(MouseEvent),
    FocusLost,
    FocusGained,
    Resize(u16, u16),
    Tick,
}

#[derive(
    Debug, Clone, Copy, Serialize, PartialEq, PartialOrd, Eq, Hash, Ord,
)]
pub enum Key {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    CtrlSpace,
    CtrlBackspace,
    CtrlEnter,
    CtrlLeft,
    CtrlRight,
    CtrlUp,
    CtrlDown,
    CtrlDelete,
    AltSpace,
    AltEnter,
    AltBackspace,
    AltDelete,
    AltUp,
    AltDown,
    AltLeft,
    AltRight,
    Home,
    End,
    PageUp,
    PageDown,
    BackTab,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Alt(char),
    Ctrl(char),
    Null,
    Esc,
    Tab,
}

/// Unified input event type that encompasses all possible inputs.
///
/// This enum provides a unified interface for handling different types of input
/// events in Television, including keyboard input, mouse events, terminal resize
/// events, and custom events. It enables the new binding system to map any
/// type of input to actions.
///
/// # Variants
///
/// - `Key(Key)` - Keyboard input events
/// - `Mouse(MouseInputEvent)` - Mouse events with position information
/// - `Resize(u16, u16)` - Terminal resize events with new dimensions
/// - `Custom(String)` - Custom events for extensibility
///
/// # Usage in Bindings
///
/// ```rust
/// use television::event::{InputEvent, Key, MouseInputEvent};
/// use television::keymap::InputMap;
///
/// let input_map = InputMap::default();
///
/// // Handle keyboard input
/// let key_event = InputEvent::Key(Key::Enter);
/// let actions = input_map.get_actions_for_input(&key_event);
/// assert_eq!(actions, None); // No bindings in empty map
///
/// // Handle mouse input
/// let mouse_event = InputEvent::Mouse(MouseInputEvent {
///     kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
///     position: (10, 5),
/// });
/// let actions = input_map.get_actions_for_input(&mouse_event);
/// assert_eq!(actions, None); // No bindings in empty map
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputEvent {
    /// Keyboard input event
    Key(Key),
    /// Mouse event with position information
    Mouse(MouseInputEvent),
    /// Terminal resize event with new dimensions (width, height)
    Resize(u16, u16),
    /// Custom event for extensibility
    Custom(String),
}

/// Mouse event with position information for input mapping.
///
/// This structure combines a mouse event type with its screen coordinates,
/// enabling position-aware mouse handling in the binding system. It provides
/// the information needed to map mouse events to appropriate actions.
///
/// # Fields
///
/// - `kind` - The type of mouse event (click, scroll, etc.)
/// - `position` - Screen coordinates as (column, row) tuple
///
/// # Examples
///
/// ```rust
/// use television::event::MouseInputEvent;
/// use crossterm::event::{MouseEventKind, MouseButton};
///
/// // Left mouse button click at position (10, 5)
/// let click_event = MouseInputEvent {
///     kind: MouseEventKind::Down(MouseButton::Left),
///     position: (10, 5),
/// };
/// assert_eq!(click_event.position, (10, 5));
///
/// // Mouse scroll up at position (20, 15)
/// let scroll_event = MouseInputEvent {
///     kind: MouseEventKind::ScrollUp,
///     position: (20, 15),
/// };
/// assert_eq!(scroll_event.position, (20, 15));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MouseInputEvent {
    /// The type of mouse event (click, scroll, etc.)
    pub kind: MouseEventKind,
    /// Screen coordinates as (column, row)
    pub position: (u16, u16),
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Key::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Key::Backspace => write!(f, "backspace"),
            Key::Enter => write!(f, "enter"),
            Key::Left => write!(f, "left"),
            Key::Right => write!(f, "right"),
            Key::Up => write!(f, "up"),
            Key::Down => write!(f, "down"),
            Key::CtrlSpace => write!(f, "ctrl-space"),
            Key::CtrlBackspace => write!(f, "ctrl-backspace"),
            Key::CtrlEnter => write!(f, "ctrl-enter"),
            Key::CtrlLeft => write!(f, "ctrl-left"),
            Key::CtrlRight => write!(f, "ctrl-right"),
            Key::CtrlUp => write!(f, "ctrl-up"),
            Key::CtrlDown => write!(f, "ctrl-down"),
            Key::CtrlDelete => write!(f, "ctrl-del"),
            Key::AltSpace => write!(f, "alt-space"),
            Key::AltEnter => write!(f, "alt-enter"),
            Key::AltBackspace => write!(f, "alt-backspace"),
            Key::AltDelete => write!(f, "alt-delete"),
            Key::AltUp => write!(f, "alt-up"),
            Key::AltDown => write!(f, "alt-down"),
            Key::AltLeft => write!(f, "alt-left"),
            Key::AltRight => write!(f, "alt-right"),
            Key::Home => write!(f, "home"),
            Key::End => write!(f, "end"),
            Key::PageUp => write!(f, "pageup"),
            Key::PageDown => write!(f, "pagedown"),
            Key::BackTab => write!(f, "backtab"),
            Key::Delete => write!(f, "delete"),
            Key::Insert => write!(f, "insert"),
            Key::F(k) => write!(f, "f{k}"),
            Key::Char(c) => write!(f, "{c}"),
            Key::Alt(c) => write!(f, "alt-{c}"),
            Key::Ctrl(c) => write!(f, "ctrl-{c}"),
            Key::Null => write!(f, "null"),
            Key::Esc => write!(f, "esc"),
            Key::Tab => write!(f, "tab"),
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct EventLoop {
    pub rx: mpsc::UnboundedReceiver<Event<Key>>,
    pub control_tx: mpsc::UnboundedSender<ControlEvent>,
}

struct PollFuture {
    timeout: Duration,
}

impl Future for PollFuture {
    type Output = bool;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> TaskPoll<Self::Output> {
        // Polling crossterm::event::poll, which is a blocking call
        // Spawn it in a separate task, to avoid blocking async runtime
        match crossterm::event::poll(self.timeout) {
            Ok(true) => TaskPoll::Ready(true),
            Ok(false) => {
                // Register the task to be polled again after a delay to avoid busy-looping
                cx.waker().wake_by_ref();
                TaskPoll::Pending
            }
            Err(_) => TaskPoll::Ready(false),
        }
    }
}

async fn poll_event(timeout: Duration) -> bool {
    PollFuture { timeout }.await
}

fn flush_existing_events() {
    let mut counter = 0;
    while let Ok(true) = crossterm::event::poll(Duration::from_millis(0)) {
        if let Ok(crossterm::event::Event::Key(_)) = crossterm::event::read() {
            counter += 1;
        }
    }
    if counter > 0 {
        debug!("Flushed {} existing events", counter);
    }
}

pub enum ControlEvent {
    /// Abort the event loop
    Abort,
    /// Pause the event loop
    Pause,
    /// Resume the event loop
    Resume,
}

impl EventLoop {
    pub fn new(tick_rate: u64) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let tick_interval = Duration::from_secs_f64(1.0 / tick_rate as f64);

        let (control_tx, mut control_rx) = mpsc::unbounded_channel();

        flush_existing_events();

        tokio::spawn(async move {
            loop {
                let delay = tokio::time::sleep(tick_interval);
                let event_available = poll_event(tick_interval);

                tokio::select! {
                    // if we receive a message on the abort channel, stop the event loop
                    Some(control_event) = control_rx.recv() => {
                        match control_event {
                            ControlEvent::Abort => {
                                debug!("Received Abort control event");
                                tx.send(Event::Closed).unwrap_or_else(|_| warn!("Unable to send Closed event"));
                                tx.send(Event::Tick).unwrap_or_else(|_| warn!("Unable to send Tick event"));
                                break;
                            },
                            ControlEvent::Pause => {
                                debug!("Received Pause control event");
                                // Stop processing events until resumed
                                while let Some(event) = control_rx.recv().await {
                                    match event {
                                        ControlEvent::Resume => {
                                            debug!("Received Resume control event");
                                            // flush any leftover events
                                            flush_existing_events();
                                            break; // Exit pause loop
                                        },
                                        ControlEvent::Abort => {
                                            debug!("Received Abort control event during Pause");
                                            tx.send(Event::Closed).unwrap_or_else(|_| warn!("Unable to send Closed event"));
                                            tx.send(Event::Tick).unwrap_or_else(|_| warn!("Unable to send Tick event"));
                                            return;
                                        },
                                        ControlEvent::Pause => {}
                                    }
                                }
                            },
                            // these should always be captured by the pause loop
                            ControlEvent::Resume => {},
                        }
                    },
                    _ = signal::ctrl_c() => {
                        debug!("Received SIGINT");
                        tx.send(Event::Input(Key::Ctrl('c'))).unwrap_or_else(|_| warn!("Unable to send Ctrl-C event"));
                    },
                    // if `delay` completes, pass to the next event "frame"
                    () = delay => {
                        tx.send(Event::Tick).unwrap_or_else(|_| warn!("Unable to send Tick event"));
                    },
                    // if the receiver dropped the channel, stop the event loop
                    () = tx.closed() => break,
                    // if an event was received, process it
                    _ = event_available => {
                        let maybe_event = crossterm::event::read();
                        match maybe_event {
                            Ok(crossterm::event::Event::Key(key)) => {
                                let key = convert_raw_event_to_key(key);
                                tx.send(Event::Input(key)).unwrap_or_else(|_| warn!("Unable to send {:?} event", key));
                            },
                            Ok(crossterm::event::Event::Mouse(mouse)) => {
                                tx.send(Event::Mouse(mouse)).unwrap_or_else(|_| warn!("Unable to send Mouse event"));
                            },
                            Ok(crossterm::event::Event::FocusLost) => {
                                tx.send(Event::FocusLost).unwrap_or_else(|_| warn!("Unable to send FocusLost event"));
                            },
                            Ok(crossterm::event::Event::FocusGained) => {
                                tx.send(Event::FocusGained).unwrap_or_else(|_| warn!("Unable to send FocusGained event"));
                            },
                            Ok(crossterm::event::Event::Resize(x, y)) => {
                                let (_, (new_x, new_y)) = flush_resize_events((x, y));
                                tx.send(Event::Resize(new_x, new_y)).unwrap_or_else(|_| warn!("Unable to send Resize event"));
                            },
                            _ => {}
                        }
                    }
                }
            }
        });

        Self {
            //tx,
            rx,
            //tick_rate,
            control_tx,
        }
    }
}

// Resize events can occur in batches.
// With a simple loop they can be flushed.
// This function will keep the first and last resize event.
fn flush_resize_events(first_resize: (u16, u16)) -> ((u16, u16), (u16, u16)) {
    let mut last_resize = first_resize;
    while let Ok(true) = crossterm::event::poll(Duration::from_millis(50)) {
        if let Ok(crossterm::event::Event::Resize(x, y)) =
            crossterm::event::read()
        {
            last_resize = (x, y);
        }
    }

    (first_resize, last_resize)
}

/// Converts a crossterm `KeyEvent` into Television's internal `Key` representation.
///
/// This function handles the conversion from crossterm's key event format into
/// Television's simplified key representation, applying modifier key combinations
/// and filtering out key release events.
///
/// # Arguments
///
/// * `event` - The crossterm `KeyEvent` to convert
///
/// # Returns
///
/// The corresponding `Key` enum variant, or `Key::Null` for unsupported events
///
/// # Key Mapping
///
/// - Modifier combinations are mapped to specific variants (e.g., `Ctrl+a` → `Key::Ctrl('a')`)
/// - Key release events are ignored (return `Key::Null`)
/// - Special keys are mapped directly (e.g., `Enter` → `Key::Enter`)
/// - Function keys preserve their number (e.g., `F1` → `Key::F(1)`)
///
/// # Examples
///
/// ```rust
/// use television::event::{convert_raw_event_to_key, Key};
/// use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, KeyEventKind};
///
/// let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
/// assert_eq!(convert_raw_event_to_key(event), Key::Ctrl('a'));
///
/// let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
/// assert_eq!(convert_raw_event_to_key(event), Key::Enter);
/// ```
pub fn convert_raw_event_to_key(event: KeyEvent) -> Key {
    trace!("Raw event: {:?}", event);
    if event.kind == KeyEventKind::Release {
        return Key::Null;
    }
    match event.code {
        Backspace => match event.modifiers {
            KeyModifiers::CONTROL => Key::CtrlBackspace,
            KeyModifiers::ALT => Key::AltBackspace,
            _ => Key::Backspace,
        },
        Delete => match event.modifiers {
            KeyModifiers::CONTROL => Key::CtrlDelete,
            KeyModifiers::ALT => Key::AltDelete,
            _ => Key::Delete,
        },
        Enter => match event.modifiers {
            KeyModifiers::CONTROL => Key::CtrlEnter,
            KeyModifiers::ALT => Key::AltEnter,
            _ => Key::Enter,
        },
        Up => match event.modifiers {
            KeyModifiers::CONTROL => Key::CtrlUp,
            KeyModifiers::ALT => Key::AltUp,
            _ => Key::Up,
        },
        Down => match event.modifiers {
            KeyModifiers::CONTROL => Key::CtrlDown,
            KeyModifiers::ALT => Key::AltDown,
            _ => Key::Down,
        },
        Left => match event.modifiers {
            KeyModifiers::CONTROL => Key::CtrlLeft,
            KeyModifiers::ALT => Key::AltLeft,
            _ => Key::Left,
        },
        Right => match event.modifiers {
            KeyModifiers::CONTROL => Key::CtrlRight,
            KeyModifiers::ALT => Key::AltRight,
            _ => Key::Right,
        },
        Home => Key::Home,
        End => Key::End,
        PageUp => Key::PageUp,
        PageDown => Key::PageDown,
        Tab => Key::Tab,
        BackTab => Key::BackTab,
        Insert => Key::Insert,
        F(k) => Key::F(k),
        Esc => Key::Esc,
        Char(' ') => match event.modifiers {
            KeyModifiers::NONE | KeyModifiers::SHIFT => Key::Char(' '),
            KeyModifiers::CONTROL => Key::CtrlSpace,
            KeyModifiers::ALT => Key::AltSpace,
            _ => Key::Null,
        },
        Char(c) => match event.modifiers {
            KeyModifiers::NONE | KeyModifiers::SHIFT => Key::Char(c),
            KeyModifiers::CONTROL => Key::Ctrl(c),
            KeyModifiers::ALT => Key::Alt(c),
            _ => Key::Null,
        },
        _ => Key::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{
        KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
    };

    #[test]
    fn test_convert_raw_event_to_key() {
        // character keys
        let event = KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert_eq!(convert_raw_event_to_key(event), Key::Char('a'));

        let event = KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert_eq!(convert_raw_event_to_key(event), Key::Ctrl('a'));

        let event = KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert_eq!(convert_raw_event_to_key(event), Key::Alt('a'));

        let event = KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert_eq!(convert_raw_event_to_key(event), Key::Char('a'));

        let event = KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert_eq!(convert_raw_event_to_key(event), Key::Char(' '));

        let event = KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert_eq!(convert_raw_event_to_key(event), Key::CtrlSpace);

        let event = KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert_eq!(convert_raw_event_to_key(event), Key::AltSpace);

        let event = KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert_eq!(convert_raw_event_to_key(event), Key::Char(' '));

        let event = KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert_eq!(convert_raw_event_to_key(event), Key::Backspace);

        let event = KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert_eq!(convert_raw_event_to_key(event), Key::CtrlBackspace);

        let event = KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        assert_eq!(convert_raw_event_to_key(event), Key::AltBackspace);

        let event = KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        assert_eq!(convert_raw_event_to_key(event), Key::Backspace);

        let event = KeyEvent {
            code: KeyCode::Delete,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        assert_eq!(convert_raw_event_to_key(event), Key::Delete);

        let event = KeyEvent {
            code: KeyCode::Delete,
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        assert_eq!(convert_raw_event_to_key(event), Key::CtrlDelete);

        let event = KeyEvent {
            code: KeyCode::Delete,
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        assert_eq!(convert_raw_event_to_key(event), Key::AltDelete);

        let event = KeyEvent {
            code: KeyCode::Delete,
            modifiers: KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        assert_eq!(convert_raw_event_to_key(event), Key::Delete);

        let event = KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        assert_eq!(convert_raw_event_to_key(event), Key::Enter);

        let event = KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        assert_eq!(convert_raw_event_to_key(event), Key::CtrlEnter);

        let event = KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::ALT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        assert_eq!(convert_raw_event_to_key(event), Key::AltEnter);

        let event = KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        assert_eq!(convert_raw_event_to_key(event), Key::Enter);

        let event = KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };

        assert_eq!(convert_raw_event_to_key(event), Key::Up);
    }
}
