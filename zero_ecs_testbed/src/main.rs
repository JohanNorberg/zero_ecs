#![allow(dead_code)]

// include main_ecs.rs
include!(concat!(env!("OUT_DIR"), "/zero_ecs.rs"));

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
/*
trait QueryFrom<'a, T> {
    fn query_from(&'a mut self) -> impl Iterator<Item = T>;
}
#[derive(Default, Debug)]
struct AQuery<T> {
    phantom: PhantomData<T>,
}
impl<'a, T: 'a> AQuery<T> {
    fn iter_mut(&self, world: &'a mut World) -> impl Iterator<Item = T> + 'a
    where
        World: QueryFrom<'a, T>,
    {
        world.query_from()
    }
}
struct Query<'a, T> {
    a_query: AQuery<T>,
    world: &'a mut World,
}
struct World {}*/
