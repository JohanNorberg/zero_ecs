use super::*;

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
