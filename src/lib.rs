use std::{
  ops::{Deref, DerefMut},
  slice::IterMut,
};

use bevy_ecs::{
  event::Event,
  system::{ResMut, Resource, SystemParam},
};

#[cfg(feature = "bevy_app")]
mod app;
#[cfg(feature = "bevy_app")]
pub use app::ConsumableEventApp;

#[derive(Resource)]
pub struct ConsumableEvents<E: Event> {
  events: Vec<EventInstance<E>>,
}

#[derive(Debug)]
struct EventInstance<E> {
  event: E,
  consumed: bool,
}

// Derived Default impl would incorrectly require E: Default
impl<E: Event> Default for ConsumableEvents<E> {
  fn default() -> Self {
    Self { events: Default::default() }
  }
}

impl<E: Event> ConsumableEvents<E> {
  pub fn send(&mut self, event: E) {
    self.events.push(EventInstance { event, consumed: false });
  }

  pub fn send_batch(&mut self, events: impl IntoIterator<Item = E>) {
    self.extend(events);
  }

  pub fn send_default(&mut self)
  where
    E: Default,
  {
    self.send(Default::default())
  }

  pub fn clear(&mut self) {
    self.events.clear();
  }

  pub fn clear_consumed(&mut self) {
    self.events.retain(|event| !event.consumed);
  }
}

impl<E: Event> Extend<E> for ConsumableEvents<E> {
  fn extend<I>(&mut self, iter: I)
  where
    I: IntoIterator<Item = E>,
  {
    self.events.extend(
      iter.into_iter().map(|event| EventInstance { event, consumed: false }),
    );
  }
}

pub struct Consume<'events, E> {
  event: &'events mut E,
  consumed: &'events mut bool,
}

impl<'events, E> Deref for Consume<'events, E> {
  type Target = E;

  fn deref(&self) -> &Self::Target {
    &self.event
  }
}

impl<'events, E> DerefMut for Consume<'events, E> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.event
  }
}

impl<'events, E> Consume<'events, E> {
  pub fn consume(&mut self) {
    *self.consumed = true;
  }
}

#[derive(SystemParam)]
pub struct ConsumableEventWriter<'w, E: Event> {
  events: ResMut<'w, ConsumableEvents<E>>,
}

impl<'w, E: Event> ConsumableEventWriter<'w, E> {
  pub fn send(&mut self, event: E) {
    self.events.send(event);
  }

  pub fn send_batch(&mut self, events: impl IntoIterator<Item = E>) {
    self.events.send_batch(events);
  }

  pub fn send_default(&mut self)
  where
    E: Default,
  {
    self.events.send_default()
  }
}

#[derive(SystemParam)]
pub struct ConsumableEventReader<'w, E: Event> {
  events: ResMut<'w, ConsumableEvents<E>>,
}

impl<'w, E: Event> ConsumableEventReader<'w, E> {
  pub fn read(&mut self) -> ConsumableEventIterator<E> {
    ConsumableEventIterator { iter: self.events.events.iter_mut() }
  }
}

#[derive(Debug)]
pub struct ConsumableEventIterator<'w, E: Event> {
  iter: IterMut<'w, EventInstance<E>>,
}

impl<'w, E: Event> Iterator for ConsumableEventIterator<'w, E> {
  type Item = Consume<'w, E>;

  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next().filter(|event_instance| !event_instance.consumed).map(
      |event_instance| Consume {
        event: &mut event_instance.event,
        consumed: &mut event_instance.consumed,
      },
    )
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    (0, self.iter.size_hint().1)
  }
}
