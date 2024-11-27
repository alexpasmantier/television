use std::{
    fmt::Display,
    future::Future,
    pin::Pin,
    task::{Context, Poll as TaskPoll},
    time::Duration,
};

use crossterm::event::{
    KeyCode::{
        BackTab, Backspace, Char, Delete, Down, End, Enter, Esc, Home, Insert,
        Left, PageDown, PageUp, Right, Tab, Up, F,
    },
    KeyEvent, KeyEventKind, KeyModifiers,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, warn};

#[derive(Debug, Clone, Copy)]
pub enum Event<I> {
    Closed,
    Input(I),
    FocusLost,
    FocusGained,
    Resize(u16, u16),
    Tick,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash,
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

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Key::Backspace => write!(f, "Backspace"),
            Key::Enter => write!(f, "Enter"),
            Key::Left => write!(f, "Left"),
            Key::Right => write!(f, "Right"),
            Key::Up => write!(f, "Up"),
            Key::Down => write!(f, "Down"),
            Key::CtrlSpace => write!(f, "Ctrl-Space"),
            Key::CtrlBackspace => write!(f, "Ctrl-Backspace"),
            Key::CtrlEnter => write!(f, "Ctrl-Enter"),
            Key::CtrlLeft => write!(f, "Ctrl-Left"),
            Key::CtrlRight => write!(f, "Ctrl-Right"),
            Key::CtrlUp => write!(f, "Ctrl-Up"),
            Key::CtrlDown => write!(f, "Ctrl-Down"),
            Key::CtrlDelete => write!(f, "Ctrl-Del"),
            Key::AltSpace => write!(f, "Alt-Space"),
            Key::AltEnter => write!(f, "Alt-Enter"),
            Key::AltBackspace => write!(f, "Alt-Backspace"),
            Key::AltDelete => write!(f, "Alt-Delete"),
            Key::AltUp => write!(f, "Alt-Up"),
            Key::AltDown => write!(f, "Alt-Down"),
            Key::AltLeft => write!(f, "Alt-Left"),
            Key::AltRight => write!(f, "Alt-Right"),
            Key::Home => write!(f, "Home"),
            Key::End => write!(f, "End"),
            Key::PageUp => write!(f, "PageUp"),
            Key::PageDown => write!(f, "PageDown"),
            Key::BackTab => write!(f, "BackTab"),
            Key::Delete => write!(f, "Delete"),
            Key::Insert => write!(f, "Insert"),
            Key::F(k) => write!(f, "F{k}"),
            Key::Char(c) => write!(f, "{c}"),
            Key::Alt(c) => write!(f, "Alt-{c}"),
            Key::Ctrl(c) => write!(f, "Ctrl-{c}"),
            Key::Null => write!(f, "Null"),
            Key::Esc => write!(f, "Esc"),
            Key::Tab => write!(f, "Tab"),
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct EventLoop {
    pub rx: mpsc::UnboundedReceiver<Event<Key>>,
    //tx: mpsc::UnboundedSender<Event<Key>>,
    pub abort_tx: mpsc::UnboundedSender<()>,
    //tick_rate: std::time::Duration,
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

impl EventLoop {
    pub fn new(tick_rate: f64, init: bool) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let tick_interval = Duration::from_secs_f64(1.0 / tick_rate);

        let (abort, mut abort_recv) = mpsc::unbounded_channel();

        if init {
            //let mut reader = crossterm::event::EventStream::new();
            tokio::spawn(async move {
                loop {
                    //let event = reader.next();
                    let delay = tokio::time::sleep(tick_interval);
                    let event_available = poll_event(tick_interval);

                    tokio::select! {
                        // if we receive a message on the abort channel, stop the event loop
                        _ = abort_recv.recv() => {
                            tx.send(Event::Closed).unwrap_or_else(|_| warn!("Unable to send Closed event"));
                            tx.send(Event::Tick).unwrap_or_else(|_| warn!("Unable to send Tick event"));
                            break;
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
                                Ok(crossterm::event::Event::FocusLost) => {
                                    tx.send(Event::FocusLost).unwrap_or_else(|_| warn!("Unable to send FocusLost event"));
                                },
                                Ok(crossterm::event::Event::FocusGained) => {
                                    tx.send(Event::FocusGained).unwrap_or_else(|_| warn!("Unable to send FocusGained event"));
                                },
                                Ok(crossterm::event::Event::Resize(x, y)) => {
                                    tx.send(Event::Resize(x, y)).unwrap_or_else(|_| warn!("Unable to send Resize event"));
                                },
                                _ => {}
                            }
                        }
                    }
                }
            });
        }

        Self {
            //tx,
            rx,
            //tick_rate,
            abort_tx: abort,
        }
    }
}

pub fn convert_raw_event_to_key(event: KeyEvent) -> Key {
    debug!("Raw event: {:?}", event);
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
