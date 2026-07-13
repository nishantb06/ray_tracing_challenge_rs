#trait #abstraction

Every new shape must implement this trait.

### Required methods
[[shape_data]] — returns shared shape state (id, transform, material, parent)
[[shape_data_mut]] — mutable access to shared shape state
[[local_intersect]] — intersect a ray already in object space
[[local_normal_at]] — surface normal at a point already in object space

### Default methods
[[intersect]] — transforms the world ray into object space, then calls local_intersect
[[normal_at]] — transforms a world point to object space, gets local normal, maps it back to world
[[id]] — returns the shape’s unique id
[[transform]] — returns the shape’s transform matrix
[[material]] — returns the shape’s material
[[material_mut]] — mutable access to the shape’s material
[[set_transform]] — sets the transform and caches its inverse
[[find_by_id]] — looks up a child by id (default: none; Groups/CSG override)

### Related helpers
[[world_to_object]] — maps a world-space point into object space, walking parents
[[normal_to_world]] — maps an object-space normal into world space, walking parents
[[shape_normal_at]] — parent-aware world normal at a point
[[shape_normal_at_with_hit]] — same as shape_normal_at, but passes hit data (for smooth triangles)

### Implementors
[[Sphere]] · [[Plane]] · [[Cube]] · [[Cylinder]] · [[Cone]] · [[Triangle]] · [[SmoothTriangle]] · [[Groups]] · [[CSG]]
