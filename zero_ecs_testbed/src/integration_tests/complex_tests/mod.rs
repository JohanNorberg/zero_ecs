// to test separating systems into other files

mod components;
mod system;
mod world;
use zero_ecs::*;

use components::*;
use system::*;
use world::*;

#[test]
fn can_make_mutable_query() {
    let mut world = World::default();
    let _ = world.create(ValueEntity { value: Value(0) });
    let _ = world.create(ValueEntity { value: Value(0) });
    let _ = world.create(ValueEntity { value: Value(0) });
    let _ = world.create(ValueEntity { value: Value(0) });

    world.inc();
    world.inc();
    world.inc();

    world.assert_value(3);
}
