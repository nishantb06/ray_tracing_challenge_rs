
#### Baseline (Branch: master)(commit 08e917a29aec325ad77025e4b0478d555a222f03)
[benchmark] rendered 400x200 sphere scene to PPM in 321.789ms (248.6 pixels/ms)
No parallelisation optimisation yet

#### One thread per row (**3.6x speedup**) (commit 357125ed138c43c58a7fb701ac5bf253f0dbef67)
[benchmark] rendered 400x200 sphere scene to PPM in 89.138ms (897.5 pixels/ms)

#### With a fixed size worker pool based on core count and without allocating extra memory (3.5x speedup)
[benchmark] rendered 400x200 sphere scene to PPM in 95.803ms (835.0 pixels/ms)

#### With Inverse caching of camera transform (6.4x speedup)
[benchmark] rendered 400x200 sphere scene to PPM in 49.515ms (1615.7 pixels/ms)
