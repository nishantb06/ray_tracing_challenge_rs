| Expression        | How it's Parsed           | What Actually Runs                                  |
|-------------------|--------------------------|-----------------------------------------------------|
| `a1.add(&a2)`     | Receiver: `a1`           | `(&a1).add(&a2)` (a1 is automatically referenced)   |
| `&a1.add(&a2)`    | `&(a1.add(&a2))`         | Calls `a1.add(&a2)`, then references the result     |

Both expressions use the same `Add` implementation. The `&` in `&a1.add(&a2)` only applies to the result, not to the way the method is called.
To make the receiver explicitly a reference, use parentheses like this: `(&a1).add(&a2)`.

-------------------


Rust doesn’t have class-style inheritance, but you can model “Point and Vector as Tuple-like types” in a few ways.

## 1. Type aliases (same type, different names)

```rust
pub type Point = Tuple;
pub type Vector = Tuple;
```

- `Point` and `Vector` are just names for `Tuple`.
- No extra type safety: you can mix them freely.
- Useful mainly for readability.

---

## 2. Newtype pattern (distinct types)

```rust
pub struct Point(Tuple);
pub struct Vector(Tuple);
```

- `Point` and `Vector` are separate types that wrap a `Tuple`.
- You get type safety (e.g. you can’t add two `Point`s by mistake).
- You need to define how to create and use them:

```rust
impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Point(Tuple::point(x, y, z))
    }
}

impl Vector {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vector(Tuple::vector(x, y, z))
    }
}

// Access inner tuple: point.0.x or add a getter
impl Vector {
    pub fn x(&self) -> f64 { self.0.x }
    pub fn y(&self) -> f64 { self.0.y }
    pub fn z(&self) -> f64 { self.0.z }
}
```

---

## 3. Shared behavior via traits

```rust
pub trait TupleLike {
    fn x(&self) -> f64;
    fn y(&self) -> f64;
    fn z(&self) -> f64;
    fn w(&self) -> f64;
}

impl TupleLike for Tuple { ... }
impl TupleLike for Point { ... }
impl TupleLike for Vector { ... }
```

- `Point` and `Vector` can implement the same trait without inheriting from `Tuple`.

---

## 4. Ray Tracing Challenge style

In this context, the usual approach is:

- A single `Tuple` type.
- `point(x, y, z)` → `Tuple` with `w = 1`.
- `vector(x, y, z)` → `Tuple` with `w = 0`.

So you only need `Tuple` plus constructors.

---

## Recommendation

- If you want simple, minimal code: keep `Tuple` and use `point()` / `vector()` constructors.
- If you want stronger types: use the newtype pattern with `Point(Tuple)` and `Vector(Tuple)` and `impl` blocks for each.


----------------
----------------


To forbid adding two points at **compile time**, you need distinct types so the compiler can reject invalid combinations.

## Approach: Separate `Point` and `Vector` types

Use the newtype pattern and implement `Add` only for valid combinations:

| Left  | Right | Result  | Valid? |
|-------|-------|---------|--------|
| Point | Vector| Point   | Yes    |
| Vector| Point | Point   | Yes    |
| Vector| Vector| Vector | Yes    |
| Point | Point | —      | No     |

## Implementation sketch

```rust
// In tuple.rs or a new point_vector.rs

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point(Tuple);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector(Tuple);

impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Point(Tuple::point(x, y, z))
    }
}

impl Vector {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vector(Tuple::vector(x, y, z))
    }
}

// Point + Vector = Point
impl Add<Vector> for Point {
    type Output = Point;
    fn add(self, rhs: Vector) -> Point {
        Point(self.0 + rhs.0)
    }
}

// Vector + Point = Point
impl Add<Point> for Vector {
    type Output = Point;
    fn add(self, rhs: Point) -> Point {
        Point(self.0 + rhs.0)
    }
}

// Vector + Vector = Vector
impl Add for Vector {
    type Output = Vector;
    fn add(self, rhs: Self) -> Vector {
        Vector(self.0 + rhs.0)
    }
}

// Point + Point is NOT implemented - won't compile!
```

## Usage

