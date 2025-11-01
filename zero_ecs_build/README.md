# Zero ECS

Zero ECS is an Entity Component System that is written with 4 goals
1. Only use zero cost abstractions - no use of dyn and Box and stuff [zero-cost-abstractions](https://doc.rust-lang.org/beta/embedded-book/static-guarantees/zero-cost-abstractions.html).
2. No use of unsafe rust code.
3. Be very user friendly. The user should write as little boilerplate as possible.
4. Be very fast

It achieves this by generating all code at compile time, using a combination of macros and build scripts.

> **This is version `0.3.*`.**
> It is almost a complete rewrite from `0.2.*`. And has breaking changes.

## Instructions

Add the dependency:

```
cargo add zero_ecs
```

Your `Cargo.toml` should look something like this:

```toml
[dependencies]
zero_ecs = "0.3.*"
```

## Using the ECS

### Components

Components are just regular structs.

```rust
#[derive(Default)]
struct Position(f32, f32);

#[derive(Default)]
struct Velocity(f32, f32);
```

It is normal to "tag" entities with a component in ECS to be able to single out those entities in systems.

```rust
#[derive(Default)]
struct EnemyComponent;

#[derive(Default)]
struct PlayerComponent;
```

### Entities & World

Entities are a collection of components. Use the `#[entity]` attribute to define them.

```rust
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
```

Define the world using the `ecs_world!` macro. Must include all entities.

World and entities must be defined in the same crate.

```rust
ecs_world!(EnemyEntity, PlayerEntity);
```

You can now instantiate the world like this:

```rust
let mut world = World::default();
```

And create entities like this:

```rust
let player_entity = world.create(PlayerEntity {
    position: Position(55.0, 165.0),
    velocity: Velocity(100.0, 50.0),
    ..Default::default()
});
```

### Systems

Systems run the logic for the application. There are two types of systems: `#[system]` and `#[system_for_each]`.

#### system_for_each

`#[system_for_each]` calls the system once for each successful query. This is the simplest way to write systems.

```rust
#[system_for_each(World)]
fn print_positions(position: &Position) {
    println!("x: {}, y: {}", position.0, position.1);
}
```

Systems can also mutate and accept resources:

```rust
struct DeltaTime(f32);

#[system_for_each(World)]
fn apply_velocity(position: &mut Position, velocity: &Velocity, delta_time: &DeltaTime) {
    position.0 += velocity.0 * delta_time.0;
    position.1 += velocity.1 * delta_time.0;
}
```

#### system

The default way of using systems. Needs to accept world, 0-many queries and optional resources.

```rust
#[system(World)]
fn print_enemy_positions(world: &World, query: Query<(&Position, &EnemyComponent)>) {
    world.with_query(query).iter().for_each(|(pos, _)| {
        println!("x: {}, y: {}", pos.0, pos.1);
    });
}
```

### Creating entities and calling systems

```rust
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

    world.apply_velocity(&delta_time);
    world.print_positions();
    world.print_enemy_positions();
}
```

## More advanced

### Destroying entities

To destroy entities, query for `&Entity` to identify them. You can't destroy entities from within an iteration.

```rust
#[system(World)]
fn collide_enemy_and_players(
    world: &mut World,
    players: Query<(&Entity, &Position, &PlayerComponent)>,
    enemies: Query<(&Entity, &Position, &EnemyComponent)>,
) {
    let mut entities_to_destroy: HashSet<Entity> = HashSet::new();

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
```

### Get & At

`get` is identical to query but takes an `Entity`.
`at` is identical to query but takes an index.

Let's say you wanted an entity that follows a player:

```rust
struct CompanionComponent {
    target_entity: Option<Entity>,
}

#[entity]
struct CompanionEntity {
    position: Position,
    companion_component: CompanionComponent,
}
```

We can't simply iterate through the companions and get the target position because we can only have one borrow if the borrow is mutable. The solution is to iterate using index, only borrowing what we need for a short time:

```rust
#[system(World)]
fn companion_follow(
    world: &mut World,
    companions: Query<(&mut Position, &CompanionComponent)>,
    positions: Query<&Position>,
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
```

### Manual queries

You can create queries outside systems using `make_query!`. Should rarely be used.

```rust
fn print_player_positions(world: &World) {
    make_query!(PlayerPositionsQuery, Position, PlayerComponent);
    world
        .with_query(Query::<PlayerPositionsQuery>::new())
        .iter()
        .for_each(|(pos, _)| {
            println!("x: {}, y: {}", pos.0, pos.1);
        });
}
```
