# Groups and reusable assemblies

`Group` is a `Shape` that owns child shapes. Import it with:

```rust
use ray_tracing_challenge_rs::group::Group;
use ray_tracing_challenge_rs::shape::Shape;
```

Create it with `Group::new()`, add children using
`group.add_child(Box::new(child))`, then transform the entire assembly with
`group.set_transform(...)`. `set_transform` takes a `Matrix` **by value**:
use `group.set_transform(translation(...))` for one transform, or
`group.set_transform(&translation(...) * &scaling(...))` for a composition.
Never pass `&translation(...)` or multiply unborrowed matrices. Child
transforms are local to the group, while the group transform places, rotates,
or scales the whole assembly in the world.
Finally add the group with `world.add_shape(group)`.

Do **not** try to clone a `Group`: it owns `Box<dyn Shape>` children and is not
cloneable. To make symmetric repeated assemblies (legs, arms, wheels, windows),
write a factory function that builds a fresh group every time, then call it
with different outer positions.

```rust
use ray_tracing_challenge_rs::canvas::Color;
use ray_tracing_challenge_rs::cylinder::Cylinder;
use ray_tracing_challenge_rs::group::Group;
use ray_tracing_challenge_rs::shape::Shape;
use ray_tracing_challenge_rs::transformation::{scaling, translation};

fn make_leg(x: f64) -> Group {
    let mut upper = Cylinder::new();
    upper.minimum = 0.0;
    upper.maximum = 1.0;
    upper.closed = true;
    upper.set_transform(&translation(0.0, -0.5, 0.0) * &scaling(0.14, 0.75, 0.14));
    upper.data.material.color = Color::new(0.2, 0.25, 0.35);

    let mut lower = Cylinder::new();
    lower.minimum = 0.0;
    lower.maximum = 1.0;
    lower.closed = true;
    lower.set_transform(&translation(0.0, -1.7, 0.0) * &scaling(0.12, 0.75, 0.12));
    lower.data.material.color = Color::new(0.2, 0.25, 0.35);

    let mut leg = Group::new();
    leg.add_child(Box::new(upper));
    leg.add_child(Box::new(lower));
    leg.set_transform(translation(x, 1.5, 0.0));
    leg
}

// Each call creates independent child shapes and positions the entire leg.
world.add_shape(make_leg(-0.32));
world.add_shape(make_leg(0.32));
```

Use the same pattern for arms: define `make_arm(side: f64) -> Group`, use
`side` to mirror its group transform, and call it once for each side.