```rust
let p1 = Point::new(1.0, 2.0, 3.0);
let p2 = Point::new(4.0, 5.0, 6.0);
let v = Vector::new(1.0, 0.0, 0.0);

let p3 = p1 + v;   // OK: Point + Vector
let v2 = v + v;    // OK: Vector + Vector
let bad = p1 + p2; // ERROR: no Add impl for Point + Point
```

## If you keep a single `Tuple` type

With one `Tuple` type, the compiler cannot tell points from vectors, so you can only enforce this at runtime:

```rust
impl Add for Tuple {
    type Output = Result<Tuple, &'static str>;
    fn add(self, rhs: Self) -> Self::Output {
        if self.w == 1.0 && rhs.w == 1.0 {
            Err("Cannot add two points")
        } else {
            Ok(Tuple { ... })
        }
    }
}
```

That forces callers to handle `Result` and can fail at runtime instead of compile time.

**Recommendation:** Use separate `Point` and `Vector` types and implement `Add` only for the valid combinations so invalid additions are rejected by the compiler.

---------------------
---------------------

Implementing both gives you flexibility in how you call negation, but they are not interchangeable in terms of ownership.

## Difference

| Implementation | Call | What happens |
|----------------|------|--------------|
| `impl Neg for Tuple` | `-t` | `t` is moved into `neg()` and consumed |
| `impl Neg for &Tuple` | `-&t` | `t` is borrowed; it stays valid afterward |

## When each is used

**`Neg for Tuple`** – for owned values:
```rust
let t = Tuple::vector(1.0, 2.0, 3.0);
let negated = -t;   // t is moved, gone
// t is no longer usable here
```

**`Neg for &Tuple`** – when you want to keep the original:
```rust
let t = Tuple::vector(1.0, 2.0, 3.0);
let negated = -&t;  // t is borrowed
// t is still usable here
```

## Do you need both?

Usually one is enough:

- If `Tuple` is `Copy`, `impl Neg for Tuple` is enough: `-t` copies `t` into `neg()`, and the original is still usable because of `Copy`.
- If `Tuple` is not `Copy`, `impl Neg for &Tuple` is often better so you can write `-&t` without moving.

## Summary

- Both produce the same `Tuple` result.
- They differ in whether the input is consumed or borrowed.
- With `Copy`, `Neg for Tuple` alone is typically sufficient.
- Without `Copy`, `Neg for &Tuple` is usually the better choice so you can keep the original value.

---------------------
---------------------

`hypot` computes the length of a vector in a way that avoids overflow and underflow better than the naive formula.

## What is `hypot`?

`hypot` stands for **hypotenuse**. For two numbers it computes:

```
hypot(x, y) = √(x² + y²)
```

In Rust, `f64::hypot(self, other)` does this for two floats. For more dimensions you chain it:

```rust
x.hypot(y).hypot(z).hypot(w)  // √(x² + y² + z² + w²)
```

## Why the naive formula can fail

Naive approach:

```rust
(x*x + y*y + z*z).sqrt()
```

Problems:

### 1. Overflow

If components are large, squaring can overflow before `sqrt`:

```rust
let x = 1e200_f64;
x * x  // 1e400 — overflows f64 (max ~1.8e308)
```

So `x² + y² + z²` can become `inf`, and the result is wrong.

### 2. Underflow

If components are very small, squaring can underflow to 0:

```rust
let x = 1e-200_f64;
x * x  // 1e-400 — underflows to 0
```

Then the computed length is too small or zero.

## How `hypot` improves this

`hypot` uses a scaled formulation. For two values:

1. Take the larger magnitude: `m = max(|x|, |y|)`
2. If `m == 0`, return 0
3. Otherwise compute: `m * √((x/m)² + (y/m)²)`

Because `|x/m| ≤ 1` and `|y/m| ≤ 1`, their squares stay in a safe range, so you avoid overflow and reduce underflow.

## Example

```rust
// Large values — naive can overflow
let x = 1e200;
let y = 1e200;

// Naive: x² + y² overflows
// let naive = (x*x + y*y).sqrt();  // inf

// hypot: scales first, stays finite
let stable = x.hypot(y);  // ~1.414e200 ✓
```

## When it matters

- For typical ray-tracing values (roughly -1000 to 1000), the naive formula is usually fine.
- `hypot` matters when:
  - Components can be very large (e.g. far from origin)
  - Components can be very small (e.g. near zero)
  - You want more robust behavior across a wide range of scales

