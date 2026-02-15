pub trait Event: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Event for T {}

pub struct EventInstance<E: Event> {
    pub event: E,
}

use crate::resource::Resource;
use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard};

pub struct Events<E: Event> {
    events_current: Vec<EventInstance<E>>,
    events_previous: Vec<EventInstance<E>>,
}

impl<E: Event> Resource for Events<E> {}

impl<E: Event> Default for Events<E> {
    fn default() -> Self {
        Self {
            events_current: Vec::new(),
            events_previous: Vec::new(),
        }
    }
}

impl<E: Event> Events<E> {
    pub fn send(&mut self, event: E) {
        self.events_current.push(EventInstance { event });
    }

    pub fn update(&mut self) {
        self.events_previous = std::mem::take(&mut self.events_current);
    }

    pub fn iter_current(&self) -> impl Iterator<Item = &E> {
        self.events_current.iter().map(|e| &e.event)
    }

    pub fn iter_previous(&self) -> impl Iterator<Item = &E> {
        self.events_previous.iter().map(|e| &e.event)
    }

    pub fn drain_current(&mut self) -> std::vec::Drain<'_, EventInstance<E>> {
        self.events_current.drain(..)
    }
}

pub struct EventWriter<'a, E: Event> {
    pub(crate) events: MappedRwLockWriteGuard<'a, Events<E>>,
}

impl<'a, E: Event> EventWriter<'a, E> {
    pub fn new(events: MappedRwLockWriteGuard<'a, Events<E>>) -> Self {
        Self { events }
    }
    pub fn send(&mut self, event: E) {
        self.events.send(event);
    }
}

pub struct EventReader<'a, E: Event> {
    pub(crate) events: MappedRwLockReadGuard<'a, Events<E>>,
    pub(crate) _last_event_count: usize, // Not used in this simple double buffer
}

impl<'a, E: Event> EventReader<'a, E> {
    pub fn new(events: MappedRwLockReadGuard<'a, Events<E>>) -> Self {
        Self {
            events,
            _last_event_count: 0,
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = &E> {
        self.events
            .iter_previous()
            .chain(self.events.iter_current())
    }
}
