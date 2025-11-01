//! Comprehensive integration tests for the zero_ecs system.
//!
//! This test suite covers:
//! - `#[system(World)]` with various QueryDef types (single, two, three components)
//! - `#[system_for_each(World)]` with various component combinations
//! - Mutable and immutable component queries
//! - Mixed mutability queries (some mutable, some immutable)
//! - Resources: reference resources, value resources, mutable resources
//! - Combinations of queries and resources
//! - Selective system execution (only affecting entities with specific components)
//! - Entity deletion and verification of correct entity removal
//! - Multiple entity deletion scenarios
//! - System call frequency verification (ensuring systems only affect intended entities)

#![allow(dead_code)]

use zero_ecs::*;

// Components for testing
#[derive(Debug, PartialEq)]
pub struct Counter(usize);

#[derive(Debug, PartialEq)]
pub struct Health(i32);

#[derive(Debug, PartialEq)]
pub struct Speed(f32);

#[derive(Debug, PartialEq)]
pub struct Name(String);

#[derive(Default)]
pub struct TagA;

#[derive(Default)]
pub struct TagB;

// Resources for testing
#[derive(Debug)]
struct DeltaTime(f32);

#[derive(Debug)]
struct TickCount(usize);

#[derive(Debug)]
struct GameState {
    paused: bool,
    score: i32,
}

// Entities
#[entity]
pub struct EntityA {
    counter: Counter,
    health: Health,
    tag_a: TagA,
}

#[entity]
pub struct EntityB {
    counter: Counter,
    speed: Speed,
    tag_b: TagB,
}

#[entity]
pub struct EntityAB {
    counter: Counter,
    health: Health,
    speed: Speed,
    name: Name,
}

ecs_world!(EntityA, EntityB, EntityAB);

// ============================================================================
// Tests for #[system(World)] with various QueryDef types
// ============================================================================

#[system(World)]
fn system_single_component_immutable(world: &World, query: QueryDef<&Counter>) {
    let q = world.with_query(query);
    // Just verify we can access
    let count = q.len();
    for i in 0..count {
        let _counter: &Counter = q.at(i).unwrap();
    }
}

#[system(World)]
fn system_single_component_mutable(world: &mut World, query: QueryDef<(&mut Counter, &Health)>) {
    world
        .with_query_mut(query)
        .iter_mut()
        .for_each(|(counter, _)| {
            counter.0 += 1;
        });
}

#[system(World)]
fn system_two_components_immutable(world: &World, query: QueryDef<(&Counter, &Health)>) {
    let q = world.with_query(query);
    for (counter, health) in q.iter() {
        let _ = (counter.0, health.0);
    }
}

#[system(World)]
fn system_two_components_mutable(world: &mut World, query: QueryDef<(&mut Counter, &mut Health)>) {
    world
        .with_query_mut(query)
        .iter_mut()
        .for_each(|(counter, health)| {
            counter.0 += 1;
            health.0 += 5;
        });
}

#[system(World)]
fn system_two_components_mixed(world: &mut World, query: QueryDef<(&mut Counter, &Health)>) {
    world
        .with_query_mut(query)
        .iter_mut()
        .for_each(|(counter, health)| {
            counter.0 += health.0 as usize;
        });
}

#[system(World)]
fn system_three_components_immutable(world: &World, query: QueryDef<(&Counter, &Health, &Speed)>) {
    let q = world.with_query(query);
    for (counter, health, speed) in q.iter() {
        let _ = (counter.0, health.0, speed.0);
    }
}

#[system(World)]
fn system_three_components_mutable(
    world: &mut World,
    query: QueryDef<(&mut Counter, &mut Health, &mut Speed)>,
) {
    world
        .with_query_mut(query)
        .iter_mut()
        .for_each(|(counter, health, speed)| {
            counter.0 += 1;
            health.0 += 1;
            speed.0 += 1.0;
        });
}

// ============================================================================
// Tests for #[system_for_each(World)]
// ============================================================================

#[system_for_each(World)]
fn for_each_single_mutable(counter: &mut Counter) {
    counter.0 += 1;
}