## Summary

| Approach | Formula | Overflow/underflow risk |
|----------|---------|---------------------------|
| Naive    | `√(x² + y² + z²)` | Higher for extreme values |
| `hypot`  | Scaled computation | Lower, more stable |

For most ray tracers, `(x*x + y*y + z*z).sqrt()` is acceptable; `hypot` is useful when you care about numerical robustness across a wide range of magnitudes.

---------------------
---------------------

Here are practical ways to cache magnitude and normalized vector in Rust.

## 1. Interior mutability with `Cell` and `RefCell`

Use `Cell` for `f64` and `RefCell` for `Tuple`:

```rust
use std::cell::{Cell, RefCell};

pub struct Tuple {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
    magnitude_cache: Cell<Option<f64>>,
    normalized_cache: RefCell<Option<Tuple>>,
}

impl Tuple {
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self {
            x, y, z, w,
            magnitude_cache: Cell::new(None),
            normalized_cache: RefCell::new(None),
        }
    }

    pub fn magnitude(&self) -> f64 {
        if let Some(m) = self.magnitude_cache.get() {
            return m;
        }
        let m = self.x.hypot(self.y).hypot(self.z).hypot(self.w);
        self.magnitude_cache.set(Some(m));
        m
    }

    pub fn normalize(&self) -> Tuple {
        if let Some(ref n) = *self.normalized_cache.borrow() {
            return n.clone();  // or return copy if Tuple is Copy
        }
        let mag = self.magnitude();
        let n = Tuple {
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
            w: self.w / mag,
        };
        self.normalized_cache.replace(Some(n.clone()));
        n
    }
}
```

You need `Clone` on `Tuple` for `normalized_cache`. If `Tuple` is `Copy`, you can use `Cell<Option<Tuple>>` instead of `RefCell`.

## 2. Separate cached wrapper

Keep `Tuple` unchanged and add a wrapper that caches:

```rust
pub struct CachedVector {
    inner: Tuple,
    magnitude_cache: Cell<Option<f64>>,
    normalized_cache: RefCell<Option<Tuple>>,
}

impl CachedVector {
    pub fn from_tuple(t: Tuple) -> Self {
        Self {
            inner: t,
            magnitude_cache: Cell::new(None),
            normalized_cache: RefCell::new(None),
        }
    }

    pub fn magnitude(&self) -> f64 {
        if let Some(m) = self.magnitude_cache.get() {
            return m;
        }
        let m = self.inner.magnitude();  // or compute here
        self.magnitude_cache.set(Some(m));
        m
    }
    // ...
}
```

## 3. `once_cell` / `std::sync::OnceLock` (Rust 1.70+)

For one-time computation per value:

```rust
use std::sync::OnceLock;

pub struct Tuple {
    pub x: f64, pub y: f64, pub z: f64, pub w: f64,
    magnitude_cache: OnceLock<f64>,
    normalized_cache: OnceLock<Tuple>,
}

impl Tuple {
    pub fn magnitude(&self) -> f64 {
        *self.magnitude_cache.get_or_init(|| {
            self.x.hypot(self.y).hypot(self.z).hypot(self.w)
        })
    }

    pub fn normalize(&self) -> &Tuple {
        self.normalized_cache.get_or_init(|| {
            let mag = self.magnitude();
            Tuple {
                x: self.x / mag,
                y: self.y / mag,
                z: self.z / mag,
                w: self.w / mag,
            }
        })
    }
}
```

`OnceLock` is thread-safe and only computes once. It returns a reference, so you don’t need `Clone` on `Tuple`.

## Trade-offs

| Approach | Pros | Cons |
|----------|------|------|
| `Cell`/`RefCell` in `Tuple` | Simple, per-instance cache | Changes `Tuple`, needs `Clone` for `Tuple` in cache |
| `CachedVector` wrapper | `Tuple` stays unchanged | Extra type, conversion at boundaries |
| `OnceLock` | Thread-safe, no `Clone` | Slightly more overhead, returns `&Tuple` |

## Recommendation

For ray tracing, magnitude and normalization are cheap (a few ops per call). Caching usually isn’t worth the extra complexity unless profiling shows these as hot spots.

