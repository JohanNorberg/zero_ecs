use std::collections::HashSet;

use zero_ecs::*;

pub struct Value(usize);

#[entity]
pub struct ValueEntity {
    value: Value,
}

ecs_world!(ValueEntity);

#[system_for_each(World)]
fn inc(value: &mut Value) {
    value.0 += 1;
}

#[system(World)]
fn assert_value(world: &World, values: Query<&Value>, expected: usize) {
    let values = world.with_query(values);
    let value: &Value = values.at(0).unwrap();

    assert_eq!(expected, value.0)
}

#[system(World)]
fn assert_value_for(world: &World, values: Query<&Value>, expected: usize, entity: Entity) {
    let values = world.with_query(values);
    let value: &Value = values.get(entity).unwrap();

    assert_eq!(expected, value.0)
}

#[test]
fn can_mutable_for_each() {
    let mut world = World::default();
    let _ = world.create(ValueEntity { value: Value(0) });

    world.inc();
    world.inc();
    world.inc();

    world.assert_value(3);
}

make_query!(ManualValueMutable, mut Value);

#[test]
fn can_make_mutable_query() {
    let mut world = World::default();
    let a = world.create(ValueEntity { value: Value(0) });
    let b = world.create(ValueEntity { value: Value(0) });
    let c = world.create(ValueEntity { value: Value(0) });
    let d = world.create(ValueEntity { value: Value(0) });

    {
        let mut q = world.with_query_mut(Query::<ManualValueMutable>::new());
        let value: &mut Value = q.get_mut(c).unwrap();

        value.0 += 5;
    }
    {
        let mut q = world.with_query_mut(Query::<ManualValueMutable>::new());
        let value: &mut Value = q.get_mut(a).unwrap();

        value.0 += 5;
    }
    {
        let mut q = world.with_query_mut(Query::<ManualValueMutable>::new());
        let value: &mut Value = q.get_mut(a).unwrap();

        value.0 += 5;
    }

    world.assert_value_for(5, c);
    world.assert_value_for(10, a);
    world.assert_value_for(0, b);
    world.assert_value_for(0, d);
}

struct ToDestroy(HashSet<Entity>);

#[system_for_each(World)]
pub fn destroy_even_values(entity: &Entity, value: &Value, to_destroy: &mut ToDestroy) {
    if value.0 % 2 == 0 {
        to_destroy.0.insert(*entity);
    }
}

struct Sum(usize);

#[system_for_each(World)]
pub fn sum_values(value: &Value, sum: &mut Sum) {
    sum.0 += value.0;
}

#[test]
fn can_destroy_specific() {
    let mut world = World::default();
    let _ = world.create(ValueEntity { value: Value(0) });
    let _ = world.create(ValueEntity { value: Value(1) });
    let _ = world.create(ValueEntity { value: Value(2) });
    let _ = world.create(ValueEntity { value: Value(3) });
    let _ = world.create(ValueEntity { value: Value(4) });
    let mut to_destroy = ToDestroy(HashSet::new());
    world.destroy_even_values(&mut to_destroy);

    to_destroy.0.iter().for_each(|entity| {
        world.destroy(*entity);
    });

    // 1+3 is 4
    let mut sum = Sum(0);
    world.sum_values(&mut sum);

    assert_eq!(4, sum.0);
}
