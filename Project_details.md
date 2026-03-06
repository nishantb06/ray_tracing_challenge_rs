- Note that the x and y parameters are assumed to be 0-based in this book. That is to say, x may be anywhere from 0 to width - 1 (inclusive), and y may be anywhere from 0 to height - 1 (inclusive).
- Indexing: Pixels are stored in row-major order, so (x, y) maps to y * width + x.
Bounds: The code assumes x < width and y < height. You can add debug_assert! or explicit checks if you want.
red.clone(): Needed because write_pixel takes ownership of Color; Color already derives Clone.
- 
## Canvas overview

### Layout

- **`width`** – number of columns (horizontal)
- **`height`** – number of rows (vertical)
- **`pixels`** – `Vec<Color>` in row-major order: `pixels[y * width + x]` = pixel at `(x, y)`

### Coordinate system

- **`x`** – column index, 0 to `width - 1` (left → right)
- **`y`** – row index, 0 to `height - 1` (top → bottom)

So **(0, 0) is the top-left corner**, not bottom-left.

### Why top-left?

1. The PPM spec says “the first row of pixels comes first, then the second row, and so forth,” which matches the usual image convention (top row first).
2. The index `y * width + x` puts row 0 at the start of the array.
3. There is no vertical flip in the PPM output.

### Visual

```
(0,0) ─────────────────────► x (width)
  │
  │   ┌─────────────────────┐
  │   │  row 0 (y=0)        │
  │   │  row 1 (y=1)        │
  │   │  ...                │
  │   │  row height-1       │
  │   └─────────────────────┘
  ▼
  y (height)
```

### If you need bottom-left origin

Ray tracing often uses a bottom-left origin. To support that, flip `y` when writing or reading:

```rust
// Convert from bottom-left y to top-left row index
let row = self.height - 1 - y;
```
