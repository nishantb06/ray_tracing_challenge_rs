# Multithreading & Caching Optimization Plan

> Audit of `ray_tracing_challenge_rs` — 24 source files scanned + `Cargo.toml`.
> `rayon = "1.11"` is already a dependency but currently used only in `src/camera.rs`.
> Reference pattern: `Camera::render` parallelizes per-pixel rendering with `par_iter_mut()`, and `Camera`/`ShapeData`/`PatternData` all cache `transform_inverse` alongside `transform`.

---

## High Return

### 1. Add a Bounding Volume Hierarchy (BVH)
- **Category:** Both (Multithreading + Caching)
- **Location:** new `src/bounding_box.rs` / `src/bvh.rs`; integration in `src/group.rs:53-60`, `src/world.rs:49-56`, `src/shape.rs`
- **Current code:** `Group::local_intersect` (`src/group.rs:53-60`) and `World::intersect_world` (`src/world.rs:49-56`) iterate every child/primitive sequentially for every ray. The teapot scene (`src/bin/teapot.rs:59`) wraps thousands of triangles in one flat `Group`, so each of the 800x600 = 480k primary rays (+ reflection/refraction sub-rays x depth 5) tests every triangle. No `bounding_box.rs` file exists.
- **Proposed optimization:** Add an Axis-Aligned Bounding Box (AABB) type and a BVH node variant of `Shape`. At scene build time (after `obj_to_group`), compute per-shape world-space AABBs and recursively subdivide into a binary tree. In `local_intersect`, test the ray against the node's AABB first (`ray_aabb_intersect` via the slab method — same math already in `cube.rs:check_axis`) and only descend into children whose AABB is hit. The BVH build is one-time work; traversal is `par_iter`-friendly across siblings. This reduces per-ray work from `O(num_primitives)` to `O(log num_primitives)`.
- **Estimated return:** **High.** For OBJ scenes this is the single largest win — typically a 10x-100x speedup on triangle-heavy scenes (teapot, dodecahedron, cottage, hand).

### 2. ✅ Cache the transpose of `transform_inverse` (the "inverse-transpose") in `ShapeData` (Done)
- **Category:** Caching
- **Location:** `src/shape.rs:13-37` (add field), `src/shape.rs:55-63` (`normal_at`), `src/shape.rs:96-111` (`normal_to_world`)
- **Current code:** Every call to `Shape::normal_at` (`src/shape.rs:60`) executes `&sd.transform_inverse.transpose() * &local_normal` — the `.transpose()` allocates and fills a brand-new `Matrix` (`src/matrix.rs:49-57`, `Vec<Vec<f64>>`), then the multiplication allocates again. `normal_to_world` (`src/shape.rs:101`) does the same on every step up the parent chain. `normal_at` is invoked once per ray hit (via `prepare_computations`/`shape_normal_at_with_hit`), i.e. per pixel x hit count x recursive depth, and again up the parent chain for grouped shapes.
- **Proposed optimization:** Add `pub transform_inverse_transpose: Matrix` to `ShapeData`. Recompute it inside `ShapeData::set_transform` (`src/shape.rs:33-36`) right after `transform_inverse = t.inverse_gauss_jordan()` — i.e. `self.transform_inverse_transpose = self.transform_inverse.transpose();`. Then replace `&sd.transform_inverse.transpose()` in `normal_at` (`src/shape.rs:60`) and `normal_to_world` (`src/shape.rs:101`) with `&sd.transform_inverse_transpose`. This mirrors the existing `transform_inverse` caching pattern and eliminates one matrix allocation + one `Mul<&Matrix> for &Tuple` per step.
- **Estimated return:** **High.** `normal_at` runs at minimum once per pixel and is called recursively up nested groups, so eliminating two allocations per call has a large aggregate impact. Closely matches the existing `transform_inverse` cache pattern, so it's a trivial, mechanical change.

### 3. Parallelize `Group::local_intersect` with rayon
- **Category:** Multithreading
- **Location:** `src/group.rs:53-60` (loop). Also `src/group.rs:1` (add `use rayon::prelude::*;`).
- **Current code:**
  ```rust
  fn local_intersect<'b>(&'b self, ray: &Ray) -> Intersections<'b> {
      let mut all_intersections = Vec::new();
      for shape in &self.shapes {
          let xs = shape.intersect(ray);
          all_intersections.extend(xs.data);
      }
      Intersections::new(all_intersections)
  }
  ```
  Each child intersection is fully independent and produces its own `Intersection<'b>` vector (with lifetimes tied to `&'b self`).
