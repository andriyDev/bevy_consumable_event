use bevy_app::{App, First, SubApp};
use bevy_ecs::{event::Event, system::ResMut};

use crate::ConsumableEvents;

/// Extension trait for `bevy_app::App` to automatically add consumable events.
pub trait ConsumableEventApp {
  /// Adds a consumable event type of `E`.
  ///
  /// These events are cleared at the start of each frame. Reads occuring
  /// *before* an event writer will **never** see the events it produces.
  ///
  /// An example for when to use this is a click in a user interface. The click
  /// would be produced in PreUpdate, and once a system consumes the click, no
  /// other system can read that click. If a click gets to the end of the frame
  /// without being consumed, no UI elements need it, so the event should be
  /// discarded.
  fn add_consumable_event<E: Event>(&mut self) -> &mut Self;

  /// Adds a "persistent" consumable event type of `E`.
  ///
  /// Only consumed events are cleared at the start of each frame. This allows
  /// users to consume events whenever they want, even several frames after the
  /// event was triggered. However this can cause the events to grow
  /// indefinitely if events are not consumed.
  ///
  /// An example for when to use this is a line of customers. Customers can join
  /// the line at any time, but they can only be served at 1 customer per
  /// second. One system would write customer events at a random rate, and
  /// another system would consume an event whenever a timer goes off.
  fn add_persistent_consumable_event<E: Event>(&mut self) -> &mut Self;
}

impl ConsumableEventApp for App {
  fn add_consumable_event<E: Event>(&mut self) -> &mut Self {
    self
      .init_resource::<ConsumableEvents<E>>()
      .add_systems(First, clear_all_events::<E>)
  }

  fn add_persistent_consumable_event<E: Event>(&mut self) -> &mut Self {
    self
      .init_resource::<ConsumableEvents<E>>()
      .add_systems(First, clear_consumed_events::<E>)
  }
}

impl ConsumableEventApp for SubApp {
  fn add_consumable_event<E: Event>(&mut self) -> &mut Self {
    self
      .init_resource::<ConsumableEvents<E>>()
      .add_systems(First, clear_all_events::<E>)
  }

  fn add_persistent_consumable_event<E: Event>(&mut self) -> &mut Self {
    self
      .init_resource::<ConsumableEvents<E>>()
      .add_systems(First, clear_consumed_events::<E>)
  }
}

/// A system for clearing all events of type `E`.
fn clear_all_events<E: Event>(mut events: ResMut<ConsumableEvents<E>>) {
  events.clear();
}

/// A system for clearing just the consumed events of type `E`.
fn clear_consumed_events<E: Event>(mut events: ResMut<ConsumableEvents<E>>) {
  events.clear_consumed();
}

#[cfg(test)]
mod tests {
  use crate::*;
  use bevy::ecs::schedule::ScheduleLabel;
  use bevy::prelude::*;

  #[derive(Event, Default)]
  struct TestEvent {
    value: usize,
  }

  fn run_first_schedule(world: &mut World) {
    world.run_schedule(First);
  }

  fn write_events(mut events: ConsumableEventWriter<TestEvent>) {
    for value in 0..10 {
      events.send(TestEvent { value });
    }
  }

  fn consume_odds_and_halve_evens(
    mut events: ConsumableEventReader<TestEvent>,
  ) {
    for mut event in events.read() {
      if event.value % 2 == 1 {
        event.consume();
      } else {
        event.value /= 2;
      }
    }
  }

  #[test]
  fn add_consumable_event() {
    let mut app = App::empty();
    app.main_mut().update_schedule = Some(Main.intern());
    app.add_consumable_event::<TestEvent>().add_systems(
      Main,
      (run_first_schedule, write_events, consume_odds_and_halve_evens).chain(),
    );

    app.update();
    let values = app
      .world_mut()
      .resource_mut::<ConsumableEvents<TestEvent>>()
      .read()
      .map(|event| event.value)
      .collect::<Vec<_>>();
    assert_eq!(values, [0, 1, 2, 3, 4]);

    app.update();
    let values = app
      .world_mut()
      .resource_mut::<ConsumableEvents<TestEvent>>()
      .read()
      .map(|event| event.value)
      .collect::<Vec<_>>();
    // All the events were cleared, so everything started from scratch.
    assert_eq!(values, [0, 1, 2, 3, 4]);
  }

  #[test]
  fn add_persistent_consumable_event() {
    let mut app = App::empty();
    app.main_mut().update_schedule = Some(Main.intern());
    app.add_persistent_consumable_event::<TestEvent>().add_systems(
      Main,
      (run_first_schedule, write_events, consume_odds_and_halve_evens).chain(),
    );

    app.update();
    let values = app
      .world_mut()
      .resource_mut::<ConsumableEvents<TestEvent>>()
      .read()
      .map(|event| event.value)
      .collect::<Vec<_>>();
    assert_eq!(values, [0, 1, 2, 3, 4]);

    app.update();
    let values = app
      .world_mut()
      .resource_mut::<ConsumableEvents<TestEvent>>()
      .read()
      .map(|event| event.value)
      .collect::<Vec<_>>();
    // The old events were consumed/halved, but also a new batch of events were
    // consumed/halved.
    assert_eq!(values, [0, 1, 2, 0, 1, 2, 3, 4]);
  }
}
