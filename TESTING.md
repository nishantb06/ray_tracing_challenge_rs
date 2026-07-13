1. Speed test for the entire framework (All scenes in the test suite)
`cargo test --release --test render_benchmark -- --nocapture`
2. Unit test for individual modules
`cargo test --lib camera` replace camera with the name of any module like CSG / Group etc
3. Sphere only
`cargo test --release --test render_benchmark benchmark_sphere_render_matches_reference -- --nocapture`
4. Cover scene only (With groups and multiple object, reflection, refraction)
`cargo test --release --test render_benchmark benchmark_cover_scene_render_matches_reference -- --nocapture`
