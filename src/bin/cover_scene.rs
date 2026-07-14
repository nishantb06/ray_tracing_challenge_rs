use ray_tracing_challenge_rs::scenes;

fn main() {
    let (camera, world) = scenes::cover_scene::build(2000, 2000);
    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/cover_scene.ppm", ppm)
        .expect("Failed to write media/images_ppm/cover_scene.ppm");
    println!("Saved to media/images_ppm/cover_scene.ppm");
}
