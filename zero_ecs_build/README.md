# Zero ECS

Zero ECS is an Entity Component System that is written with 4 goals
1. Only use zero cost abstractions - no use of dyn and Box and stuff [zero-cost-abstractions](https://doc.rust-lang.org/beta/embedded-book/static-guarantees/zero-cost-abstractions.html).
2. No use of unsafe rust code.
3. Be very user friendly. The user should write as little boilerplate as possible.
4. Be very fast

It achieves this by generating all code at compile time, using a combination of macros and build scripts.

## Instructions

Create a new project

```sh
cargo new zero_ecs_example
cd zero_ecs_example
```

Add the dependencies

```sh
cargo add zero_ecs
cargo add zero_ecs_build --build
```

Your Cargo.toml should look something like this:

```sh
[dependencies]
zero_ecs = "*"

[build-dependencies]
zero_ecs_build = "*"
```

Create `build.rs`

```sh
touch build.rs
```

Edit `build.rs` to call the zero_ecs's build generation code.

```rust
use zero_ecs_build::*;
fn main() {
    generate_ecs("src/main.rs"); // look for components, entities and systems in main.rs
}
```

This will generate the entity component system based on the component, entities and systems in main.rs.
It accepts a glob so you can use wild cards.

```rust
use zero_ecs_build::*;
fn main() {
    generate_ecs("src/**/*.rs"); // look in all *.rs files in src. 
}
```

## Using the ECS

### Include ECS

In `main.rs`
Include the ECS like so:

```rust
include!(concat!(env!("OUT_DIR"), "/zero_ecs.rs"));
```

### Components

Define some components:

Position and velocity has x and y

```rust
#[component]
struct Position(f32, f32);

#[component]
struct Velocity(f32, f32);
```

It is normal to "tag" entities with a component in ECS to be able to single out those entities in systems.

```rust
#[component]
struct EnemyComponent;

#[component]
struct PlayerComponent;
```

### Entities

Entities are a collection of components, they may also be referred to as archetypes, or bundles, or game objects. 
Note that once "in" the ECS. An Entity is simply an ID that can be copied.

In our example, we define an enemy and a player, they both have position and velocity but can be differentiated by their "tag" components. 

```rust
#[entity]
struct Enemy {
    position: Position,
    velocity: Velocity,
    enemy_component: EnemyComponent,
}

#[entity]
struct Player {
    position: Position,
    velocity: Velocity,
    player_component: PlayerComponent,
}
```

### Systems

Systems run the logic for the application. They can accept references, mutable references and queries. 

In our example we can create a system that simply prints the position of all entities

```rust
#[system]
fn print_positions(world: &World, query: Query<&Position>) {
    world.with_query(query).iter().for_each(|position| {
        println!("Position: {:?}", position);
    });
}
```

#### Explained:

- world: &World - Since the system doesn't modify anything, it can be an immutable reference
- query: Query<&Position> - We want to query the world for all positions
- world.with_query(query).iter() - creates an iterator over all Position components

### Creating entities and calling system

In our `fn main` change it to create 10 enemies and 10 players,
Also add the `systems_main(&world);` to call all systems.

```rust
fn main() {
    let mut world = World::default();

    for i in 0..10 {
        world.create(Enemy {
            position: Position(i as f32, 5.0),
            velocity: Velocity(0.0, 1.0),
            ..Default::default()
        });

        world.create(Player {
            position: Position(5.0, i as f32),
            velocity: Velocity(1.0, 0.0),
            ..Default::default()
        });
    }

    systems_main(&world);
}
```

Running the program now, will print the positions of the entities. 

## More advanced

Continuing our example

### mutating systems

Most systems will mutate the world state and needs additional resources, like texture managers, time managers, input managers etc. 
A good practice is to group them in a Resources struct. (But Not nescessary)

```rust
struct Resources {
    delta_time: f32,
}

#[system]
fn apply_velocity(
    world: &mut World, // world mut be mutable
    resources: &Resources, // we need the delta time
    query: Query<(&mut Position, &Velocity)>, // position should be mutable, velocity not.
) {
    world
        .with_query_mut(query) // we call with_query_mut because it's a mutable query
        .iter_mut() // iterating mutable
        .for_each(|(position, velocity)| {
            position.0 += velocity.0 * resources.delta_time;
            position.1 += velocity.1 * resources.delta_time;
        });
}

```

We also have to change the main function to include resources in the call. 

```rust
let resources = Resources { delta_time: 0.1 };

systems_main(&resources, &mut world);
```


#### Destroying entities

Let's say we want to create a rule that if player and enemies get within 3 units of eachother they should both be destroyed. 
This is how we might implement that:

```rust
#[system]
fn collide_enemy_and_players(
    world: &mut World, // we are destorying entities so it needs to be mutable
    players: Query<(&Entity, &Position, &PlayerComponent)>, // include the Entity to be able to identify entities
    enemies: Query<(&Entity, &Position, &EnemyComponent)>, // same but for enemies
) {
    let mut entities_to_destroy: Vec<Entity> = vec![]; // we can't (for obvious reasons) destroy entities from within an iteration.

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
                        entities_to_destroy.push(*player_entity);
                        entities_to_destroy.push(*enemy_entity);
                    }
                });
        });

    for entity in entities_to_destroy {
        world.destroy(entity);
    }
}
```

#### Get & At entities

Get is identical to query but takes an Entity. 
At is identical to query but takes an index.

Let's say you wanted an entity that follows a player. This is how you could implement that:

Define a component for the companion
```rust
#[component]
struct CompanionComponent {
    target_entity: Option<Entity>,
}
```

Define the Companion Entity. It has a position and a companion component:
```rust
#[entity]
struct Companion {
    position: Position,
    companion_component: CompanionComponent,
}
```

Now we need to write the companion system. 
For every companion we need to check if it has a target. 
If it has a target we need to check if target exists (it could have been deleted).
If the target exists we get the *value of* target's position and set the companion's position with that value. 

We need to query for companions and their position as mutable. And we need to query for every entity that has a position. This means a companion could technically follow it self. 

```rust
#[system]
fn companion_follow(
    world: &mut World,
    companions: Query<(&mut Position, &CompanionComponent)>,
    positions: Query<&Position>,
) {
```

Implementation: 
We can't simply iterate through the companions, get the target position and update the position because we can only have one borrow if the borrow is mutable (unless we use unsafe code).

We can do what we did with destroying entities, but it will be slow.

The solution is to iterate using index, only borrowing what we need for a short time:


```rust
#[system]
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
                    .map(|p| (p.0, p.1)) // map to get the VALUE
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

# TODO:
- [ ] Re use IDs of deleted entities

