# Composition recipes

Make a readable humanoid with an elongated cube torso, a smaller cube or sphere
head, and closed truncated cylinders for limbs. A standing figure reads best
when the head is one torso-width above its center, shoulders are near the torso
top, and feet sit on a plane. Rotate arms and legs slightly away from vertical
for a relaxed pose; stagger them and tilt the torso for motion.

For legibility, use a light from above-left, a matte floor (`specular = 0.0`),
and a camera at about `(0, 2, -8)` looking toward `(0, 1.5, 0)`. Frame the full
silhouette with a little empty border. When feedback says a piece is 25 percent
smaller, multiply its existing scale by 0.75 rather than changing unrelated
translations.