#[system_for_each(World)]
fn for_each_single_immutable(counter: &Counter) {
    let _ = counter.0;
}

#[system_for_each(World)]
fn for_each_two_components_mutable(counter: &mut Counter, health: &mut Health) {
    counter.0 += 1;
    health.0 += 2;
}

#[system_for_each(World)]
fn for_each_two_components_mixed(counter: &mut Counter, health: &Health) {
    counter.0 += health.0 as usize;
}

#[system_for_each(World)]
fn for_each_three_components(counter: &mut Counter, health: &Health, speed: &Speed) {
    counter.0 += (health.0 as f32 * speed.0) as usize;
}

// ============================================================================
// Tests with resources (reference resources only for system_for_each)
// ============================================================================

#[system(World)]
fn system_with_ref_resource(
    world: &mut World,
    query: QueryDef<(&mut Counter, &Health)>,
    tick: &TickCount,
) {
    world
        .with_query_mut(query)
        .iter_mut()
        .for_each(|(counter, _)| {
            counter.0 += tick.0;
        });
}

#[system(World)]
fn system_with_value_resource(
    world: &mut World,
    query: QueryDef<(&mut Counter, &Health)>,
    increment: usize,
) {
    world
        .with_query_mut(query)
        .iter_mut()
        .for_each(|(counter, _)| {
            counter.0 += increment;
        });
}

#[system(World)]
fn system_with_multiple_resources(
    world: &mut World,
    query: QueryDef<(&mut Counter, &mut Health)>,
    tick: &TickCount,
    dt: &DeltaTime,
    multiplier: i32,
) {
    world
        .with_query_mut(query)
        .iter_mut()
        .for_each(|(counter, health)| {
            counter.0 += tick.0;
            health.0 += (dt.0 * multiplier as f32) as i32;
        });
}

#[system(World)]
fn system_with_mutable_resource(world: &World, query: QueryDef<&Counter>, state: &mut GameState) {
    let q = world.with_query(query);
    let count = q.len();
    for i in 0..count {
        let counter: &Counter = q.at(i).unwrap();
        state.score += counter.0 as i32;
    }
}

#[system_for_each(World)]
fn for_each_with_ref_resource(counter: &mut Counter, tick: &TickCount) {
    counter.0 += tick.0;
}

#[system_for_each(World)]
fn for_each_with_value_resource(counter: &mut Counter, tick: &TickCount) {
    counter.0 += tick.0;
}

#[system_for_each(World)]
fn for_each_with_multiple_ref_resources(
    counter: &mut Counter,
    health: &mut Health,
    tick: &TickCount,
    dt: &DeltaTime,
) {
    counter.0 += tick.0;
    health.0 += dt.0 as i32;
}

// ============================================================================
// Selective update systems
// ============================================================================

#[system(World)]
fn increment_only_tag_a(world: &mut World, query: QueryDef<(&mut Counter, &TagA)>) {
    world
        .with_query_mut(query)
        .iter_mut()
        .for_each(|(counter, _)| {
            counter.0 += 100;
        });
}

#[system(World)]
fn increment_only_tag_b(world: &mut World, query: QueryDef<(&mut Counter, &TagB)>) {
    world
        .with_query_mut(query)
        .iter_mut()
        .for_each(|(counter, _)| {
            counter.0 += 200;
        });
}

#[system(World)]
fn increment_with_health(world: &mut World, query: QueryDef<(&mut Counter, &Health)>) {
    world
        .with_query_mut(query)
        .iter_mut()
        .for_each(|(counter, _)| {
            counter.0 += 1;
        });
}

// Helper query for verification
make_query!(QueryCounter, Counter);

make_query!(QueryCounterHealth, Counter, Health);

make_query!(QueryCounterHealthSpeed, Counter, Health, Speed);

make_query!(QueryEntityCounter, Entity, Counter);

make_query!(QueryEntityCounterTagA, Entity, Counter, TagA);

make_query!(QueryEntityCounterTagB, Entity, Counter, TagB);

