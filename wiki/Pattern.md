#trait #abstraction

Surface color patterns applied via [[Material]].

### Required methods
[[pattern_data]] — returns shared pattern state (transform + inverse)
[[pattern_data_mut]] — mutable access to shared pattern state
[[pattern_at]] — color at a point already in pattern space

### Default methods
[[pattern_at_shape]] — maps a world point through shape + pattern transforms, then calls pattern_at
[[pattern.transform |transform]] — returns the pattern’s transform matrix
[[pattern.set_transform|set_transform]] — sets the transform and caches its inverse

### Implementors
[[StripePattern]] · [[GradientPattern]] · [[RingPattern]] · [[CheckersPattern]]
