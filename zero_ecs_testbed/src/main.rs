use zero_ecs::just_for_test;

#[just_for_test]
fn my_test() {
    println!("hej");
}

fn main() {
    my_test();
    println!("Hello, world!");
}