make_query!(QueryEntityCounterName, Entity, Counter, Name);

make_query!(QueryEntityCounterHealth, Entity, Counter, Health);

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_system_single_component_queries() {
    let mut world = World::default();

    world.create(EntityA {
        counter: Counter(0),
        health: Health(100),
        tag_a: TagA,
    });

    // Test immutable access
    world.system_single_component_immutable();

    // Test mutable access
    world.system_single_component_mutable();

    // Verify the counter was incremented
    let query = world.with_query(Query::<QueryCounter>::new());
    let QueryCounter(counter) = query.at(0).unwrap();
    assert_eq!(counter.0, 1);
}

#[test]
fn test_system_two_component_queries() {
    let mut world = World::default();

    world.create(EntityA {
        counter: Counter(10),
        health: Health(50),
        tag_a: TagA,
    });

    world.system_two_components_immutable();
    world.system_two_components_mutable();

    let query = world.with_query(Query::<QueryCounterHealth>::new());
    let QueryCounterHealth(counter, health) = query.at(0).unwrap();
    assert_eq!(counter.0, 11);
    assert_eq!(health.0, 55);
}

#[test]
fn test_system_two_component_mixed_mutability() {
    let mut world = World::default();

    world.create(EntityA {
        counter: Counter(0),
        health: Health(7),
        tag_a: TagA,
    });

    world.system_two_components_mixed();

    let query = world.with_query(Query::<QueryCounterHealth>::new());
    let QueryCounterHealth(counter, health) = query.at(0).unwrap();
    assert_eq!(counter.0, 7);
    assert_eq!(health.0, 7);
}

#[test]
fn test_system_three_component_queries() {
    let mut world = World::default();

    world.create(EntityAB {
        counter: Counter(1),
        health: Health(10),
        speed: Speed(2.5),
        name: Name("test".to_string()),
    });

    world.system_three_components_immutable();
    world.system_three_components_mutable();

    let query = world.with_query(Query::<QueryCounterHealthSpeed>::new());
    let QueryCounterHealthSpeed(counter, health, speed) = query.at(0).unwrap();
    assert_eq!(counter.0, 2);
    assert_eq!(health.0, 11);
    assert_eq!(speed.0, 3.5);
}

#[test]
fn test_for_each_single_component() {
    let mut world = World::default();

    world.create(EntityA {
        counter: Counter(0),
        health: Health(100),
        tag_a: TagA,
    });

    world.for_each_single_immutable();
    world.for_each_single_mutable();
    world.for_each_single_mutable();
    world.for_each_single_mutable();

    let query = world.with_query(Query::<QueryCounter>::new());
    let QueryCounter(counter) = query.at(0).unwrap();
    assert_eq!(counter.0, 3);
}

#[test]
fn test_for_each_two_components() {
    let mut world = World::default();

    world.create(EntityA {
        counter: Counter(5),
        health: Health(20),
        tag_a: TagA,
    });

    world.for_each_two_components_mutable();
    world.for_each_two_components_mutable();

    let query = world.with_query(Query::<QueryCounterHealth>::new());
    let QueryCounterHealth(counter, health) = query.at(0).unwrap();
    assert_eq!(counter.0, 7);
    assert_eq!(health.0, 24);
}

#[test]
fn test_for_each_with_mixed_mutability() {
    let mut world = World::default();

    world.create(EntityA {
        counter: Counter(0),
        health: Health(5),
        tag_a: TagA,
    });

    world.for_each_two_components_mixed();

    let query = world.with_query(Query::<QueryCounterHealth>::new());
    let QueryCounterHealth(counter, health) = query.at(0).unwrap();
    assert_eq!(counter.0, 5);
    assert_eq!(health.0, 5);
}

#[test]
fn test_for_each_three_components() {
    let mut world = World::default();

    world.create(EntityAB {
        counter: Counter(0),
        health: Health(10),
        speed: Speed(2.0),
        name: Name("entity".to_string()),
    });

    world.for_each_three_components();

    let query = world.with_query(Query::<QueryCounter>::new());
    let QueryCounter(counter) = query.at(0).unwrap();
    assert_eq!(counter.0, 20); // 0 + (10 * 2.0)
}

