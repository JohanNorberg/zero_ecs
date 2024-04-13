// include main_ecs.rs
include!(concat!(env!("OUT_DIR"), "/zero_ecs.rs"));

use zero_ecs::{component, entity, system};

#[component]
pub struct Position(f32, f32);

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

#[entity]
pub struct EntityWithPosition {
    pub position: Position,
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
        println!("Name: {:?}", name.0);
    });
}

#[derive(Debug, Default)]
struct Resources {
    test: i32,
}

#[component]
struct FollowerComponent {
    target_entity: Option<Entity>,
}

#[entity]
struct FollowerEntity {
    follower: FollowerComponent,
    position: Position,
}

#[component]
struct MyUnused {
    _unused: i32,
}

#[system]
fn unused_system(world: &mut World, le_query: Query<&mut MyUnused>) {
    world
        .with_query_mut(le_query)
        .iter_mut()
        .for_each(|unused| {
            unused._unused += 1;
        });
}

#[system]
fn follower_update_position(
    world: &mut World,
    followers: Query<(&mut Position, &FollowerComponent)>,
    positions: Query<&Position>, // This is all entities with a position. Including the followers, meaning followers can follow followers, and even themselfs.
) {
    // Iterate all followers using idx so we don't borrow world
    for follower_idx in 0..world.with_query_mut(followers).len() {
        // Get the target entity of the follower. Entity is just a lightweight ID that is copied.
        if let Some(target_entity) = world
            .with_query_mut(followers)
            .at_mut(follower_idx)
            .and_then(|(_, follower)| follower.target_entity)
        {
            // If the target entity exists, get its position
            if let Some(target_position) = world
                .with_query(positions)
                .get(target_entity)
                .map(|p| (p.0, p.1))
            {
                // Get the position component of the follower and update its position with the target_position.
                if let Some((follower_position, _)) =
                    world.with_query_mut(followers).at_mut(follower_idx)
                {
                    follower_position.0 = target_position.0;
                    follower_position.1 = target_position.1;
                }
            }
        }
    }
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

    world.with_query_mut(query).iter_mut().for_each(|pos| {
        pos.0 *= 0.99;
        pos.1 *= 0.99;
    });
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
    fn test_followers() {
        // create 10 entities, and 10 followers and make sure they get their target's position
        let mut world = World::default();

        let targets: Vec<_> = (0..10)
            .map(|i| {
                world.create(EntityWithPosition {
                    position: Position(i as f32, i as f32),
                })
            })
            .collect();

        let followers = (0..10)
            .map(|i| {
                world.create(FollowerEntity {
                    follower: FollowerComponent {
                        target_entity: Some(targets[i]),
                    },
                    position: Position(0.0, 0.0),
                })
            })
            .collect::<Vec<_>>();

        follower_update_position(&mut world, Query::new(), Query::new());

        for (i, follower) in followers.iter().enumerate() {
            let target_position: &Position = world.get_from(targets[i]).unwrap();
            let follower_position: &Position = world.get_from(*follower).unwrap();
            assert_eq!(target_position.0, follower_position.0);
            assert_eq!(target_position.1, follower_position.1);
        }
    }

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
