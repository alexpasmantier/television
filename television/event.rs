use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent,
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

/// This is a stripped down version of crossterm's `KeyEvent`.
/// Only meant to represent key presses and repeats (not releases, these get converted to the
/// `NULL_KEY`).
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash, PartialOrd)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

pub const NULL_KEY: Key = Key {
    code: KeyCode::Null,
    modifiers: KeyModifiers::NONE,
};

impl Key {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    /// The character this key types into the input, if any: a char key with no
    /// modifiers other than shift.
    pub fn text_char(&self) -> Option<char> {
        match self.code {
            KeyCode::Char(c)
                if (self.modifiers - KeyModifiers::SHIFT).is_empty() =>
            {
                Some(c)
            }
            _ => None,
        }
    }

    // Copied over from crossterm's KeyEvent::normalize_case
    pub fn normalize_case(mut self) -> Key {
        let KeyCode::Char(c) = self.code else {
            return self;
        };

        if c.is_ascii_uppercase() {
            self.modifiers.insert(KeyModifiers::SHIFT);
        } else if self.modifiers.contains(KeyModifiers::SHIFT) {
            self.code = KeyCode::Char(c.to_ascii_uppercase());
        }
        self
    }

    pub fn ctrl(c: char) -> Self {
        Self {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::CONTROL,
        }
    }

    pub fn escape() -> Self {
        Self {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
        }
    }

    pub fn enter() -> Self {
        Self {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
        }
    }

    pub fn backspace() -> Self {
        Self {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
        }
    }

    pub fn delete() -> Self {
        Self {
            code: KeyCode::Delete,
            modifiers: KeyModifiers::NONE,
        }
    }

    pub fn tab() -> Self {
        Self {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
        }
    }

    pub fn space() -> Self {
        Self {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
        }
    }

    pub fn up() -> Self {
        Self {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
        }
    }

    pub fn down() -> Self {
        Self {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
        }
    }

    pub fn left() -> Self {
        Self {
            code: KeyCode::Left,
            modifiers: KeyModifiers::NONE,
        }
    }

    pub fn right() -> Self {
        Self {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
        }
    }

    pub fn home() -> Self {
        Self {
            code: KeyCode::Home,
            modifiers: KeyModifiers::NONE,
        }
    }

    pub fn end() -> Self {
        Self {
            code: KeyCode::End,
            modifiers: KeyModifiers::NONE,
        }
    }

    pub fn page_up() -> Self {
        Self {
            code: KeyCode::PageUp,
            modifiers: KeyModifiers::NONE,
        }
    }

    pub fn page_down() -> Self {
        Self {
            code: KeyCode::PageDown,
            modifiers: KeyModifiers::NONE,
        }
    }

    pub fn back_tab() -> Self {
        Self {
            code: KeyCode::BackTab,
            modifiers: KeyModifiers::NONE,
        }
    }

    pub fn f(n: u8) -> Self {
        Self {
            code: KeyCode::F(n),
            modifiers: KeyModifiers::NONE,
        }
    }
}

impl From<KeyCode> for Key {
    fn from(keycode: KeyCode) -> Self {
        Self {
            code: keycode,
            modifiers: KeyModifiers::NONE,
        }
    }
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
        let mut modifiers = Vec::new();
        if self.modifiers.contains(KeyModifiers::SUPER) {
            modifiers.push("super");
        }
        if self.modifiers.contains(KeyModifiers::CONTROL) {
            modifiers.push("ctrl");
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
            modifiers.push("alt");
        }
        if self.modifiers.contains(KeyModifiers::SHIFT) {
            modifiers.push("shift");
        }
        if !modifiers.is_empty() {
            write!(f, "{}-", modifiers.join("-"))?;
        }
        if let KeyCode::Char(c) = self.code {
            write!(f, "{}", c)
        } else if let KeyCode::F(n) = self.code {
            write!(f, "f{}", n)
        } else {
            // convert to lowercase
            let code_str = format!("{:?}", self.code).to_lowercase();
            write!(f, "{}", code_str)
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
                        tx.send(Event::Input(Key::ctrl('c'))).unwrap_or_else(|_| warn!("Unable to send Ctrl-C event"));
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

/// We only keep key presses and repeats, not releases.
/// Releases are converted to the `NULL_KEY`.
pub fn convert_raw_event_to_key(event: KeyEvent) -> Key {
    trace!("Raw event: {:?}", event);
    if event.kind == KeyEventKind::Release {
        return NULL_KEY;
    }
    if event.code == KeyCode::BackTab
        || event.code == KeyCode::Tab && event.modifiers == KeyModifiers::SHIFT
    {
        return Key::new(KeyCode::BackTab, KeyModifiers::NONE);
    }
    Key::new(event.code, event.modifiers).normalize_case()
}

#[cfg(test)]
mod tests {
    use super::*;
    use yare::parameterized;

    #[parameterized(
        plain_char = { Key::new(KeyCode::Char('a'), KeyModifiers::NONE), "a" },
        ctrl_char = { Key::new(KeyCode::Char('a'), KeyModifiers::CONTROL), "ctrl-a" },
        alt_char = { Key::new(KeyCode::Char('a'), KeyModifiers::ALT), "alt-a" },
        shift_char = { Key::new(KeyCode::Char('a'), KeyModifiers::SHIFT), "shift-a" },
        super_char = { Key::new(KeyCode::Char('a'), KeyModifiers::SUPER), "super-a" },
        ctrl_alt_char = { Key::new(KeyCode::Char('a'), KeyModifiers::CONTROL | KeyModifiers::ALT), "ctrl-alt-a" },
        ctrl_shift_char = { Key::new(KeyCode::Char('a'), KeyModifiers::CONTROL | KeyModifiers::SHIFT), "ctrl-shift-a" },
        alt_shift_char = { Key::new(KeyCode::Char('a'), KeyModifiers::ALT | KeyModifiers::SHIFT), "alt-shift-a" },
        ctrl_alt_shift_char = { Key::new(KeyCode::Char('a'), KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT), "ctrl-alt-shift-a" },
        f_key = { Key::new(KeyCode::F(1), KeyModifiers::NONE), "f1" },
        enter_key = { Key::new(KeyCode::Enter, KeyModifiers::NONE), "enter" },
        ctrl_enter_key = { Key::new(KeyCode::Enter, KeyModifiers::CONTROL), "ctrl-enter" },
    )]
    fn test_key_display(key: Key, expected: &str) {
        assert_eq!(key.to_string(), expected);
    }

    #[parameterized(
        lowercase = { KeyCode::Char('a'), KeyModifiers::NONE, Some('a') },
        // crossterm reports typed capitals with the shift modifier set
        uppercase = { KeyCode::Char('A'), KeyModifiers::SHIFT, Some('A') },
        space = { KeyCode::Char(' '), KeyModifiers::NONE, Some(' ') },
        ctrl = { KeyCode::Char('a'), KeyModifiers::CONTROL, None },
        alt = { KeyCode::Char('a'), KeyModifiers::ALT, None },
        ctrl_shift = { KeyCode::Char('A'), KeyModifiers::CONTROL | KeyModifiers::SHIFT, None },
        non_char = { KeyCode::Enter, KeyModifiers::NONE, None },
    )]
    fn test_typed_key_to_text_char(
        code: KeyCode,
        modifiers: KeyModifiers,
        expected: Option<char>,
    ) {
        let key = convert_raw_event_to_key(KeyEvent::new(code, modifiers));
        assert_eq!(key.text_char(), expected);
    }
}
