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
use bevy::prelude::Event;
use bevy_consumable_event::ConsumableEventReader;

#[derive(Event)]
struct MyEvent;

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

## Caveats

Events and consumable events are not exclusive - it is possible to add a type as
a regular event **and** a consumable event. Consumable events don't care
interact at all with regular events, so everything will work fine. It just may
be confusing.

Events also have multiple strategies to 1) prevent systems from double-reading
events, and 2) prevent systems from missing events. For consumable events, the
hope is these solutions aren't necessary. If you are using non-persistent
consumable events, then a double read shouldn't occur (except from FixedUpdate
systems). Regardless, these are just extra opportunities to consume the events.
Missing events is fine since the assumption going in is that the events are
consumed as they head towards the end of the frame. If you *are* using
persistent consumable events, then a double read is expected. Missing reads are
also not a problem, since events are not cleared.

## License

License under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Contributions are welcome!

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
