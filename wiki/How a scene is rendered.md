[[Camera]].[[render]]
  → [[ray_for_pixel]] (per pixel)
    → [[Ray]]
      → [[World]].[[color_at]]
        → [[intersect_world]]
          → [[Shape]].[[intersect]] (each object)
            → [[Intersections.hit]]
              → [[prepare_computations]] → [[Computations]]
                → [[shade_hit]]
                  → [[lighting]] (per [[Light]])
                  → [[reflected_color]]
                  → [[refracted_color]]
                  → [[schlick]] (if reflective + transparent)
  → [[Canvas]].[[write_pixel]]
  → [[Canvas]].[[canvas_to_ppm]] q