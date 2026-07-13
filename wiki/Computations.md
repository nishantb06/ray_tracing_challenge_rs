#struct 

```rust
pub struct Computations<'a> {

	pub t: f64,
	pub object: &'a dyn Shape,
	pub point: Tuple,
	pub eye_vector: Tuple,
	pub normal_vector: Tuple,
	pub inside: bool,
	pub over_point: Tuple,
	pub reflectv: Tuple,
	pub n1: f64,
	pub n2: f64,
	pub under_point: Tuple,
}
```

This object is returned by the [[prepare_computations]] function

- inside indicates whether the intersection is inside the shape or not.