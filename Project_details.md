- Note that the x and y parameters are assumed to be 0-based in this book. That is to say, x may be anywhere from 0 to width - 1 (inclusive), and y may be anywhere from 0 to height - 1 (inclusive).
- Indexing: Pixels are stored in row-major order, so (x, y) maps to y \* width + x.
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
| 1 0 0 x |
| 0 1 0 y |
| 0 0 1 z |
| 0 0 0 1 |

Scaling
| x 0 0 0 |
| 0 y 0 0 |
| 0 0 z 0 |
| 0 0 0 1 |
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
| 1 0 0 0 |  
| 0 cos(r) -sin(r) 0 |
| 0 sin(r) cos(r) 0 |
| 0 0 0 1 |

Y
| cos(r) 0 sin(r) 0 |
| 0 1 0 0 |
| -sin(r) 0 cos(r) 0 |
| 0 0 0 1 |

Z
| cos(r) -sin(r) 0 0 |
| sin(r) cos(r) 0 0 |
| 0 0 1 0 |
| 0 0 0 1 |

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

object_point ← inverse(transform) \* world_point

With that point now in object space, you can compute the normal as before, because in object space, the sphere’s origin is at the world’s origin. However! The normal vector you get will also be in object space...and to draw anything useful with it you’re going to need to convert it back to world space somehow.

So how do you go about keeping the normals perpendicular to their surface? The answer is to multiply the normal by the inverse transpose matrix instead. So you take your transformation matrix, invert it, and then transpose the result. This is what you need to multiply the normal by.
world_normal ← transpose(inverse(transform)) \* object_normal

Technically, you should be finding submatrix(transform, 3, 3) (from Spotting Submatrices, on page 34) first, and multiplying by the inverse and transpose of that. Otherwise, if your transform includes any kind of translation, then multiplying by its transpose will wind up mucking with the w coordinate in your vector, which will wreak all kinds of havoc in later computations. But if you don’t mind a bit of a hack, you can avoid all that by just setting world_normal.w to 0 after multiplying by the 4x4 inverse transpose matrix.

The inverse transpose matrix may change the length of your vector, so if you feed it a vector of length 1 (a normalized vector), you may not get a normalized vector out! It’s best to be safe, and always normalize the result.

It simulates the interaction between three different types of lighting:
• Ambient reflection is background lighting, or light reflected from other objects in the environment. The Phong model treats this as a constant, coloring all points on the surface equally.
• Diffuse reflection is light reflected from a matte surface. It depends only on the angle between the light source and the surface normal.
• Specular reflection is the reflection of the light source itself and results in what is called a specular highlight—the bright spot on a curved surface. It depends only on the angle between the reflection vector and the eye vector and is controlled by a parameter that we’ll call shininess. The higher the shininess, the smaller and tighter the specular highlight.

prepare_computations() precomputes the point (in world space) where the intersection occurred, the eye vector (pointing back toward the eye, or camera), and the normal vector.new data structure encapsulating some precomputed information relating to the intersection.

the eye vector (pointing back toward the eye, or camera)

Add the following two tests, which show that prepare_computations() sets a fourth attribute, inside, which will be true if the hit occurs inside the object, and false otherwise. Notice, too, that the normal is inverted when the intersection is inside an object, so that the surface may be illuminated properly. Take the dot product of the two vectors, and if the result is nega- tive, they’re pointing in (roughly) opposite directions.

To calculate the total number of line
find src -name "\*.rs" | xargs wc -l

View transformation

- pretends the eye moves instead of the world
- the view transformation is actually moving the world with respect to the eye
- Note that the up vector doesn’t need to be normalized. In fact, it doesn’t even need to be exactly perpendicular to the viewing direction. As you’ll see shortly, the view_transform() function will tidy that up vector, so you only have to point vaguely in the direction you want. Isn’t that convenient?

The camera is defined by the following four attributes:
• hsize is the horizontal size (in pixels) of the canvas that the picture will be rendered to.
• vsize is the canvas’s vertical size (in pixels).
• field_of_view is an angle that describes how much the camera can see. When the field of view is small, the view will be “zoomed in,” magnifying a smaller area of the scene.
• transform is a matrix describing how the world should be oriented relative to the camera. This is usually a view transformation like you implemented in the previous section.

One of the primary responsibilities of the camera is to map the three-dimen- sional scene onto a two-dimensional canvas. To do this, you’ll make the camera do just what you’ve done in previous exercises and place the canvas somewhere in the scene so that rays can be projected through it. But contrary to what you’ve done before, the camera’s canvas will always be exactly one unit in front of the camera. As you’ll see shortly, this makes the math a bit cleaner.

You’ll use the pixel_size and those half_width and half_height values you computed to create rays that can pass through any given pixel on the canvas. Implement the following three tests to ensure this works. These introduce a new function, ray_for_pixel(camera, x, y), which returns a new ray that starts at the camera and passes through the indicated (x, y) pixel on the canvas. The first two tests use an untransformed camera to cast rays through the center and corner of the canvas, and the third tries a ray with a camera that has been translated and rotated.

