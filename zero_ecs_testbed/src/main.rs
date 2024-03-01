#![allow(dead_code, unused_mut, unused_variables)]

// include main_ecs.rs
include!(concat!(env!("OUT_DIR"), "/zero_ecs.rs"));

use zero_ecs::{component, entity, make_mut, system};

#[component]
struct Position(f32, f32);

#[component]
struct Velocity(f32, f32);

#[entity]
struct Enemy {
    position: Position,
    velocity: Velocity,
}

#[entity]
struct Flower {
    position: Position,
}

#[system]
fn print_positions(mut query: Query<&mut Position>) {}

#[system]
fn apply_velocity(mut query: Query<(&mut Position, &Velocity)>) {}

fn main() {
    println!("Hello, world!");

    let mut world = World::default();

    let t = world.query::<&mut Position>();
}

// create some unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query() {
        let mut world = World::default();
        let t = world.query::<&mut Position>();
    }

    #[test]
    fn test_create_entities() {
        let mut world = World::default();
        let e = world.create(Enemy {
            position: Position(0.0, 0.0),
            velocity: Velocity(1.0, 1.0),
        });
        let f = world.create(Flower {
            position: Position(0.0, 0.0),
        });
        let f1 = world.create(Flower {
            position: Position(0.0, 0.0),
        });

        assert!(matches!(e.entity_type, EntityType::Enemy));
        assert!(matches!(f.entity_type, EntityType::Flower));
        assert!(matches!(f1.entity_type, EntityType::Flower));

        assert_eq!(0, e.id);
        assert_eq!(0, f.id);
        assert_eq!(1, f1.id);
    }
}
