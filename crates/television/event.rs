use std::{
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
    KeyEvent, KeyModifiers,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::warn;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Event<I> {
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
pub(crate) enum Key {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    CtrlBackspace,
    CtrlEnter,
    CtrlLeft,
    CtrlRight,
    CtrlUp,
    CtrlDown,
    CtrlDelete,
    AltBackspace,
    AltDelete,
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

#[allow(clippy::module_name_repetitions)]
pub(crate) struct EventLoop {
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
    pub(crate) fn new(tick_rate: f64, init: bool) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let tx_c = tx.clone();
        let tick_interval =
            tokio::time::Duration::from_secs_f64(1.0 / tick_rate);

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
                            tx_c.send(Event::Closed).unwrap_or_else(|_| warn!("Unable to send Closed event"));
                            tx_c.send(Event::Tick).unwrap_or_else(|_| warn!("Unable to send Tick event"));
                            break;
                        },
                        // if `delay` completes, pass to the next event "frame"
                        () = delay => {
                            tx_c.send(Event::Tick).unwrap_or_else(|_| warn!("Unable to send Tick event"));
                        },
                        // if the receiver dropped the channel, stop the event loop
                        () = tx_c.closed() => break,
                        // if an event was received, process it
                        _ = event_available => {
                            let maybe_event = crossterm::event::read();
                            match maybe_event {
                                Ok(crossterm::event::Event::Key(key)) => {
                                    let key = convert_raw_event_to_key(key);
                                    tx_c.send(Event::Input(key)).unwrap_or_else(|_| warn!("Unable to send {:?} event", key));
                                },
                                Ok(crossterm::event::Event::FocusLost) => {
                                    tx_c.send(Event::FocusLost).unwrap_or_else(|_| warn!("Unable to send FocusLost event"));
                                },
                                Ok(crossterm::event::Event::FocusGained) => {
                                    tx_c.send(Event::FocusGained).unwrap_or_else(|_| warn!("Unable to send FocusGained event"));
                                },
                                Ok(crossterm::event::Event::Resize(x, y)) => {
                                    tx_c.send(Event::Resize(x, y)).unwrap_or_else(|_| warn!("Unable to send Resize event"));
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

pub(crate) fn convert_raw_event_to_key(event: KeyEvent) -> Key {
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
            _ => Key::Enter,
        },
        Up => match event.modifiers {
            KeyModifiers::CONTROL => Key::CtrlUp,
            _ => Key::Up,
        },
        Down => match event.modifiers {
            KeyModifiers::CONTROL => Key::CtrlDown,
            _ => Key::Down,
        },
        Left => match event.modifiers {
            KeyModifiers::CONTROL => Key::CtrlLeft,
            _ => Key::Left,
        },
        Right => match event.modifiers {
            KeyModifiers::CONTROL => Key::CtrlRight,
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
        Char(c) => match event.modifiers {
            KeyModifiers::NONE | KeyModifiers::SHIFT => Key::Char(c),
            KeyModifiers::CONTROL => Key::Ctrl(c),
            KeyModifiers::ALT => Key::Alt(c),
            _ => Key::Null,
        },
        _ => Key::Null,
    }
}
