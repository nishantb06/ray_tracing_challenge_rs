use clap::Parser;
use shape_composer::{run, Mode, RunRequest};
use std::path::PathBuf;

#[derive(Parser)]
#[command(about = "Compose ray-traced scenes from visual feedback")]
struct Cli {
    #[arg(long)]
    goal: String,
    #[arg(long, default_value = "auto")]
    mode: String,
    #[arg(long, default_value_t = 25)]
    max_iterations: u32,
    #[arg(long, default_value = "image.png")]
    reference_image: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mode = match cli.mode.as_str() {
        "auto" => Mode::Auto,
        "hil" => Mode::Hil,
        _ => anyhow::bail!("mode must be auto or hil"),
    };
    let summary = run(RunRequest {
        goal: cli.goal,
        mode,
        max_iterations: cli.max_iterations,
        seed_prompt: None,
        reference_image: cli.reference_image,
    }).await?;
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}
