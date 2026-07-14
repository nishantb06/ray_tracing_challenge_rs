use ray_tracing_challenge_rs::scenes;

fn main() {
    let (camera, world) = scenes::single_glass_sphere::build(1920, 1080);
    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/single_glass_sphere.ppm", ppm)
        .expect("Failed to write single_glass_sphere.ppm");
    println!("Saved to media/images_ppm/single_glass_sphere.ppm");
}
