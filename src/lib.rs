#![doc = include_str!("../README.md")]

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

/// A collection of events that can be consumed.
/// Events can be written by using a [`ConsumableEventWriter`] and are typically
/// read using a [`ConsumableEventReader`].
///
/// Events can be read many times, but only "consumed" once. See
/// [`ConsumableEventReader`] for details.
///
/// Generally, all systems using `ConsumableEvents` should be stricly
/// [ordered](https://github.com/bevyengine/bevy/blob/main/examples/ecs/ecs_guide.rs).
/// This ensures that events are consumed in a consistent order.
///
/// Events will remain readable in this collection until they are consumed or
/// [`ConsumableEvents::clear`] is called. Events that have been consumed will
/// only be fully removed once [`ConsumableEvents::clear_consumed`] or
/// [`ConsumableEvents::clear`] is called.
///
/// Typically, `ConsumableEvents` are initialized automatically using the
/// [`ConsumableEventApp`] extension trait.
///
/// # Example
/// ```rust
/// use bevy_ecs::event::Event;
/// use bevy_consumable_event::ConsumableEvents;
///
/// #[derive(Event)]
/// struct MyEvent {
///     value: usize,
/// }
///
/// // setup
/// let mut events = ConsumableEvents::<MyEvent>::default();
///
/// // somewhere else: send an event
/// events.send(MyEvent { value: 1 });
///
/// // somewhere else: read the events, and even mutate them.
/// for mut event in events.read() {
///   assert_eq!(event.value, 1);
///   event.value = 2;
/// }
///
/// // somewhere else: consume the event
/// for mut event in events.read() {
///   assert_eq!(event.value, 2);
///   event.consume();
/// }
///
/// // somewhere else: read the events (nothing left).
/// assert_eq!(events.read().count(), 0);
/// ```
#[derive(Resource)]
pub struct ConsumableEvents<E: Event> {
  /// The events in the buffer. Some of these events may have already been
  /// consumed.
  events: Vec<EventInstance<E>>,
}

/// A single event in the buffer.
#[derive(Debug)]
struct EventInstance<E> {
  /// The actual event.
  event: E,
  /// Whether the event has already been consumed.
  consumed: bool,
}

// Derived Default impl would incorrectly require E: Default
impl<E: Event> Default for ConsumableEvents<E> {
  fn default() -> Self {
    Self { events: Default::default() }
  }
}

impl<E: Event> ConsumableEvents<E> {
  /// "Sends" `event` by writing it to the buffer. [`read`] can then read the
  /// event.
  pub fn send(&mut self, event: E) {
    self.events.push(EventInstance { event, consumed: false });
  }

  /// Sends a list of `events` all at once, which can later be [`read`]. This is
  /// more efficient than sending each event individually.
  pub fn send_batch(&mut self, events: impl IntoIterator<Item = E>) {
    self.extend(events);
  }

  /// Sends the default value of the event. Useful when the event is an empty
  /// struct.
  pub fn send_default(&mut self)
  where
    E: Default,
  {
    self.send(Default::default())
  }

  /// Reads the unconsumed events stored in self.
  pub fn read(&mut self) -> ConsumableEventIterator<E> {
    ConsumableEventIterator { iter: self.events.iter_mut() }
  }

  /// Clears all events stored in self. Unconsumed events are also dropped.
  pub fn clear(&mut self) {
    self.events.clear();
  }

  /// Clears only consumed events stored in self. This is not strictly required,
  /// but calling it regularly will reduce memory usage (since the consumed
  /// events cannot be read anyway).
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

/// Mutable borrow of a consumable event.
pub struct Consume<'events, E> {
  /// The event itself.
  event: &'events mut E,
  /// The consumed flag.
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
  /// "Consumes" the event. "Consumed" simply means that other readers will be
  /// unable to read (or consume) the event. This function does **not** take
  /// ownership of the event value itself (although this can be done using
  /// something like `swap`).
  pub fn consume(&mut self) {
    *self.consumed = true;
  }
}

/// Sends consumable events of type `E`.
///
/// # Usage
///
/// `ConsumableEventWriter`s are usually declared as a [`SystemParam`].
/// ```
/// use bevy_ecs::prelude::*;
/// use bevy_consumable_event::ConsumableEventWriter;
///
/// #[derive(Event)]
/// pub struct MyEvent; // Custom event type.
///
/// fn my_system(mut writer: ConsumableEventWriter<MyEvent>) {
///   writer.send(MyEvent);
/// }
///
/// bevy_ecs::system::assert_is_system(my_system);
/// ```
#[derive(SystemParam)]
pub struct ConsumableEventWriter<'w, E: Event> {
  /// The events to write to.
  events: ResMut<'w, ConsumableEvents<E>>,
}

impl<'w, E: Event> ConsumableEventWriter<'w, E> {
  /// "Sends" `event` by writing it to the buffer. [`ConsumableEventReader`] can
  /// then read the event.
  pub fn send(&mut self, event: E) {
    self.events.send(event);
  }

  /// Sends a list of `events` all at once, which can later be [`read`]. This is
  /// more efficient than sending each event individually.
  pub fn send_batch(&mut self, events: impl IntoIterator<Item = E>) {
    self.events.send_batch(events);
  }

