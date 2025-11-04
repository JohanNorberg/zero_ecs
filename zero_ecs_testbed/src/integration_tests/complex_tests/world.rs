use super::*;

#[entity]
pub struct ValueEntity {
    value: Value,
}

ecs_world!(ValueEntity);
