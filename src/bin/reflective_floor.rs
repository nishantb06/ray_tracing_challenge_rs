use ray_tracing_challenge_rs::scenes;

fn main() {
    let (camera, world) = scenes::reflective_floor::build(3840, 2160);
    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/reflective_floor.ppm", ppm)
        .expect("Failed to write reflective_floor.ppm");
    println!("Saved to media/images_ppm/reflective_floor.ppm");
}