A ray tracer computes shadows by casting a ray, called a shadow ray, from each point of intersection toward the light source. If something intersects that shadow ray between the point and the light source, then the point is considered to be in shadow. You’re going to write a new function, is_shadowed(world, point), which will do just this.

Note that the test compares the over_point’s z component to half of -EPSILON to
make sure the point has been adjusted in the correct direction.
In pseudocode, your prepare_computations() function will need to do something like this:

# after computing and (if appropriate) negating

# the normal vector...

comps.over_point ← comps.point + comps.normalv \* EPSILON

This effect is called acne, and it happens because computers cannot represent floating point numbers very precisely. In general they do okay, but because of rounding errors, it will be impossible to say exactly where a ray intersects a surface. The answer you get will be close—generally within a tiny margin of error—but that wiggle is sometimes just enough to cause the calculated point of intersection to lie beneath the actual surface of the sphere.

As a result, the shadow ray intersects the sphere itself, causing the sphere to cast a shadow on its own point of intersection. This is obviously not ideal.
The solution is to adjust the point just slightly in the direction of the normal, before you test for shadows. This will bump it above the surface and prevent self-shadowing.

1. Measure the distance from point to the light source by subtracting point from the light position, and taking the magnitude of the resulting vector. Call this distance.
2. Create a ray from point toward the light source by normalizing the vector from step 1.
3. Intersect the world with that ray.
4. Check to see if there was a hit, and if so, whether t is less than distance. If
   so, the hit lies between the point and the light source, and the point is in shadow

Because the point being passed to the stripe_at() function is in world space,
the patterns completely ignore the transformations of the objects to which
they are applied.
This is unfortunate, because we expect a pattern to move when its object
moves. If you make an object bigger or smaller, the pattern on it should get
bigger or smaller. Rotating an object ought to rotate the pattern, too.
Further, it makes sense to be able to transform the patterns themselves,
independently of the object. Want your stripes closer together or farther apart?
Scale them. Want to change how they are oriented on the object? Rotate them.
What to change their phase? Translate them to shift them to one side or the
other.

he good news is that every pattern will be essentially
the same, differentiated only by the function that converts points into colors.
Besides that function, every pattern will have a transformation matrix, and
every pattern will need to use it to help transform a given point from world
space to pattern space before producing a color.

If you take this route, use the following tests as guidelines for writing your
own. These tests assume that the abstract function (the one that transforms
the point and delegates to the concrete function) is called pattern_at_shape(pattern,
shape, point). The concrete function (to be implemented by each pattern) is here
simply called pattern_at(pattern, point).


As mentioned, n1 and n2 are the names given to the refractive indices of the
materials on either side of a ray-object intersection, with n1 belonging to the
material being exited, and n2 belonging to the material being entered.

# Important notes for the reflection and refraction guidelines 
Ray tracers are best known for mirrors and glass. Take some time and
experiment, to see why. Here are a few tips for figuring out how to employ
reflection and refraction effectively in your scenes.
1. 2. 3. 4. We tend to think of glass as exclusively transparent, but no one is sur-
prised to look in a window and see their own ghostly reflection superim-
posed over the scene. When rendering glass or any similar material, set
both transparency and reflectivity to high values, 0.9 or even 1. This allows
the Fresnel effect to kick in, and gives your material an added touch of
realism!
Because the reflected and refracted colors are added to the surface color,
they’ll tend to make such objects brighter. You can tone down the mate-
rial’s diffuse and ambient properties to compensate. The more transparent
or reflective the surface, the smaller the diffuse property should be. This
way, more of the color comes from the secondary rays, and less from the
object’s surface.
If you’d like a subtly colored mirror, or slightly tinted glass, use a very
dark color, instead of a very light one. Red glass, for instance, should use
a very dark red, almost black, instead of a very bright red. In general, the
more reflective or transparent the surface, the darker its surface color
should be. Note that if you add color, make sure that you have some diffuse
and possibly ambient contribution, too; otherwise, your surface will render
as black regardless of what color you give to it.
Reflective and transparent surfaces pair nicely with tight specular high-
lights. Set specular to 1 and bump shininess to 300 or more to get a highlight
that really shines.
Also, here’s a closing challenge for you: suppose you wanted to render a scene
where you were looking through the surface of a pond at some rocks beneath
it. In terms of implementation, that would be a transparent plane, with some
spheres scattered below it. As your ray tracer is currently implemented, the
plane is going to cast a shadow on anything beneath it, which leaves everything
under the water in darkness, ruining the effect. You could add a light source
beneath the plane, but that will introduce odd shadows and highlights—not
a good solution either.
What you really want is for some objects to “opt out” of the shadow calculation.
The surface of the pond, for instance, should be ignored when calculating
shadows.
How would you go about changing your ray tracer to support that? What
would you need to do to allow objects to individually declare that they cast
no shadow?
Chew on that one for a bit. When you’re ready to move on, turn the page! Next
up, you’ll add another primitive shape to your ray tracer: the humble cube.