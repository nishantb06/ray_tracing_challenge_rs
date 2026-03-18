use image::{ImageBuffer, Rgb};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.len() > 3 {
        return Err(format!(
            "Usage: {} <input.ppm> [output.png]",
            args.first().map_or("ppm_to_png", String::as_str)
        ));
    }

    let input = PathBuf::from(&args[1]);
    let output = if args.len() == 3 {
        PathBuf::from(&args[2])
    } else {
        default_output_path(&input)
    };

    let ppm_text = fs::read_to_string(&input)
        .map_err(|e| format!("failed to read '{}': {e}", input.display()))?;

    let (width, height, max_value, values) = parse_p3(&ppm_text)?;
    let rgb_bytes = to_rgb_bytes(max_value, values)?;

    let image = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(width, height, rgb_bytes)
        .ok_or_else(|| "failed to build image buffer from parsed data".to_string())?;

    image
        .save(&output)
        .map_err(|e| format!("failed to save '{}': {e}", output.display()))?;

    println!("Converted '{}' -> '{}'", input.display(), output.display());
    Ok(())
}

fn default_output_path(input: &Path) -> PathBuf {
    input.with_extension("png")
}

fn parse_p3(ppm: &str) -> Result<(u32, u32, u32, Vec<u32>), String> {
    let mut tokens = Vec::new();
    for line in ppm.lines() {
        let before_comment = line.split('#').next().unwrap_or("");
        tokens.extend(before_comment.split_whitespace().map(ToOwned::to_owned));
    }

    if tokens.len() < 4 {
        return Err("PPM file is too short".to_string());
    }

    if tokens[0] != "P3" {
        return Err(format!(
            "unsupported PPM magic '{}'; only ASCII P3 is supported",
            tokens[0]
        ));
    }

    let width = parse_u32(&tokens[1], "width")?;
    let height = parse_u32(&tokens[2], "height")?;
    let max_value = parse_u32(&tokens[3], "max value")?;
    if max_value == 0 {
        return Err("max value must be > 0".to_string());
    }

    let expected_values = width
        .checked_mul(height)
        .and_then(|px| px.checked_mul(3))
        .ok_or_else(|| "image dimensions are too large".to_string())? as usize;

    let mut values = Vec::with_capacity(expected_values);
    for token in &tokens[4..] {
        values.push(parse_u32(token, "pixel channel")?);
    }

    if values.len() != expected_values {
        return Err(format!(
            "pixel count mismatch: expected {expected_values} channel values, got {}",
            values.len()
        ));
    }

    Ok((width, height, max_value, values))
}

fn parse_u32(token: &str, field: &str) -> Result<u32, String> {
    token
        .parse::<u32>()
        .map_err(|_| format!("invalid {field}: '{token}'"))
}

fn to_rgb_bytes(max_value: u32, values: Vec<u32>) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(values.len());

    for value in values {
        if value > max_value {
            return Err(format!(
                "pixel channel value {value} exceeds max value {max_value}"
            ));
        }

        let scaled = if max_value == 255 {
            value
        } else {
            ((value as f64 / max_value as f64) * 255.0).round() as u32
        };
        out.push(scaled.min(255) as u8);
    }

    Ok(out)
}
