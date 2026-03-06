- Note that the x and y parameters are assumed to be 0-based in this book. That is to say, x may be anywhere from 0 to width - 1 (inclusive), and y may be anywhere from 0 to height - 1 (inclusive).
- Indexing: Pixels are stored in row-major order, so (x, y) maps to y * width + x.
Bounds: The code assumes x < width and y < height. You can add debug_assert! or explicit checks if you want.
red.clone(): Needed because write_pixel takes ownership of Color; Color already derives Clone.