If you do cache:

- Use `OnceLock` if you’re on Rust 1.70+ and want a clean API.
- Use `Cell`/`RefCell` if you need to keep compatibility with older Rust or want owned `Tuple` from `normalize()`.
---------------------
---------------------
They're Rust methods on integer types that prevent overflow/underflow by clamping at the type's bounds.

### `saturating_add`

Adds two numbers, but if the result would overflow, it returns the maximum value instead of panicking or wrapping.

```rust
let x: usize = usize::MAX;
x.saturating_add(1) // returns usize::MAX, not panic/wrap
```

### `saturating_sub`

Subtracts, but if the result would go below zero (for unsigned types like `usize`), it returns `0` instead of panicking.

```rust
let x: usize = 0;
x.saturating_sub(1) // returns 0, not panic

let y: usize = 5;
y.saturating_sub(3) // returns 2 (normal subtraction)
```

### Why I used them in the clock program

In this line:

```32:33:src/main.rs
                let x = px.saturating_add(dx).saturating_sub(1).min(size - 1);
                let y = py.saturating_add(dy).saturating_sub(1).min(size - 1);
```

We're drawing a 3x3 block of pixels around each hour marker. `dx` and `dy` go from 0 to 2, and we subtract 1 to center the block (offsets become -1, 0, +1). Since `px` and `py` are `usize` (unsigned), subtracting 1 when the value is 0 would underflow and panic. `saturating_sub` prevents that by clamping to 0. Similarly, `.min(size - 1)` clamps the upper bound so we don't write outside the canvas.

### Alternatives

Rust also has:
- `wrapping_add` / `wrapping_sub` — wraps around (like C unsigned arithmetic)
- `checked_add` / `checked_sub` — returns `Option<T>`, `None` on overflow
- `saturating_*` — clamps at bounds (what we used)
---------------------
---------------------
All 6 tests pass. Here's a summary of the changes:

**Unique ID** — each `Sphere::new()` call gets a unique `id` via a global atomic counter:

```1:6:src/sphere.rs
use std::sync::atomic::{AtomicU64, Ordering};
// ...
static NEXT_SPHERE_ID: AtomicU64 = AtomicU64::new(1);
```

```20:26:src/sphere.rs
    pub fn new() -> Self {
        Sphere {
            id: NEXT_SPHERE_ID.fetch_add(1, Ordering::Relaxed),
            center: Tuple::point(0.0, 0.0, 0.0),
            radius: 1.0,
        }
    }
```

**5 new tests added:**

| Test | Ray origin | Behavior | Expected t values |
|------|-----------|----------|-------------------|
| `a_sphere_has_unique_id` | — | Two spheres get different IDs | `s1.id != s2.id` |
| `a_ray_intersects_a_sphere_at_a_tangent` | `(0, 1, -5)` | Grazes the top edge | `5.0, 5.0` |
| `a_ray_misses_a_sphere` | `(0, 2, -5)` | Passes above entirely | empty (count = 0) |
| `a_ray_originates_inside_a_sphere` | `(0, 0, 0)` | Starts at center | `-1.0, 1.0` |
| `a_sphere_is_behind_a_ray` | `(0, 0, 5)` | Sphere is behind the ray | `-6.0, -4.0` |

The tangent case returns two identical `t` values (discriminant = 0), and rays originating inside or past the sphere correctly produce negative `t` values — the intersection math doesn't clip to positive `t`, which will be useful later for CSG operations.
---------------------
---------------------
std::mem::swap(&mut t1, &mut t2); what is this 
use std::sync::atomic::{AtomicU64, Ordering}; look more into this as well

Here's what's going on:

### `AtomicU64`

`AtomicU64` is a thread-safe integer. A normal `static mut u64` counter would require `unsafe` code and wouldn't be safe across threads. `AtomicU64` lets you read and modify the value from any thread without a mutex — the CPU guarantees the operation is indivisible (atomic), so no two threads can see a half-updated value.

### `fetch_add(1, Ordering::Relaxed)`

This does two things in one atomic step:
1. **Returns** the current value of the counter.
2. **Increments** the counter by 1.

So the sequence of `Sphere::new()` calls produces:

