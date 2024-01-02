# bevy_consumable_event

A crate to add events to [Bevy](https://bevyengine.org/) that can be consumed.

## Why?

Events in Bevy are very powerful. However one limitation they have is that they
cannot be consumed. For example, if you are clicking in a UI, you'd ideally only
want one system to handle that click. Otherwise, clicking on one UI element
would also click everything underneath it, making "order" irrelevant.

## Usage

The canonical usage is by using `App::add_consumable_event` or
`App::add_persistent_consumable_event` (through the `ConsumableEventApp`
extension trait). Both of these functions allow you to use consumable events,
but which one determines when you can read and consume events.

* `App::add_consumable_event`: Events will be cleared at the start of each
frame. Therefore, only systems running in the same frame *and* after a
`ConsumableEventWriter` will be able to read events.
* `App::add_persistent_consumable_event`: Events are not automatically cleared
(except already consumed events). Therefore, events will stay until a system
consumes them (or you manually clear the events). Note this means that the same
system can read the same event multiple times (so it has multiple opportunities
to consume the event).

In order to consume an event, simply call `consume()` on the items read from the
`ConsumableEventReader`.

```rust
fn consume_all_events(mut events: ConsumableEventReader<MyEvent>) {
  for mut event in events.read() {
    event.consume();
  }
}
```

In addition, you can mutate events read from the `ConsumableEventReader` (so
later reads can act on these mutations).

While using `App::add_consumable_event` and
`App::add_persistent_consumable_event` is the canonical way to use consumable
events, just as with regular events, these are just conveniences. You can just
as easily interact directly with `ConsumableEvents` and have custom clearing
strategies using `ConsumableEvents::clear` and
`ConsumableEvents::clear_consumed`.

## Example

```rust
use bevy::{app::{ScheduleRunnerPlugin, RunMode}, prelude::*};
use bevy_consumable_event::{
  ConsumableEventApp,
  ConsumableEventReader,
  ConsumableEventWriter,
};

fn main() {
  App::new()
    .add_plugins(ScheduleRunnerPlugin { run_mode: RunMode::Once })
    .add_consumable_event::<MyEvent>()
    .add_systems(Main, (
        write_events,
        consume_odds_and_add_ten_to_evens,
        assert_remaining_events,
      ).chain()
    )
    .run();
}

#[derive(Event)]
struct MyEvent {
  value: usize,
}

fn write_events(mut events: ConsumableEventWriter<MyEvent>) {
  for value in 0..10 {
    events.send(MyEvent { value });
  }
}

fn consume_odds_and_add_ten_to_evens(
  mut events: ConsumableEventReader<MyEvent>,
) {
  for mut event in events.read() {
    if event.value % 2 == 1 {
      event.consume();
    } else {
      event.value += 10;
    }
  }
}

fn assert_remaining_events(mut events: ConsumableEventReader<MyEvent>) {
  let values = events.read().map(|event| event.value).collect::<Vec<_>>();
  assert_eq!(values, [10, 12, 14, 16, 18]);
}
```
