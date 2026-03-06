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