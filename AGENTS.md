# AGENTS.md

## Cursor Cloud specific instructions

This is a single Rust crate (`ray_tracing_challenge_rs`): a CPU ray tracer/3D renderer. There are no long-running services, servers, databases, or ports — the "product" is a set of CLI scene binaries in `src/bin/` that render an image to a `.ppm` file, plus a `ppm_to_png` binary that converts `.ppm` to `.png`.

### Toolchain
- The crate uses **edition 2024**, which requires Rust **≥ 1.85**. The base VM image may default to an older toolchain (e.g. 1.83), which fails with `feature 'edition2024' is required`. The startup update script installs and defaults to `stable`, so builds should work out of the box. If you hit the edition2024 error, run `rustup default stable`.

### Standard commands
- Build: `cargo build`
- Test (matches CI in `.github/workflows/rust-tests.yml`): `cargo test --all-targets --all-features`
- Lint: `cargo clippy --all-targets --all-features` (note: some scene binaries emit pre-existing clippy warnings; they are not errors)

### Running / rendering (gotcha)
- Scene binaries write to `media/images_ppm/<name>.ppm`, but this directory is **not** committed (`*.ppm` is gitignored). Create it first or the render panics: `mkdir -p media/images_ppm`.
- Render a scene, then convert to PNG:
  ```bash
  mkdir -p media/images_ppm
  cargo run --release --bin checker_sphere
  cargo run --release --bin ppm_to_png -- media/images_ppm/checker_sphere.ppm media/images/checker_sphere.png
  ```
- Use `--release` for renders; some scenes are compute-heavy and are very slow in the default debug profile.

### Render benchmark / regression test
- `tests/render_benchmark.rs` renders a fixed sphere scene via the public API, times the pipeline from `camera.render` through `canvas_to_ppm`, and compares the output pixel-by-pixel against the committed golden file `tests/fixtures/benchmark_sphere.ppm`.
- The golden `.ppm` is force-added because `*.ppm` is gitignored. After an *intentional* change to rendered output, regenerate it with `UPDATE_REFERENCE=1 cargo test --release --test render_benchmark`.
- Measure timing (harness hides stdout otherwise) with: `cargo test --release --test render_benchmark -- --nocapture`. Always use `--release` for meaningful numbers.
