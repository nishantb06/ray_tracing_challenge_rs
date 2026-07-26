# Primitive shapes

Import the concrete shape, `Shape`, `Color`, `Tuple`, transforms, `World`, `Camera`,
and `PointLight`. `Sphere::new()`, `Cube::new()`, `Plane::new()`,
`Cylinder::new()`, and `Cone::new()` create unit primitives centered at the
origin. Cubes extend from -1 to 1 on each axis, so scale a cube by half of the
desired dimensions.

Every primitive implements `Shape`: use `shape.set_transform(&translation(x,y,z)
* &scaling(x,y,z))` and edit `shape.data.material.color`, `diffuse`, `specular`,
or `shininess`. Cylinders and cones are infinite until setting their public
`minimum`, `maximum`, and `closed` fields. Add primitives with
`world.add_shape(shape)`.

For assemblies use `Group::new()` and `group.add_child(Box::new(shape))`.