#[test]
fn test_system_with_ref_resource() {
    let mut world = World::default();

    world.create(EntityA {
        counter: Counter(0),
        health: Health(100),
        tag_a: TagA,
    });

    let tick = TickCount(3);
    world.system_with_ref_resource(&tick);
    world.system_with_ref_resource(&tick);

    let query = world.with_query(Query::<QueryCounter>::new());
    let QueryCounter(counter) = query.at(0).unwrap();
    assert_eq!(counter.0, 6);
}

#[test]
fn test_system_with_value_resource() {
    let mut world = World::default();

    world.create(EntityA {
        counter: Counter(10),
        health: Health(100),
        tag_a: TagA,
    });

    world.system_with_value_resource(5);
    world.system_with_value_resource(7);

    let query = world.with_query(Query::<QueryCounter>::new());
    let QueryCounter(counter) = query.at(0).unwrap();
    assert_eq!(counter.0, 22);
}

#[test]
fn test_system_with_multiple_resources() {
    let mut world = World::default();

    world.create(EntityA {
        counter: Counter(0),
        health: Health(100),
        tag_a: TagA,
    });

    let tick = TickCount(2);
    let dt = DeltaTime(0.5);
    world.system_with_multiple_resources(&tick, &dt, 10);

    let query = world.with_query(Query::<QueryCounterHealth>::new());
    let QueryCounterHealth(counter, health) = query.at(0).unwrap();
    assert_eq!(counter.0, 2);
    assert_eq!(health.0, 105); // 100 + (0.5 * 10)
}

#[test]
fn test_system_with_mutable_resource() {
    let mut world = World::default();

    world.create(EntityA {
        counter: Counter(10),
        health: Health(100),
        tag_a: TagA,
    });
    world.create(EntityA {
        counter: Counter(20),
        health: Health(100),
        tag_a: TagA,
    });

    let mut state = GameState {
        paused: false,
        score: 0,
    };

    world.system_with_mutable_resource(&mut state);

    assert_eq!(state.score, 30);
}

#[test]
fn test_for_each_with_ref_resource() {
    let mut world = World::default();

    world.create(EntityA {
        counter: Counter(0),
        health: Health(100),
        tag_a: TagA,
    });

    let tick = TickCount(5);
    world.for_each_with_ref_resource(&tick);

    let query = world.with_query(Query::<QueryCounter>::new());
    let QueryCounter(counter) = query.at(0).unwrap();
    assert_eq!(counter.0, 5);
}

#[test]
fn test_for_each_with_multiple_ref_resources() {
    let mut world = World::default();

    world.create(EntityA {
        counter: Counter(0),
        health: Health(100),
        tag_a: TagA,
    });

    let tick = TickCount(3);
    let dt = DeltaTime(7.0);
    world.for_each_with_multiple_ref_resources(&tick, &dt);

    let query = world.with_query(Query::<QueryCounterHealth>::new());
    let QueryCounterHealth(counter, health) = query.at(0).unwrap();
    assert_eq!(counter.0, 3);
    assert_eq!(health.0, 107);
}

#[test]
fn test_selective_system_calls() {
    let mut world = World::default();

    let entity_a = world.create(EntityA {
        counter: Counter(0),
        health: Health(100),
        tag_a: TagA,
    });

    let entity_b = world.create(EntityB {
        counter: Counter(0),
        speed: Speed(1.0),
        tag_b: TagB,
    });

    let entity_ab = world.create(EntityAB {
        counter: Counter(0),
        health: Health(100),
        speed: Speed(1.0),
        name: Name("both".to_string()),
    });

    // Increment TagA entities 5 times
    for _ in 0..5 {
        world.increment_only_tag_a();
    }

    // Increment TagB entities 3 times
    for _ in 0..3 {
        world.increment_only_tag_b();
    }

    // Check entity A (has TagA)
    let query_a = world.with_query(Query::<QueryEntityCounterTagA>::new());
    for QueryEntityCounterTagA(entity, counter, _) in query_a.iter() {
        if *entity == entity_a {
            assert_eq!(counter.0, 500); // 5 * 100
        }
    }

    // Check entity B (has TagB)
    let query_b = world.with_query(Query::<QueryEntityCounterTagB>::new());
    for QueryEntityCounterTagB(entity, counter, _) in query_b.iter() {
        if *entity == entity_b {
            assert_eq!(counter.0, 600); // 3 * 200
        }
    }

    // EntityAB has neither TagA nor TagB, so should remain 0
    let query_ab = world.with_query(Query::<QueryEntityCounterName>::new());
    for QueryEntityCounterName(entity, counter, _) in query_ab.iter() {
        if *entity == entity_ab {
            assert_eq!(counter.0, 0);
        }
    }
}