| Call # | `fetch_add` returns | Counter becomes |
|--------|-------------------|-----------------|
| 1st    | **1**             | 2               |
| 2nd    | **2**             | 3               |
| 3rd    | **3**             | 4               |
| ...    | ...               | ...             |

The first sphere gets `id = 1`, the second gets `id = 2`, and so on. Every sphere is guaranteed a unique ID.

### `Ordering::Relaxed`

The `Ordering` parameter controls how this atomic operation interacts with **other** memory operations (reads/writes to other variables) across threads. There are several levels:

- **`Relaxed`** — the weakest. It only guarantees that the `fetch_add` itself is atomic (no torn reads/writes). It makes **no** guarantees about the ordering of other memory operations relative to this one.
- **`Acquire` / `Release`** — creates a happens-before relationship. Useful when the atomic guards access to shared data (like a lock).
- **`SeqCst`** — the strongest. All threads see all `SeqCst` operations in the same global order.

`Relaxed` is the right choice here because the sphere ID counter is self-contained — we don't use it to synchronize access to any other shared data. We only need the increment itself to be atomic (i.e., no two calls return the same value), and `Relaxed` guarantees that. Using a stronger ordering would add unnecessary overhead for no benefit.

### In summary

It's essentially a global auto-increment counter that starts at 1. Each `Sphere::new()` atomically grabs the next number and bumps the counter, so no two spheres ever share an ID — even if created from different threads.
---------------------
---------------------
### Why lifetimes are needed here

`Intersection` stores a **reference** (`&Sphere`) rather than owning the sphere. This is efficient — you don't clone the sphere for every intersection — but it creates a fundamental question for the Rust compiler: *how long does the Sphere behind that reference live?* The lifetime `'a` is the answer.

### Walking through each piece

**1. The struct definition**

```5:8:src/intersection.rs
pub struct Intersection<'a> {
    pub t: f64,
    pub object: &'a Sphere,
}
```

`'a` is a **lifetime parameter** on the struct. It says: "an `Intersection<'a>` holds a reference to a `Sphere` that is guaranteed to live for at least `'a`." The struct itself cannot outlive `'a`, because if it did, `object` would be a dangling pointer.

Think of `'a` as a label that represents some span of time during program execution. The compiler uses it to ensure the `Sphere` isn't dropped while an `Intersection` still points to it.

**2. The constructor**

```17:21:src/intersection.rs
impl<'a> Intersection<'a> {
    pub fn new(t: f64, object: &'a Sphere) -> Self {
        Intersection { t, object }
    }
}
```

- `impl<'a>` declares the lifetime parameter for the entire impl block.
- `object: &'a Sphere` in the function signature says: "the reference you pass in must live for `'a`."
- The returned `Self` is `Intersection<'a>`, which inherits the same lifetime — so the returned intersection can't outlive the sphere reference you passed in.

**3. Intersections wraps a Vec of them**

```12:14:src/intersection.rs
pub struct Intersections<'a> {
    pub data: Vec<Intersection<'a>>,
}
```

The lifetime propagates upward. An `Intersections<'a>` contains many `Intersection<'a>` values, and *all of them* borrow spheres that must live for at least `'a`. The entire collection is invalid once any referenced sphere is dropped.

**4. Where `'a` gets its concrete meaning — `Sphere::intersect`**

```27:27:src/sphere.rs
    pub fn intersect<'a>(&'a self, ray: &Ray) -> Intersections<'a> {
```

This is where it all comes together. The signature says:
- `&'a self` — borrow the sphere for some lifetime `'a`
- `-> Intersections<'a>` — the returned intersections live for that same `'a`

So the compiler links them: **the `Intersections` you get back can't outlive the `&self` borrow of the sphere**. Inside the body, `Intersection::new(t1, self)` passes `self` (which is `&'a Sphere`) as the object, and the types all line up.

### What the compiler actually prevents

Consider what would happen without lifetimes:

```rust
let xs;
{
    let s = Sphere::new();
    let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
    xs = s.intersect(&r);
}  // s is dropped here
// xs.data[0].object would be a dangling pointer!
println!("{}", xs.data[0].object.id);  // use-after-free
```

The lifetime `'a` on `intersect(&'a self) -> Intersections<'a>` makes the compiler reject this. It sees that `xs` (which is `Intersections<'a>`) must not outlive the borrow `&'a self`, but `s` is dropped at the closing brace while `xs` is used after — compile error.

