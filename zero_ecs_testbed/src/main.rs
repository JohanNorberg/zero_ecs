#![allow(dead_code, unused_mut, unused_variables, unused_imports)]

// include main_ecs.rs
include!(concat!(env!("OUT_DIR"), "/zero_ecs.rs"));

use zero_ecs::{component, entity, make_mut, system};

#[component]
struct Position(f32, f32);

#[component]
struct Velocity(f32, f32);

#[component]
struct Name(String);

#[component]
struct EnemyTag;

#[component]
struct FlowerTag;

#[entity]
struct Enemy {
    position: Position,
    velocity: Velocity,
    name: Name,
    enemy_tag: EnemyTag,
}

#[entity]
struct NameEntity {
    name: Name,
}

#[entity]
struct Flower {
    position: Position,
    flower_tag: FlowerTag,
}

#[system]
fn print_positions(world: &mut World, query: Query<&Position>) {
    world.with_query(query).iter().for_each(|pos| {
        println!("Position: {:?}", pos);
    });
}
#[system]
fn print_positions_copy(world: &mut World, query: Query<&Position>) {
    world.with_query(query).iter().for_each(|pos| {
        println!("Position: {:?}", pos);
    });
}

#[system]
fn get_name(world: &World, query: Query<&Name>, entity: Entity) -> Option<String> {
    if let Some(name) = world.with_query(query).get(entity) {
        Some(name.0.to_string())
    } else {
        None
    }
}

#[system]
fn apply_velocity(world: &mut World, query: Query<(&mut Position, &Velocity)>) {}

#[system]
fn count_types(
    world: &mut World,
    mut enemy_query: Query<&EnemyTag>,
    mut flower_query: Query<&FlowerTag>,
) {
    let mut test = 0;
    for e in world.with_query(enemy_query).iter() {
        test += 1;
        println!("enemy: {}", test);
        for f in world.with_query(flower_query).iter() {
            test += 1;
            println!("flower: {}", test);
        }
    }
}

fn main() {
    println!("Hello, world!");

    let mut world = World::default();
    let e = world.create(Enemy {
        position: Position(0.0, 0.0),
        velocity: Velocity(1.0, 1.0),
        ..Default::default()
    });
    let f = world.create(Flower {
        position: Position(0.0, 0.0),
        ..Default::default()
    });
    let f1 = world.create(Flower {
        position: Position(0.0, 0.0),
        ..Default::default()
    });

    print_positions(&mut world, Query::new());
    count_types(&mut world, Query::new(), Query::new());
}

// create some unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_entities() {
        let mut world = World::default();
        let e = world.create(Enemy {
            position: Position(0.0, 0.0),
            velocity: Velocity(1.0, 1.0),
            name: Name("test".into()),
            ..Default::default()
        });
        let f = world.create(Flower {
            position: Position(1.0, 0.0),
            ..Default::default()
        });
        let f1 = world.create(Flower {
            position: Position(0.0, 0.0),
            ..Default::default()
        });

        assert!(matches!(e.entity_type, EntityType::Enemy));
        assert!(matches!(f.entity_type, EntityType::Flower));
        assert!(matches!(f1.entity_type, EntityType::Flower));

        assert_eq!(0, e.id);
        assert_eq!(0, f.id);
        assert_eq!(1, f1.id);

        let name: Option<&Name> = world.get_from(e);
        assert!(name.is_some());
        let name = &name.unwrap().0;
        assert_eq!("test", name);
        let name: Option<&Name> = world.get_from(f);
        assert!(name.is_none());
    }

    #[test]
    fn test_create_and_destroy_entity() {
        let mut world = World::default();
        // test creating 5 enemies, 100 times,
        for _ in 0..100 {
            let entity_ids: Vec<_> = (0..5)
                .map(|index| {
                    let enemy_name = format!("enemy_{}", index);
                    world.create(NameEntity {
                        name: Name(enemy_name),
                        ..Default::default()
                    })
                })
                .collect();

            {
                let entity_id = entity_ids[1];
                world.destroy(entity_id);
                let try_get: Option<&Name> = world.get_from(entity_id);
                assert!(try_get.is_none());
            }
            {
                let entity_id = entity_ids[3];
                world.destroy(entity_id);
                let try_get: Option<&Name> = world.get_from(entity_id);
                assert!(try_get.is_none());
            }

            // assert that get on destroyed enemies return None
            // assert that not destroyed enemies exist, and that they are named correctly
            for (index, entity_id) in entity_ids.iter().enumerate() {
                let name: Option<&Name> = world.get_from(*entity_id);

                if index == 1 || index == 3 {
                    assert!(name.is_none());
                } else {
                    assert!(name.is_some());
                    let name = &name.unwrap().0;
                    assert_eq!(&format!("enemy_{}", index), name);
                }
            }
        }
    }
}
