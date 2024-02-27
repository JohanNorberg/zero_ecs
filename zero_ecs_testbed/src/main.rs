#![allow(dead_code)]

use zero_ecs::{component, entity, system};

#[component]
struct FlowerComponent;

#[entity]
struct FlowerEntity {
    flower_component: FlowerComponent,
}

#[system]
fn print_names() {}

fn main() {
    println!("Hello, world!");
}
