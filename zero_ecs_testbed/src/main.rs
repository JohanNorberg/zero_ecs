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
struct Flower {
    position: Position,
    flower_tag: FlowerTag,
}

#[system]
fn print_positions(world: &mut World, query: Query<&mut Position>) {
    //query.iter_mut().for_each(|mut pos| {
    //    println!("Position: {:?}", pos);
    //});
    world.with_query(query).iter_mut().for_each(|mut pos| {
        println!("Position: {:?}", pos);
    });
}

#[system]
fn get_name(world: &mut World, query: Query<&Name>) -> String {
    "".into()
}

#[system]
fn apply_velocity(world: &mut World, query: Query<(&mut Position, &Velocity)>) {}

#[system]
fn count_types(
    world: &mut World,
    mut enemy_query: Query<&EnemyTag>,
    mut flower_query: Query<&FlowerTag>,
) {
    /*let enemy_count = world.with_query(enemy_query).iter_mut().count();
    let flower_count = world.with_query(flower_query).iter_mut().count();
    println!("Enemy count: {}", enemy_count);
    println!("Flower count: {}", flower_count);*/

    let mut test = 0;
    for e in world.with_query(enemy_query).iter_mut() {
        test += 1;
        println!("enemy: {}", test);
        for f in world.with_query(flower_query).iter_mut() {
            test += 1;
            println!("flower: {}", test);
        }
    }
}

pub trait GetMutFrom<'a, T> {
    fn get_mut_from(&'a mut self, entity: Entity) -> Option<T>;
}

impl<'a> GetMutFrom<'a, (&'a mut Position, &'a Velocity)> for Enemies {
    fn get_mut_from(&'a mut self, entity: Entity) -> Option<(&'a mut Position, &'a Velocity)> {
        if let Some(Some(index)) = self.index_lookup.get(entity.id) {
            Some((
                self.positions.get_mut(*index)?,
                self.velocities.get(*index)?,
            ))
        } else {
            None
        }
    }
}

#[allow(unused_parens)]
impl<'a> GetMutFrom<'a, (&'a Name)> for Enemies {
    fn get_mut_from(&'a mut self, entity: Entity) -> Option<(&'a Name)> {
        if let Some(Some(index)) = self.index_lookup.get(entity.id) {
            Some((self.names.get(*index)?))
        } else {
            None
        }
    }
}

#[allow(unused_parens)]
impl<'a> GetMutFrom<'a, (&'a Name)> for World {
    fn get_mut_from(&'a mut self, entity: Entity) -> Option<(&'a Name)> {
        match entity.entity_type {
            EntityType::Enemy => self.enemies.get_mut_from(entity),
            _ => None,
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
            name: Name("test".into()),
        });
        let f = world.create(Flower {
            position: Position(1.0, 0.0),
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

        let name: Option<&Name> = world.get_mut_from(e);
        assert!(name.is_some());
        let name = &name.unwrap().0;
        assert_eq!("test", name);
        let name: Option<&Name> = world.get_mut_from(f);
        assert!(name.is_none());
    }
}
