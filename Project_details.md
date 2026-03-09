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
Translation 
| 1  0  0  x |
| 0  1  0  y |
| 0  0  1  z |
| 0  0  0  1 |

Scaling 
| x  0  0  0 |
| 0  y  0  0 |
| 0  0  z  0 |
| 0  0  0  1 |
scaling moves it by multiplication. When applied to an object centered at the origin, this transformation scales all points on the object, effectively making it larger (if the scale value is greater than 1) or smaller (if the scale value is less than 1), as shown in the figure.

 Reflection is a transformation that takes a point and reflects it—moving it to the other side of an axis. It can be useful when you have an object in your scene that you want to flip (or mirror) in some direction. Maybe the model is leaning the wrong way, facing the wrong direction. Maybe it’s a face that’s looking to the right when you want it looking to the left. Rather than breaking out a 3D modeler and editing the model, you can simply reflect the model across the appropriate axis.

 Rotation
 Multiplying a tuple by a rotation matrix will rotate that tuple around an axis. This can get complicated if you’re trying to rotate around an arbitrary line, so we’re not going to take that route. We’re only going to deal with the simplest rotations here—rotating around the x, y, and z axes.

 The rotation will appear to be clockwise around the corresponding axis when viewed along that axis, toward the negative end. So, if you’re rotating around the x axis, it will rotate as depicted in the following figure.

 Another way to describe this is to say that rotations in your ray tracer will
obey the left-hand rule, which harks back to Left-Handed vs. Right-Handed Coordinates, on page 3: if you point the thumb of your left hand in the direction of the axis of rotation, then the rotation itself will follow the direction of your remaining fingers as you curl them toward the palm of your hand.

Each of the three axes requires a different matrix to implement the rotation, so we’ll look at them each in turn. Angles will be given in radians, so if your math library prefers other units (like degrees), you’ll need to adapt accordingly.

This first rotation matrix rotates a tuple some number of radians around the x axis,
X                           
| 1    0       0      0 |   
| 0    cos(r) -sin(r) 0 |
| 0    sin(r)  cos(r) 0 |
| 0    0       0      1 |

Y
|  cos(r)  0  sin(r)  0 |
|  0       1  0       0 |
| -sin(r)  0  cos(r)  0 |
|  0       0  0       1 |

Z
| cos(r) -sin(r)  0  0 |
| sin(r)  cos(r)  0  0 |
| 0       0       1  0 |
| 0       0       0  1 |

A shearing (or skew) transformation has the effect of making straight lines slanted. It’s probably the most (visually) complex transformation that we’ll implement, though the implementation is no more complicated than any of the others.
When applied to a tuple, a shearing transformation changes each component of the tuple in proportion to the other two components. So the x component changes in proportion to y and z, y changes in proportion to x and z, and z changes in proportion to x and y.
The following illustration shows how this works in two dimensions. Specifically, note how differently the same transformation affects each point in x as the y component changes.

This is what “changing in proportion” means: the farther the y coordinate is from zero, the more the x value changes.
In three dimensions each component may be affected by either of the other two components, so there are a total of six parameters that may be used to define the shear transformation:
• x in proportion to y • x in proportion to z • y in proportion to x • y in proportion to z • z in proportion to x • z in proportion to y
Write the following tests, demonstrating how a point is affected by each of these parameters. In each, notice how the coordinate being moved moves by the amount of the other coordinate. For instance, in this first test x is initially 2, but moving x in proportion to y adds 1 times y (or 3) to x (2) and produces a new x of 5.

Chaining
So, if you want a single matrix that rotates, and then scales, and then translates, you can multiply the translation matrix by the scaling matrix, and then by the rotation matrix. That is to say, you must concatenate the transformations in reverse order to have them applied in the order you want! Add the following tests to demonstrate this (particularly counterintuitive) result.


TODO operator overloading is not implemented for tuple 