- **Proposed optimization:** Replace with a parallel iterator:
  ```rust
  let all: Vec<Intersection<'b>> = self.shapes
      .par_iter()
      .flat_map(|shape| shape.intersect(ray).data)
      .collect();
  Intersections::new(all)
  ```
  `Intersection<'a>` already requires that the leaf `Shape` be `Sync` (the trait is `Debug + Send + Sync` at `src/shape.rs:40`), and `Intersection` holds only `&'a dyn Shape` plus `Option<f64>`, so the lifetime bounds already admit parallel collection. This is exactly analogous to `Camera::render`'s existing `par_iter_mut().zip(...).for_each(...)` pattern (`src/camera.rs:82-89`).
- **Estimated return:** **High.** `Group` is touched by every ray; for a teapot with thousands of triangles this loop is the dominant inner kernel. A flat parallel map yields roughly an `N_cores` speedup before BVH is added, and still helps post-BVH for the top-level world group.
- This backfired as well 
Baseline : [benchmark] rendered 1000x1000 group hexagon scene to PPM in 798.787ms (1251.9 pixels/ms) 
With optimisation: [benchmark] rendered 1000x1000 group hexagon scene to PPM in 1.724s (580.0 pixels/ms)

### 4. Parallelize `World::intersect_world` with rayon
- **Category:** Multithreading
- **Location:** `src/world.rs:49-56`
- **Current code:**
  ```rust
  pub fn intersect_world(&self, ray: &Ray) -> Intersections<'_> {
      let mut all: Vec<Intersection> = Vec::new();
      for obj in &self.objects {
          let obj_xs = obj.intersect(ray);
          all.extend(obj_xs.data);
      }
      Intersections::new(all)
  }
  ```
  Same shape as `Group::local_intersect` — independent per-object work, gathered into one sorted list.
- **Proposed optimization:**
  ```rust
  let all: Vec<Intersection> = self.objects
      .par_iter()
      .flat_map(|obj| obj.intersect(ray).data)
      .collect();
  Intersections::new(all)
  ```
  Add `use rayon::prelude::*;` at `src/world.rs:1`.
- **Estimated return:** **High.** `intersect_world` is invoked once per `color_at` call and again once per shadow ray (`is_shadowed_light`, `src/world.rs:75`). With 800x600 pixels x up to `MAX_RECURSION_DEPTH = 5` (`src/utils.rs:3`) this is several million calls per frame; an `N_cores` parallel speedup is significant. Combined with #3, every shape-intersection loop in the program becomes parallel.

This Backfired 
For a 2000×2000 cover scene, that's 4M pixel tasks (plus reflection/refraction/shadow rays inside each). The thread pool is already busy.

What the intersect_world change adds

Each color_at / shadow check calls intersect_world, which then tries to par_iter over world objects. The cover scene only has ~19 top-level shapes (1 plane, 1 glass sphere, 17 cubes). Per ray, that inner parallel loop is:

~19 tiny intersection tests
then Rayon scheduling, joining, and collecting into a Vec
That overhead dominates the actual math.

Why ~3× slower (11s → 34s)

Cores already full — pixel parallelism fills the pool; nested par_iter mostly fights for the same workers and adds job-queue churn.
Granularity too fine — parallelizing 19 cheap cube/sphere tests millions of times pays spawn/join cost for almost no speedup.
Call volume is huge — every primary ray, every recursive bounce, every shadow ray (2 lights) hits intersect_world, so the overhead is multiplied across the whole frame.
The optimization plan's “High” estimate assumed intersect_world was a sequential hot loop. That was true before pixel-level Rayon. After camera parallelization, this change is the same class of mistake as “one thread per pixel”: parallelizing the wrong level.

When it would help

A single-threaded render (no camera par_iter), or
Worlds with hundreds/thousands of independent top-level objects per ray, where object work dwarfs scheduling cost
Practical takeaway: keep intersect_world sequential. Parallelize at the outer level (pixels — already done), or at a coarser scene structure (e.g. BVH / large groups), not over a handful of objects inside every ray.



