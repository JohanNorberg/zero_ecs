mod integration_tests;

use std::collections::HashSet;

use zero_ecs::*;

#[derive(Default)]
struct Position(f32, f32);

#[derive(Default)]
struct Velocity(f32, f32);

#[derive(Default)]
struct EnemyComponent;

#[derive(Default)]
struct PlayerComponent;

#[entity]
#[derive(Default)]
struct EnemyEntity {
    position: Position,
    velocity: Velocity,
    enemy_component: EnemyComponent,
}

#[entity]
#[derive(Default)]
struct PlayerEntity {
    position: Position,
    velocity: Velocity,
    player_component: PlayerComponent,
}

struct DeltaTime(f32);

// system_for_each calls the system once for each successful query.
#[system_for_each(World)]
fn print_positions(position: &Position) {
    println!("print_positions - x: {}, y: {}", position.0, position.1);
}

// The default way of using systems. Needs to accept world, 0-many queries and optional resources.
#[system(World)]
fn print_enemy_positions(world: &World, query: QueryDef<(&Position, &EnemyComponent)>) {
    world.with_query(query).iter().for_each(|(pos, _)| {
        println!("print_enemy_positions - x: {}, y: {}", pos.0, pos.1);
    });
}

// Example of how to create queries outside a system. Should rarely be used.
fn print_player_positions(world: &World) {
    // Defines a query called PlayerPositionsQuery for components Position & PlayerComponent
    make_query!(PlayerPositionsQuery, Position, PlayerComponent);
    world
        .with_query(Query::<PlayerPositionsQuery>::new())
        .iter()
        .for_each(|(pos, _)| {
            println!("print_player_positions - x: {}, y: {}", pos.0, pos.1);
        });
}

// Systems can also mutate and accept resources (DeltaTime)
#[system_for_each(World)]
fn apply_velocity(position: &mut Position, velocity: &Velocity, delta_time: &DeltaTime) {
    position.0 += velocity.0 * delta_time.0;
    position.1 += velocity.1 * delta_time.0;
}

// More complex system with multiple queries. Since they are not mutating,
// it's fine to nest queries.
#[system(World)]
fn collide_enemy_and_players(
    world: &mut World, // we are destroying entities so it needs to be mutable
    players: QueryDef<(&Entity, &Position, &PlayerComponent)>, // include the Entity to be able to identify entities
    enemies: QueryDef<(&Entity, &Position, &EnemyComponent)>,  // same but for enemies
) {
    let mut entities_to_destroy: HashSet<Entity> = HashSet::new(); // we can't (for obvious reasons) destroy entities from within an iteration.

    world
        .with_query(players)
        .iter()
        .for_each(|(player_entity, player_position, _)| {
            world
                .with_query(enemies)
                .iter()
                .for_each(|(enemy_entity, enemy_position, _)| {
                    if (player_position.0 - enemy_position.0).abs() < 3.0
                        && (player_position.1 - enemy_position.1).abs() < 3.0
                    {
                        entities_to_destroy.insert(*player_entity);
                        entities_to_destroy.insert(*enemy_entity);
                    }
                });
        });

    for entity in entities_to_destroy {
        world.destroy(entity);
    }
}

struct CompanionComponent {
    target_entity: Option<Entity>,
}

#[entity]
struct CompanionEntity {
    position: Position,
    companion_component: CompanionComponent,
}

// Defines the world, must include all entities.
ecs_world!(EnemyEntity, PlayerEntity, CompanionEntity);

#[system(World)]
fn companion_follow(
    world: &mut World,
    companions: QueryDef<(&mut Position, &CompanionComponent)>,
    positions: QueryDef<&Position>,
) {
    for companion_idx in 0..world.with_query_mut(companions).len() {
        // iterate the count of companions
        if let Some(target_position) = world
            .with_query_mut(companions)
            .at_mut(companion_idx) // get the companion at index companion_idx
            .and_then(|(_, companion)| companion.target_entity) // then get the target entity, if it is not none
            .and_then(|companion_target_entity| {
                // then get the VALUE of target position (meaning we don't use a reference to the position)
                world
                    .with_query(positions)
                    .get(companion_target_entity) // get the position for the companion_target_entity
                    .map(|p: &Position| (p.0, p.1)) // map to get the VALUE
            })
        {
            if let Some((companion_position, _)) =
                world.with_query_mut(companions).at_mut(companion_idx)
            // Then simply get the companion position
            {
                // and update it to the target's position
                companion_position.0 = target_position.0;
                companion_position.1 = target_position.1;
            }
        }
    }
}

fn main() {
    let delta_time = DeltaTime(1.0);

    let mut world = World::default();

    for i in 0..10 {
        world.create(EnemyEntity {
            position: Position(i as f32, 5.0),
            velocity: Velocity(0.0, 1.0),
            ..Default::default()
        });

        world.create(PlayerEntity {
            position: Position(5.0, i as f32),
            velocity: Velocity(1.0, 0.0),
            ..Default::default()
        });
    }

    {
        let player_entity = world.create(PlayerEntity {
            position: Position(55.0, 165.0),
            velocity: Velocity(100.0, 50.0),
            ..Default::default()
        });

        world.create(CompanionEntity {
            position: Position(0.0, 0.0),
            companion_component: CompanionComponent {
                target_entity: Some(player_entity),
            },
        });
    }

    world.collide_enemy_and_players();
    world.apply_velocity(&delta_time);
    world.companion_follow();
    world.print_positions();
    world.print_enemy_positions();
    print_player_positions(&world);
}
