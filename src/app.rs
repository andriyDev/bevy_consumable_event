use bevy_app::{App, First};
use bevy_ecs::{event::Event, system::ResMut};

use crate::ConsumableEvents;

pub trait ConsumableEventApp {
  fn add_consumable_event<E: Event>(&mut self) -> &mut Self;

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

fn clear_all_events<E: Event>(mut events: ResMut<ConsumableEvents<E>>) {
  events.clear();
}

fn clear_consumed_events<E: Event>(mut events: ResMut<ConsumableEvents<E>>) {
  events.clear_consumed();
}
