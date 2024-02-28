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
}

#[system]
fn print_positions(mut query: Query<&mut Position>) {}

#[system]
fn apply_velocity(mut query: Query<(&mut Position, &Velocity)>) {}
fn main() {
    println!("Hello, world!");
}
