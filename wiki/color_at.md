#independent-function 

Intersects a [[Ray]] with the [[World]], and returns black on a miss or the shaded color of the nearest hit.

It ties the [[intersect]],[[prepare_computations]] and [[shade_hit]] functions together. It will intersect the world with the given Ray and then return the color for that intersection. 

```rust
pub fn color_at(world: &World, ray: &Ray, remaining: i32) -> Color {
    let xs = world.intersect_world(ray);
    match xs.hit() {
        None => Color::new(0.0, 0.0, 0.0),
        Some(hit) => {
            let comps = prepare_computations(hit, ray, &xs, &|id| world.resolve_shape(id));
            shade_hit(world, &comps, remaining)
        }
    }
}
```

1. Call intersect_world to find the intersections of the given ray with the given world.
2. Find the hit from the resulting intersections.
3. Return the color black if there is no such intersection.
4. Otherwise, precompute the necessary values with prepare_computations.
5. Finally, call shade_hit to find the color at the hit.