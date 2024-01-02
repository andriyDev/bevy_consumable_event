use bevy_app::{App, First};
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

/// A system for clearing all events of type `E`.
fn clear_all_events<E: Event>(mut events: ResMut<ConsumableEvents<E>>) {
  events.clear();
}

/// A system for clearing just the consumed events of type `E`.
fn clear_consumed_events<E: Event>(mut events: ResMut<ConsumableEvents<E>>) {
  events.clear_consumed();
}
