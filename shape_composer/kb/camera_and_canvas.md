# Camera and canvas

Create a scene with `let mut world = World::new();`, add a point light through
`world.lights = vec![PointLight::new(Tuple::point(-10.0, 10.0, -10.0),
Color::new(1.0, 1.0, 1.0))];`, then create
`Camera::new(400, 400, FRAC_PI_3)`. The camera dimensions are the render
canvas dimensions.

Use `camera.set_transform(view_transform(&Tuple::point(from_x, from_y, from_z),
&Tuple::point(to_x, to_y, to_z), &Tuple::vector(0.0, 1.0, 0.0)))`.
Render with `let canvas = camera.render(&world);` and persist with
`std::fs::write("PATH.ppm", canvas.canvas_to_ppm())?; println!("Saved to PATH.ppm");`.
Create parents before writing the file if the output path needs it.