  /// Sends the default value of the event. Useful when the event is an empty
  /// struct.
  pub fn send_default(&mut self)
  where
    E: Default,
  {
    self.events.send_default()
  }
}

/// Reads (and possibly consumes) events of type `E`.
///
/// # Usage
///
/// `ConsumableEventReader`s are usually declared as a [`SystemParam`].
/// ```
/// use bevy_ecs::prelude::*;
/// use bevy_consumable_event::ConsumableEventReader;
///
/// #[derive(Event, Debug)]
/// pub struct MyEvent; // Custom event type.
///
/// fn my_system(mut reader: ConsumableEventReader<MyEvent>) {
///   for mut event in reader.read() {
///     println!("{:?}", *event);
///     event.consume();
///   }
/// }
///
/// bevy_ecs::system::assert_is_system(my_system);
/// ```
#[derive(SystemParam)]
pub struct ConsumableEventReader<'w, E: Event> {
  /// The events to read from.
  events: ResMut<'w, ConsumableEvents<E>>,
}

impl<'w, E: Event> ConsumableEventReader<'w, E> {
  /// Reads the unconsumed events.
  pub fn read(&mut self) -> ConsumableEventIterator<E> {
    self.events.read()
  }
}

/// An iterator over the unconsumed events.
#[derive(Debug)]
pub struct ConsumableEventIterator<'w, E: Event> {
  /// The iterator being wrapped.
  iter: IterMut<'w, EventInstance<E>>,
}

impl<'w, E: Event> Iterator for ConsumableEventIterator<'w, E> {
  type Item = Consume<'w, E>;

  fn next(&mut self) -> Option<Self::Item> {
    self.iter.find(|event_instance| !event_instance.consumed).map(
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

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Event, Default)]
  struct TestEvent {
    value: usize,
  }

  #[test]
  fn consumed_events_are_not_read() {
    let mut events = ConsumableEvents::<TestEvent>::default();

    events.send(TestEvent { value: 1 });
    events.send(TestEvent { value: 2 });
    events.send(TestEvent { value: 3 });
    events.send(TestEvent { value: 4 });

    for mut event in events.read().filter(|event| event.value % 3 == 1) {
      event.consume();
    }

    let unconsumed_values =
      events.read().map(|event| event.value).collect::<Vec<_>>();
    assert_eq!(unconsumed_values, [2, 3]);
  }

  #[test]
  fn cleared_events_are_not_read() {
    let mut events = ConsumableEvents::<TestEvent>::default();

    events.send(TestEvent { value: 1 });
    events.send(TestEvent { value: 2 });
    events.send(TestEvent { value: 3 });
    events.send(TestEvent { value: 4 });

    events.clear();

    assert_eq!(events.read().count(), 0);
  }

  #[test]
  fn clear_consumed_removes_consumed_events() {
    let mut events = ConsumableEvents::<TestEvent>::default();

    events.send(TestEvent { value: 1 });
    events.send(TestEvent { value: 2 });
    events.send(TestEvent { value: 3 });
    events.send(TestEvent { value: 4 });

    events.read().skip(2).for_each(|mut event| event.consume());

    assert_eq!(events.read().count(), 2);
    assert_eq!(events.events.len(), 4);

    events.clear_consumed();

    assert_eq!(events.read().count(), 2);
    assert_eq!(events.events.len(), 2);
  }

  #[test]
  fn send_batch() {
    let mut events = ConsumableEvents::<TestEvent>::default();

    events.send_batch((0..5).map(|value| TestEvent { value }));

    let values = events.read().map(|event| event.value).collect::<Vec<_>>();
    assert_eq!(values, [0, 1, 2, 3, 4]);
  }

  #[test]
  fn send_default() {
    let mut events = ConsumableEvents::<TestEvent>::default();

    events.send_default();
    events.send_default();
    events.send_default();

    let values = events.read().map(|event| event.value).collect::<Vec<_>>();
    assert_eq!(values, [0, 0, 0]);
  }

  #[test]
  fn write_and_read_events_through_systems() {
    use bevy_ecs::prelude::*;

    let mut world = World::new();
    world.init_resource::<ConsumableEvents<TestEvent>>();

    let mut schedule = Schedule::default();
    schedule.add_systems(
      (
        |mut events: ConsumableEventWriter<TestEvent>| {
          events.send(TestEvent { value: 0 });
          events.send_batch((1..=2).map(|value| TestEvent { value }));
          events.send_default();
        },
        |mut events: ResMut<ConsumableEvents<TestEvent>>| {
          assert_eq!(events.read().count(), 4);
        },
        |mut events: ConsumableEventReader<TestEvent>| {
          events.read().for_each(|mut event| event.consume());
        },
        |mut events: ResMut<ConsumableEvents<TestEvent>>| {
          assert_eq!(events.read().count(), 0);
        },
      )
        .chain(),
    );
    schedule.run(&mut world);
    assert_eq!(world.resource::<ConsumableEvents<TestEvent>>().events.len(), 4);
  }
}
