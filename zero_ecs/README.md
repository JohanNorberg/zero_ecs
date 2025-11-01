# Zero ECS

Zero ECS is an Entity Component System that is written with 4 goals
1. Only use zero cost abstractions - no use of dyn and Box and stuff [zero-cost-abstractions](https://doc.rust-lang.org/beta/embedded-book/static-guarantees/zero-cost-abstractions.html).
2. No use of unsafe rust code.
3. Be very user friendly. The user should write as little boilerplate as possible.
4. Be very fast

It achieves this by generating all code at compile time, using a combination of macros and build scripts.

## Instructions

... Todo
