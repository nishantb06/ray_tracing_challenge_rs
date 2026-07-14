use ray_tracing_challenge_rs::scenes;

fn main() {
    let (camera, world) = scenes::football::build(900, 700);
    let path = "media/images_ppm/football.ppm";
    std::fs::write(path, camera.render(&world).canvas_to_ppm()).expect("write ppm");
    println!("Saved to {}", path);
}
