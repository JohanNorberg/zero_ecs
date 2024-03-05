// include main_ecs.rs
include!(concat!(env!("OUT_DIR"), "/zero_ecs.rs"));

use zero_ecs::{component, entity, system};

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
#[system(group=last)]
fn print_positions_copy(world: &mut World, query: Query<&Position>) {
    world.with_query(query).iter().for_each(|pos| {
        println!("Position: {:?}", pos);
    });
}

#[system]
fn apply_velocity(world: &mut World, query: Query<(&mut Position, &Velocity)>) {
    world
        .with_query_mut(query)
        .par_iter_mut()
        .for_each(|(pos, vel)| {
            pos.0 += vel.0;
            pos.1 += vel.1;
        });
}

#[system]
fn print_names(world: &mut World, query: Query<&Name>) {
    world.with_query(query).iter().for_each(|name| {
        println!("Name: {:?}", name);
    });
}

#[derive(Debug, Default)]
struct Resources {
    test: i32,
}

#[system(group = with_resources)]
fn print_names_with_resources(world: &mut World, query: Query<&Name>, resources: &Resources) {
    world.with_query(query).iter().for_each(|name| {
        println!("Name: {:?}, test: {}", name, resources.test);
    });
}

#[system]
fn count_types(world: &mut World, enemy_query: Query<&EnemyTag>, flower_query: Query<&FlowerTag>) {
    let mut test = 0;
    for _ in world.with_query(enemy_query).iter() {
        test += 1;
        println!("enemy: {}", test);
        for _ in world.with_query(flower_query).iter() {
            test += 1;
            println!("flower: {}", test);
        }
    }
}

#[system]
fn mutate_position_twice(world: &mut World, query: Query<&mut Position>) {
    // multiply all position with 0.99

    world.with_query_mut(query).iter_mut().for_each(|pos| {
        pos.0 *= 0.99;
        pos.1 *= 0.99;
    });

    /*world.with_query_mut(query).iter_mut().for_each(|pos| {
        pos.0 *= 0.99;
        pos.1 *= 0.99;
    });*/
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

    systems_main(&mut world);
    systems_last(&mut world);

    world.destroy(e);
    world.destroy(f);
    world.destroy(f1);
}

// create some unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_iteration() {
        // create an Enemy with position, run apply velocity, check that position is updated
        let mut world = World::default();
        let e = world.create(Enemy {
            position: Position(0.0, 0.0),
            velocity: Velocity(1.0, 1.0),
            ..Default::default()
        });

        apply_velocity(&mut world, Query::new());

        let pos: Option<&Position> = world.get_from(e);
        assert!(pos.is_some());
        let pos = pos.unwrap();
        assert_eq!(1.0, pos.0);
        assert_eq!(1.0, pos.1);
    }

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
