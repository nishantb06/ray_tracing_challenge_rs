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


When rendering your scene, you’ll need to be able to identify which one of all the intersections is actually visible from the ray’s origin. Some may be behind the ray, and others may be hidden behind (or occluded by) other objects. For the sake of discussion, we’ll call the visible intersection the hit. This is really the only intersection that matters for most things.

The hit will never be behind the ray’s origin, since that’s effectively behind the camera, so you can ignore all intersections with negative t values when determining the hit. In fact, the hit will always be the intersection with the lowest nonnegative t value.

Don’t let that last test trip you up! The intersections are intentionally given in random order; it’s up to your intersections() function to maintain a sorted list or, at the very least, sort the list on demand. This will be important down the road when you have more complicated scenes with multiple objects. It won’t be feasible for each shape to manually preserve the sort order of that intersec- tion list. We might need to implement a more efficient data structure to track the hits like a Binary indexed Tree or Segment tree which can keep the hits sorted 

In other words: whatever transformation you want to apply to the sphere, apply the inverse of that transformation to the ray, instead. Crazy, right? But it works!

Another way to think about transformation matrices is to think of them as converting points between two different coordinate systems. At the scene level, everything is in world space coordinates, relative to the overall world. But at the object level, everything is in object space coordinates, relative to the object itself.
Multiplying a point in object space by a transformation matrix converts that point to world space—scaling it, translating, rotating it, or whatever. Multiplying a point in world space by the inverse of the transformation matrix converts that point back to object space.
Want to intersect a ray in world space with a sphere in object space? Just convert the ray’s origin and direction to that same object space, and you’re golden.

Notice how, in the second test, the ray’s direction vector is left unnormalized. This is intentional, and important! Transforming a ray has the effect of (potentially) stretching or shrinking its direction vector. You have to leave that vector with its new length, so that when the t value is eventually comput- ed, it represents an intersection at the correct distance (in world space!) from the ray’s origin.

Chapter 6
The truth is that most ray tracers favor approximations over physically accurate simulations, so that to shade any point, you only need to know four vectors.
If P is where your ray intersects an object, these four vectors are defined as:
• E is the eye vector, pointing from P to the origin of the ray (usually, where
the eye exists that is looking at the scene).
• L is the light vector, pointing from P to the position of the light source.
• N is the surface normal, a vector that is perpendicular to the surface at P.
• R is the reflection vector, pointing in the direction that incoming light would bounce, or reflect.

 first you have to convert the point from world space to object space by multiplying the point by the inverse of the transformation matrix, thus:

 object_point ← inverse(transform) * world_point

 With that point now in object space, you can compute the normal as before, because in object space, the sphere’s origin is at the world’s origin. However! The normal vector you get will also be in object space...and to draw anything useful with it you’re going to need to convert it back to world space somehow.

 So how do you go about keeping the normals perpendicular to their surface? The answer is to multiply the normal by the inverse transpose matrix instead. So you take your transformation matrix, invert it, and then transpose the result. This is what you need to multiply the normal by.
world_normal ← transpose(inverse(transform)) * object_normal

 Technically, you should be finding submatrix(transform, 3, 3) (from Spotting Submatrices, on page 34) first, and multiplying by the inverse and transpose of that. Otherwise, if your transform includes any kind of translation, then multiplying by its transpose will wind up mucking with the w coordinate in your vector, which will wreak all kinds of havoc in later computations. But if you don’t mind a bit of a hack, you can avoid all that by just setting world_normal.w to 0 after multiplying by the 4x4 inverse transpose matrix.

 The inverse transpose matrix may change the length of your vector, so if you feed it a vector of length 1 (a normalized vector), you may not get a normalized vector out! It’s best to be safe, and always normalize the result.

 It simulates the interaction between three different types of lighting:
• Ambient reflection is background lighting, or light reflected from other objects in the environment. The Phong model treats this as a constant, coloring all points on the surface equally.
• Diffuse reflection is light reflected from a matte surface. It depends only on the angle between the light source and the surface normal.
• Specular reflection is the reflection of the light source itself and results in what is called a specular highlight—the bright spot on a curved surface. It depends only on the angle between the reflection vector and the eye vector and is controlled by a parameter that we’ll call shininess. The higher the shininess, the smaller and tighter the specular highlight.

prepare_computations() precomputes the point (in world space) where the intersection occurred, the eye vector (pointing back toward the eye, or camera), and the normal vector.new data structure encapsulating some precomputed information relating to the intersection.

the eye vector (pointing back toward the eye, or camera)