### 5. Early-exit shadow ray testing
- **Category:** Both
- **Location:** `src/world.rs:70-87` (`is_shadowed_light`/`is_shadowed`), `src/world.rs:49-56` (called via `intersect_world`)
- **Current code:** `is_shadowed_light` (`src/world.rs:70-78`) calls `self.intersect_world(&ray)`, which collects **all** intersections across **all** objects into a `Vec<Intersection>` and then **sorts** them by `t` (`Intersections::new`, `src/intersection.rs:135-138: items.sort_by(...)`), then calls `xs.hit()` which scans again to find the first non-negative `t`. All of this work is done just to answer a single boolean: "is anything between `p` and the light?". `is_shadowed` (`src/world.rs:80-87`) then sequentially calls this for each light in a `.all(...)` fold. `shade_hit` (`src/world.rs:90-107`) calls `is_shadowed_light` once per light per hit. So per pixel: `#lights x #hits x #objects` cost just for shadows.
- **Proposed optimization:** Add a dedicated short-circuit shadow test that returns `true` as soon as **any** object yields an intersection with `0 < t < magnitude` — no global sort, no merge, no allocation of a sorted `Intersections`. Use rayon's `par_any`:
  ```rust
  self.objects.par_iter().any(|obj| {
      obj.intersect(ray).data.iter().any(|i|
          i.t > EPSILON && i.t < distance_to_light
      )
  })
  ```
  This also pairs naturally with a future BVH (#1) — early-out the moment one primitive's AABB/t-range proves an occluder exists.
- **Estimated return:** **High.** Shadow testing is currently a noticeable fraction of per-pixel cost (every shade-hit runs one full intersect+sort per light). Short-circuiting removes the sort and stops at the first occluder, often after examining far fewer objects.

---

## Medium-High Return

### 6. Precompute the camera-space ray origin once per `render` call
- **Category:** Caching
- **Location:** `src/camera.rs:53-72` (`ray_for_pixel`), `src/camera.rs:74-92` (`render`)
- **Current code:** `ray_for_pixel` is called once per pixel (`src/camera.rs:87`, inside the `par_iter_mut` loop) and executes:
  ```rust
  let origin = inv * &Tuple::point(0.0, 0.0, 0.0);
  ```
  `Tuple::point(0,0,0)` is the same literal every call, and `inv` (`self.transform_inverse`) doesn't change during `render`, so this matrix-tuple product produces the **same** `origin` for every pixel of the image. That's `hsize x vsize` identical matrix multiplications (e.g. 480k for the 800x600 teapot).
- **Proposed optimization:** Compute `origin` once before the parallel loop in `Camera::render` (e.g. `let cam_origin = &self.transform_inverse * &Tuple::point(0.0, 0.0, 0.0);`) and either pass it into `ray_for_pixel` as an extra argument or clone it per pixel (a `Tuple` is 32 bytes). Equivalently, recognize that `M * point(0,0,0)` is just the last column of `M` and read `inv.data[*][3]` directly without any multiplication.
- **Estimated return:** **Medium-High.** Removes one matrix-tuple product per pixel; for 800x600 this eliminates ~480k matrix-vector products and ~480k small heap allocations (the `Mul<&Tuple> for &Matrix` in `src/matrix.rs:206-223` constructs a fresh `Tuple`). Per-pixel-hot.

### 7. Cache a `HashMap<u64, &dyn Shape>` lookup in `Group` and `World`
- **Category:** Caching
- **Location:** `src/group.rs:69-87` (`Group::find_by_id`), `src/world.rs:58-68` (`World::resolve_shape`)
- **Current code:** `resolve_shape` is called on every `color_at`/`shade_hit` via `prepare_computations` (`src/world.rs:126`, closure `|id| world.resolve_shape(id)`). `resolve_shape` linearly scans `self.objects` and then calls `object.find_by_id(id)` on each, which in `Group::find_by_id` (`src/group.rs:69-87`) recursively **linearly** scans the entire group subtree. For grouped children of a teapot this is `O(depth x siblings)` per lookup, per pixel, per hit. `Group` already stores an `ids: HashSet<u64>` (`src/group.rs:13`) but never uses it to index back to the actual child reference.
- **Proposed optimization:** Maintain a `HashMap<u64, *const dyn Shape>` (or non-leaking equivalent) in each `Group` and one in `World`, populated as children are added (`Group::add_child`, `src/group.rs:30-36`, and `World::add_shape`, `src/world.rs:45-47`). Replace `find_by_id`'s linear scan with a single `HashMap::get` followed by recursion only if the id isn't local. (`HashMap<u64, &dyn Shape>` is fine if lifetimes are scoped to the containing `Group`/`World`.)
- **Estimated return:** **Medium-High.** `resolve_shape`/`find_by_id` are called at least once per primary ray hit and once per reflection/refraction sub-hit. Big-O goes from `O(objects x tree_size)` to `O(1)` average per lookup, per pixel. Significant for grouped/OBJ scenes.

---

## Medium Return

### 8. Cache a descendant-id `HashSet` in `CSG` to replace `contains`
- **Category:** Caching
- **Location:** `src/csg.rs:103-106` (`CSG::contains`), `src/csg.rs:108-130` (`filter_intersections`)
- **Current code:** `filter_intersections` iterates every intersection `i` in `xs` and calls `Self::contains(self.left.as_ref(), i.object.id())` (`src/csg.rs:113`). `contains` (`src/csg.rs:103-106`) does `root.id() == id || root.find_by_id(id).is_some()`, i.e. a full recursive subtree traversal per intersection. For a CSG-built scene this is `O(#intersections x #nodes)` per ray.
- **Proposed optimization:** Precompute a `HashSet<u64>` of all descendant shape ids inside `CSG::new` (`src/csg.rs:70-87`) at construction time (collect ids from `s1`/`s2` once, recursively). Then `contains(left, id)` becomes `self.left_ids.contains(&id)`, an `O(1)` set lookup. This also eliminates the `find_by_id` recursion entirely from the hot filter path.
- **Estimated return:** **Medium.** For CSG-heavy scenes (dice_csg, csg_tree_demo, football, refractive_sphere) this turns an `O(n^2)`-ish filter into `O(n)`. Per-ray, per-CSG-node cost is meaningful but CSG scenes are a minority of the demo binaries.

### 9. Store `Matrix` data as a flat `Vec<f64>` instead of `Vec<Vec<f64>>`
- **Category:** Caching (data structure)
- **Location:** `src/matrix.rs:7-11` (struct), `src/matrix.rs:15-39` (constructors), `src/matrix.rs:99-146` (`inverse_gauss_jordan`), `src/matrix.rs:165-204` (`Mul` impls), `src/matrix.rs:206-242` (matrix x tuple)
- **Current code:** `Matrix` holds `data: Vec<Vec<f64>>`. Every row is a separate heap allocation, every access is a double indirection (`data[i][j]` first loads `data[i]` (a pointer into a heap-allocated `Vec`'s buffer) then indexes), and `Mul for &Matrix` (`src/matrix.rs:168-204`) builds `Vec<Vec<f64>>` via `final_matrix.push(new_row)` only to immediately `flatten()` it (`src/matrix.rs:198-202`) — wasted intermediary allocations. `Mat x Tuple` (`src/matrix.rs:206-223`) and `Tuple x Mat` (`src/matrix.rs:225-242`) are the hottest in the renderer (called once per `ray.transform` in every `Shape::intersect`, `src/shape.rs:51`, and again in every `pattern_at_shape`, `src/pattern.rs:31-35`).
- **Proposed optimization:** Switch to `data: Vec<f64>` of length `rows * columns`, row-major. Implement `at(i, j) = data[i*columns + j]`. Rewrite `Mul<&Matrix> for &Matrix` to write flat output directly (no per-row `Vec`). Keep the public `data[i][j]` syntax-compatible by adding an accessor if needed, or migrate call-sites. This is a mechanical refactor because most code already uses `data[i][j]` indexing which becomes one helper call. Alternatively, keep the existing `Vec<Vec<f64>>` representation but at least rewrite `Mul<&Matrix> for &Matrix` to write directly into a pre-sized `Vec<Vec<f64>>` (no flatten).
- **Estimated return:** **Medium.** Per pixel this is called many times (ray transforms, normal_at, pattern eval). Flat storage removes the inner `Vec` allocation per row and improves cache locality — measured speedups in similar ray tracers are on the order of 1.5x-2x on matrix-heavy inner loops. Cross-cuts several hot paths at once.

---

## Low-Medium Return

### 10. Parallelize `Canvas::canvas_to_ppm` row-by-row
- **Category:** Multithreading
- **Location:** `src/canvas.rs:107-143`
- **Current code:** Building the PPM string is a single sequential `for y in 0..self.height { for x in 0..self.width { ... } }` that mutates one shared `String` and tracks a `line_len` cursor (the 70-char wrap state). For the teapot (800x600), that's 480k `scale_component().to_string()` calls plus repeated `out.push_str` with growing string buffer — one of the slower steps in the render-to-PPM pipeline, and it's strictly serial today.
- **Proposed optimization:** Render each row independently to its own `String` in parallel, since row-line-wrap state resets at every row boundary (`src/canvas.rs:137-139`). Use:
  ```rust
  let body: String = (0..self.height)
      .into_par_iter()
      .map(|y| self.format_row(y))
      .collect::<Vec<_>>()
      .join("");
  ```
  Prepend the header (`P3\n{w} {h}\n255\n`), with the newline-after-each-row invariant preserved. Wrap state stays per-row, so no cross-thread sharing.
- **Estimated return:** **Medium-Low.** Only invoked once per frame (not per pixel), but for large images it's a noticeable serial tail (often ~5-15% of frame time). Trivial to parallelize cleanly because rows are independent.

### 14. Skip sorting in `Intersections::new` when callers only need a boolean
- **Category:** Caching
- **Location:** `src/intersection.rs:135-138` (`Intersections::new` always sorts), called from every shape's `local_intersect` and from `intersect_world`/`is_shadowed_light`.
- **Current code:** Every `local_intersect` returns `Intersections::new(vec![...])` and `intersect_world` returns `Intersections::new(all)`, each invoking `items.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap())`. For shadow rays (most rays in a lit scene) the sort is thrown away.
- **Proposed optimization:** Add a `Intersections::new_unsorted` constructor and use it from the new early-exit shadow path (see #5). Color/reflection paths still need the sort for the containers algorithm in `prepare_computations` (`src/intersection.rs:92-116`). Also: each leaf shape already emits intersections in roughly ascending `t` order (e.g. sphere `t1 < t2`, `src/sphere.rs:42-47`), so a merge step at `intersect_world` could replace the full `sort_by`.
- **Estimated return:** **Medium-Low.** Removes a global sort per shadow ray; matters most for many-object scenes (#5 already addresses this more aggressively).

### 15. Reduce `prepare_computations` linear `containers` search to a `HashSet`
- **Category:** Caching
- **Location:** `src/intersection.rs:90-106`
- **Current code:** The `containers: Vec<&'a dyn Shape>` algorithm does `containers.iter().position(|o| o.id() == i.object.id())` for every intersection in `xs`, then `containers.remove(pos)` — both `O(n)` per step, making the whole n1/n2 computation `O(#intersections^2)` per ray. `prepare_computations` is called once per `color_at` and once per reflection/refraction sub-ray (`src/world.rs:126`, `148`, `184`).
- **Proposed optimization:** Use a `HashSet<u64>` for `containers` (keyed on shape `id()`). Membership and insert/remove become `O(1)`, dropping the loop to `O(#intersections)`. The "last entered refractive index" needs the last-pushed ordering, so additionally keep a `Vec<u64>` as the stack and use the `HashSet` only for membership probes.
- **Estimated return:** **Medium-Low.** `#intersections` is usually small per object, but for scenes where many objects stack along the ray (the glass/refraction demos in `src/bin/refractive_sphere.rs`, `src/bin/single_glass_sphere.rs`) the quadratic scan is noticeable.

---

## Low Return (quick wins)

### 11. Parallelize the per-light `fold` inside `shade_hit`
- **Category:** Multithreading
- **Location:** `src/world.rs:90-107`
- **Current code:** `shade_hit` builds `surface` by sequentially folding over `world.lights` (`src/world.rs:92-107`), computing one `lighting(...)` per light and summing up. Each per-light computation is fully independent (it only reads `light`, `comps`, and calls `is_shadowed_light`).
- **Proposed optimization:** Replace the `fold` with `world.lights.par_iter().map(|light| { ... }).reduce(Color::black, |a, b| &a + &b)`. `Color` is plain `f64`s, `Send`+`Sync` trivially. Combined with optimization #5 (early-exit shadows), each light becomes a clean parallel task.
- **Estimated return:** **Low.** Most demo scenes have 1-2 lights, so the speedup is small in practice. Grows with light count but is bounded — not a hot path.

### 12. Eliminate `comps.over_point.clone()` per light in `shade_hit`
- **Category:** Caching / clone elimination
- **Location:** `src/world.rs:96`
- **Current code:** Inside the per-light fold (`src/world.rs:92-107`), `world.is_shadowed_light(comps.over_point.clone(), light)` clones `over_point` (a `Tuple`, 32 bytes) once per light per shaded hit. With many lights and recursively reflective scenes, this is a small but repeated unit of allocation via `Tuple::clone` (`src/tuple.rs:5` derives `Clone`).
- **Proposed optimization:** Change `is_shadowed_light` (`src/world.rs:70`) to take `&Tuple` instead of `Tuple`. Propagate to `is_shadowed` (`src/world.rs:80-87`) iterating `&p` across lights. Removable cheap clone = no allocation, and matches the rest of the code which borrows `Tuple`s.
- **Estimated return:** **Low.** Removes the only per-light allocation in the hot shading path; small but free.

### 13. Avoid `i.clone()` in `CSG::filter_intersections`
- **Category:** Clone elimination
- **Location:** `src/csg.rs:120` (`kept.push(i.clone())`)
- **Current code:** `Intersection<'a>` is `#[derive(Clone)]` (`src/intersection.rs:8`); each kept intersection is cloned into the result vector. `Intersection` is small (an `f64`, a `&dyn Shape`, two `Option<f64>`), but cloning is still pointer copies that scale with `#intersections x #csg_nodes`. The original `xs.data` vec is dropped right after, so we could move rather than clone.
- **Proposed optimization:** Take `xs` by value (or `&mut Intersections`) and `drain`/`retain` in place, or build `kept` from `xs.data.into_iter().filter(...)` to move entries. Combined with #8 (set-based `contains`). Test code constructs `xs` and passes `&xs` (`src/csg.rs:240`) so signature change is contained.
- **Estimated return:** **Low.** Only CSG scenes, and `Intersection` clones are cheap, but eliminates memory churn in `O(n)` per ray per CSG node.

### 16. Pre-size intersection buffers and avoid repeated `Vec::new()` per ray
- **Category:** Caching (allocation)
- **Location:** `src/world.rs:50` (`Vec::new()` per `intersect_world`), `src/group.rs:54` (`Vec::new()` per `Group::local_intersect`), `src/cone.rs:72`, `src/cylinder.rs:68`, `src/intersection.rs:90` (`Vec::new()` for `containers`)
- **Current code:** Every ray allocates a fresh `Vec<Intersection>` for `all_intersections` and (after #4) the parallel `collect()`. With deep recursion (`MAX_RECURSION_DEPTH = 5`, `src/utils.rs:3`) each ray spawns multiple shadow + reflection + refraction sub-rays, each allocating. `Vec::new()` starts at capacity 0 and reallocates as `extend` pushes.
- **Proposed optimization:** Pre-size with `Vec::with_capacity(self.objects.len() * 2)` for world-level gathering (objects rarely yield more than 2 hits each) and similarly for groups. For the shadow path (#5) the early-exit `any` short-circuits anyway.
- **Estimated return:** **Low.** Each allocator call is cheap, but multiplied by millions of rays the reallocation savings add up. Free improvement.

---

## Already Done Well (no action needed)

These are existing caching/multithreading patterns the codebase already has in place, listed here for completeness so they aren't duplicated:

- `Camera::transform_inverse` cached alongside `transform`, recomputed only in `Camera::set_transform` (`src/camera.rs:48-51`).
- `ShapeData::transform_inverse` cached alongside `transform`, recomputed in `ShapeData::set_transform` (`src/shape.rs:33-36`).
- `PatternData::transform_inverse` cached similarly (`src/pattern.rs:9-25`).
- `Camera::render` already parallelizes pixels via `image.pixels_mut().par_iter_mut().zip(coordinates.par_iter()).for_each(...)` (`src/camera.rs:82-89`).
- `Triangle` precomputes `e1`, `e2`, and `normal` in its constructor (`src/triangle.rs:19-33`); `SmoothTriangle` precomputes `e1`/`e2` (`src/smooth_triangle.rs:25-39`).
- `Group` keeps an `ids: HashSet<u64>` (`src/group.rs:13`) for `includes` membership — though (per #7) it isn't yet used to speed up `find_by_id`.
- `ray.transform` and matrix x Tuple avoid heap allocations (uses stack arrays internally, `src/matrix.rs:206-223`).

---

## Recommended Implementation Order

For maximum reward/effort, land these incrementally:

1. **#2** cache inverse-transpose — small, mechanical, hot path.
2. **#4 + #3** parallelize `intersect_world` and `Group::local_intersect` — one-liners using existing rayon dep, called per ray.
3. **#5** short-circuit shadow rays — combines with #4 to remove wasted sort.
4. **#6** precompute camera origin — small, per-pixel win.
5. **#1** BVH — biggest isolated algorithmic win; substantial code addition but the most transformational for OBJ scenes.
6. **#7** HashMap lookups in `Group`/`World` — medium, also benefits scene setup.
7. **#8-#16** diminishing returns but each is local and low-risk.