#[test]
fn test_system_called_multiple_times_affects_only_target() {
    let mut world = World::default();

    world.create(EntityA {
        counter: Counter(0),
        health: Health(100),
        tag_a: TagA,
    });

    world.create(EntityB {
        counter: Counter(0),
        speed: Speed(1.0),
        tag_b: TagB,
    });

    // Call system that only affects entities with Health component
    for _ in 0..5 {
        world.increment_with_health();
    }

    // Check EntityA (has Health) - should be affected
    let query_a = world.with_query(Query::<QueryEntityCounterTagA>::new());
    let QueryEntityCounterTagA(_, counter, _) = query_a.at(0).unwrap();
    assert_eq!(counter.0, 5);

    // Check EntityB (no Health) - should not be affected
    let query_b = world.with_query(Query::<QueryEntityCounterTagB>::new());
    let QueryEntityCounterTagB(_, counter, _) = query_b.at(0).unwrap();
    assert_eq!(counter.0, 0);
}

#[test]
fn test_entity_deletion_and_verification() {
    let mut world = World::default();

    let entity_keep = world.create(EntityA {
        counter: Counter(100),
        health: Health(100),
        tag_a: TagA,
    });

    let entity_delete = world.create(EntityA {
        counter: Counter(200),
        health: Health(50),
        tag_a: TagA,
    });

    let entity_keep2 = world.create(EntityB {
        counter: Counter(300),
        speed: Speed(1.0),
        tag_b: TagB,
    });

    // Verify we have 3 entities
    let query_all = world.with_query(Query::<QueryCounter>::new());
    assert_eq!(query_all.len(), 3);

    // Delete the middle entity
    world.destroy(entity_delete);

    // Verify we now have 2 entities
    let query_all = world.with_query(Query::<QueryCounter>::new());
    assert_eq!(query_all.len(), 2);

    // Verify the correct entities remain with correct values
    let query_a = world.with_query(Query::<QueryEntityCounterTagA>::new());
    assert_eq!(query_a.len(), 1);
    let QueryEntityCounterTagA(entity, counter, _) = query_a.at(0).unwrap();
    assert_eq!(*entity, entity_keep);
    assert_eq!(counter.0, 100);

    let query_b = world.with_query(Query::<QueryEntityCounterTagB>::new());
    assert_eq!(query_b.len(), 1);
    let QueryEntityCounterTagB(entity, counter, _) = query_b.at(0).unwrap();
    assert_eq!(*entity, entity_keep2);
    assert_eq!(counter.0, 300);
}