### How it works in the tests

```rust
let s = Sphere::new();                          // s lives until end of test
let i = Intersection::new(3.5, &s);             // i borrows s — 'a = rest of test
assert_eq!(i.object.id, s.id);                  // fine, s is still alive
```

Both `s` and `i` live until the end of the function. The compiler infers `'a` to be the overlapping region where both are alive, and everything checks out.

### In short

| Piece | What `'a` means |
|-------|----------------|
| `Intersection<'a>` | "I hold a reference to a Sphere that lives for at least `'a`" |
| `Intersections<'a>` | "All my Intersection records borrow Spheres that live for at least `'a`" |
| `intersect(&'a self) -> Intersections<'a>` | "The result borrows me (`self`), so it can't outlive me" |

The lifetime is the compiler's proof that no `Intersection` will ever hold a dangling reference to a destroyed `Sphere`. You get the efficiency of references (no cloning) with the safety guarantee at compile time — zero runtime cost.
---------------------
---------------------
Don’t let that last test trip you up! The intersections are intentionally given in random order; it’s up to your intersections() function to maintain a sorted list or, at the very least, sort the list on demand. This will be important down the road when you have more complicated scenes with multiple objects. It won’t be feasible for each shape to manually preserve the sort order of that intersec- tion list. We might need to implement a more efficient data structure to track the hits like a Binary indexed Tree or Segment tree which can keep the hits sorted 
---------------------
---------------------
in tracking hits commit , they added an Option and a corresponding unwrap, come back to it later . 
---------------------
---------------------
so Transformation T was supposed to be done to the sphere , but we put the inverse of the that transformation to the ray instead , does that mean we bought the ray into the object space from the world space ?
Exactly right. Here's the intuition:

### World space vs Object space

- **World space** is where everything lives together — rays, spheres, lights, etc., all positioned in the same coordinate system.
- **Object space** is the sphere's "local" coordinate system. In object space, the sphere is always a unit sphere at the origin — no scaling, no translation, no rotation applied.

### The equivalence

Say you have a sphere scaled by 2x. You could either:

1. **Transform the sphere** into world space (make it radius 2) and intersect with the original ray, or
2. **Transform the ray** into object space (shrink it by the inverse — scale by 0.5) and intersect with the simple unit sphere at the origin.

Both give the same `t` values because the relative geometry is identical. If you double the sphere, the ray hits it at certain points. If instead you halve the ray's coordinates, the relationship between the ray and the unit sphere is exactly the same.

### Why do it this way?

The unit sphere intersection math is simple and always the same — you already wrote it. It only knows about a sphere of radius 1 at the origin. Rather than making that math more complex to handle arbitrary transforms, you "undo" the sphere's transform on the ray, bringing it into the coordinate system where the sphere *is* just a unit sphere at the origin.

That's what this line does:

```rust
let inv = self.transform.inverse_gauss_jordan();
let local_ray = ray.transform(&inv);
```

- `self.transform` moves the sphere from object space **to** world space.
- `self.transform.inverse()` moves from world space **back to** object space.
- Applying that inverse to the ray brings the ray from world space into the sphere's object space.

Then the intersection math runs on `local_ray` against the plain unit sphere — exactly the code you already have. The resulting `t` values are valid in world space too, because `t` is a scalar parameter along the ray's direction, and the transformation preserves the parametric relationship.

### Visual example

Your scaled sphere test: sphere scaled `(2, 2, 2)`, ray starts at `(0, 0, -5)`.

- **Approach 1 (conceptual):** Make the sphere radius 2, intersect the original ray. Hits at `t=3` and `t=7`.
- **Approach 2 (what the code does):** Apply `scaling(0.5, 0.5, 0.5)` to the ray, getting origin `(0, 0, -2.5)` and direction `(0, 0, 0.5)`. Intersect with unit sphere. The quadratic gives `t=3` and `t=7` — same answer.

This "inverse transform the ray" trick is the standard approach in ray tracers. It keeps the intersection code clean and works for any transformation — translation, rotation, scaling, or any combination.
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------
---------------------
---------------------

---------------------
---------------------

---------------------
---------------------