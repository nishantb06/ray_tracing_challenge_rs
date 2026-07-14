use ray_tracing_challenge_rs::scenes;

fn main() {
    let (camera, world) = scenes::group_hexagon::build(1000, 1000);
    let canvas = camera.render(&world);
    let ppm = canvas.canvas_to_ppm();
    std::fs::write("media/images_ppm/group_hexagon.ppm", ppm)
        .expect("Failed to write media/images_ppm/group_hexagon.ppm");
    println!("Saved to media/images_ppm/group_hexagon.ppm");
}
