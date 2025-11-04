use super::*;

#[entity]
pub struct ValueEntity {
    pub value: Value,
}

ecs_world!(ValueEntity);
