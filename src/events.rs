use std::{pin::Pin, time::Duration};

use crossterm::event::{Event as CrosstermEvent, *};
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::time::interval;
use tokio_stream::{wrappers::IntervalStream, StreamMap};

use crate::config;

pub struct Events {
    streams: StreamMap<StreamName, Pin<Box<dyn Stream<Item = Event>>>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum StreamName {
    Ticks,
    KeyRefresh,
    Render,
    Crossterm,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Init,
    Quit,
    Error,
    Closed,
    Tick,
    KeyRefresh,
    Render,
    Crossterm(CrosstermEvent),
}

impl Events {
    pub fn new() -> Self {
        Self {
            streams: StreamMap::from_iter([
                (StreamName::Ticks, tick_stream()),
                (StreamName::KeyRefresh, key_refresh_stream()),
                (StreamName::Render, render_stream()),
                (StreamName::Crossterm, crossterm_stream()),
            ]),
        }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.streams.next().await.map(|(_name, event)| event)
    }
}

fn tick_stream() -> Pin<Box<dyn Stream<Item = Event>>> {
    let tick_delay = Duration::from_secs_f64(1.0 / config::get().tick_rate);
    let tick_interval = interval(tick_delay);
    Box::pin(IntervalStream::new(tick_interval).map(|_| Event::Tick))
}

fn key_refresh_stream() -> Pin<Box<dyn Stream<Item = Event>>> {
    let key_refresh_delay = Duration::from_secs_f64(1.0 / config::get().key_refresh_rate);
    let key_refresh_interval = interval(key_refresh_delay);
    Box::pin(IntervalStream::new(key_refresh_interval).map(|_| Event::KeyRefresh))
}

fn render_stream() -> Pin<Box<dyn Stream<Item = Event>>> {
    let render_delay = Duration::from_secs_f64(1.0 / config::get().frame_rate);
    let render_interval = interval(render_delay);
    Box::pin(IntervalStream::new(render_interval).map(|_| Event::Render))
}

fn crossterm_stream() -> Pin<Box<dyn Stream<Item = Event>>> {
    Box::pin(EventStream::new().fuse().filter_map(|event| async move {
        match event {
            // Ignore key release / repeat events
            Ok(CrosstermEvent::Key(key)) if key.kind == KeyEventKind::Release => None,
            Ok(event) => Some(Event::Crossterm(event)),
            Err(_) => Some(Event::Error),
        }
    }))
}
