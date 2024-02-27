#![allow(dead_code)]

use zero_ecs::{component, entity};

#[component]
struct FlowerComponent;

#[entity]
struct FlowerEntity {
    flower_component: FlowerComponent,
}

fn my_test() {
    println!("hej");
}

fn main() {
    my_test();
    println!("Hello, world!");
}
