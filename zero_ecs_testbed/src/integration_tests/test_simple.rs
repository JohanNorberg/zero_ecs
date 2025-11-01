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