#[test]
fn test_delete_multiple_entities_and_verify_correct_ones() {
    let mut world = World::default();

    let e1 = world.create(EntityA {
        counter: Counter(1),
        health: Health(10),
        tag_a: TagA,
    });

    let e2 = world.create(EntityA {
        counter: Counter(2),
        health: Health(20),
        tag_a: TagA,
    });

    let e3 = world.create(EntityA {
        counter: Counter(3),
        health: Health(30),
        tag_a: TagA,
    });

    let e4 = world.create(EntityA {
        counter: Counter(4),
        health: Health(40),
        tag_a: TagA,
    });

    let e5 = world.create(EntityA {
        counter: Counter(5),
        health: Health(50),
        tag_a: TagA,
    });

    // Delete e2 and e4
    world.destroy(e2);
    world.destroy(e4);

    // Verify correct count
    let query = world.with_query(Query::<QueryEntityCounterHealth>::new());
    assert_eq!(query.len(), 3);

    // Verify correct entities remain
    let mut found = std::collections::HashSet::new();
    for QueryEntityCounterHealth(entity, counter, health) in query.iter() {
        found.insert(*entity);

        match counter.0 {
            1 => {
                assert_eq!(*entity, e1);
                assert_eq!(health.0, 10);
            }
            3 => {
                assert_eq!(*entity, e3);
                assert_eq!(health.0, 30);
            }
            5 => {
                assert_eq!(*entity, e5);
                assert_eq!(health.0, 50);
            }
            _ => panic!("Unexpected counter value: {}", counter.0),
        }
    }

    assert!(found.contains(&e1));
    assert!(!found.contains(&e2));
    assert!(found.contains(&e3));
    assert!(!found.contains(&e4));
    assert!(found.contains(&e5));
}

#[test]
fn test_combination_queries_resources_and_deletion() {
    let mut world = World::default();

    // Create entities
    let e1 = world.create(EntityA {
        counter: Counter(0),
        health: Health(100),
        tag_a: TagA,
    });

    let e2 = world.create(EntityB {
        counter: Counter(0),
        speed: Speed(2.0),
        tag_b: TagB,
    });

    let e3 = world.create(EntityAB {
        counter: Counter(0),
        health: Health(50),
        speed: Speed(1.5),
        name: Name("hybrid".to_string()),
    });

    // Apply systems with resources
    let tick = TickCount(2);
    world.system_with_ref_resource(&tick);

    // Apply selective system
    world.increment_only_tag_a();

    // Check values before deletion
    let query = world.with_query(Query::<QueryEntityCounter>::new());
    for QueryEntityCounter(entity, counter) in query.iter() {
        if *entity == e1 {
            assert_eq!(counter.0, 102); // 0 + 2 (from system_with_ref_resource) + 100 (from increment_only_tag_a)
        } else if *entity == e2 {
            assert_eq!(counter.0, 0); // 0 + 0 (no Health, so not affected by system_with_ref_resource; no TagA)
        } else if *entity == e3 {
            assert_eq!(counter.0, 2); // 0 + 2 (from system_with_ref_resource; no TagA or TagB)
        }
    }

    // Delete e2
    world.destroy(e2);

    // Apply more updates
    world.system_with_value_resource(10);

    // Verify final state
    let query = world.with_query(Query::<QueryEntityCounter>::new());
    assert_eq!(query.len(), 2);

    for QueryEntityCounter(entity, counter) in query.iter() {
        if *entity == e1 {
            assert_eq!(counter.0, 112); // 102 + 10 (from system_with_value_resource)
        } else if *entity == e3 {
            assert_eq!(counter.0, 12); // 2 + 10 (from system_with_value_resource)
        } else {
            panic!("Unexpected entity found");
        }
    }
}

#[test]
fn test_system_and_for_each_equivalence() {
    let mut world1 = World::default();
    let mut world2 = World::default();

    // Create identical entities in both worlds
    for i in 0..5 {
        world1.create(EntityA {
            counter: Counter(i),
            health: Health(100),
            tag_a: TagA,
        });
        world2.create(EntityA {
            counter: Counter(i),
            health: Health(100),
            tag_a: TagA,
        });
    }

    // Apply system in world1
    world1.system_single_component_mutable();

    // Apply for_each in world2
    world2.for_each_single_mutable();

    // Verify results are the same
    let query1 = world1.with_query(Query::<QueryCounter>::new());
    let query2 = world2.with_query(Query::<QueryCounter>::new());

    assert_eq!(query1.len(), query2.len());

    for i in 0..query1.len() {
        let QueryCounter(counter1) = query1.at(i).unwrap();
        let QueryCounter(counter2) = query2.at(i).unwrap();
        assert_eq!(counter1.0, counter2.0);
        assert_eq!(counter1.0, i + 1);
    }
}
