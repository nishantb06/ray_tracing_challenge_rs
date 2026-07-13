#### Baseline (Loop over every pixel)
[benchmark] rendered 400x200 sphere scene to PPM in 321.789ms (248.6 pixels/ms)
No parallelisation optimisation yet

#### Each thread per pixel (commit : efec34bd067fbc415f4bed3b39059bfa8012207a)
##### So much worse than baseline
[benchmark] rendered 400x200 sphere scene to PPM in 47.108s (1.7 pixels/ms)
remarks: spawning so many threads has a huge overhead cost!

#### Letting a thread pool the size of the number of cores handle the computation by chunking the pixels into segments.
##### Speed up 3.2x
[benchmark] rendered 400x200 sphere scene to PPM in 97.434ms (821.1 pixels/ms)
remarks: makes as many threads as the number of chunks = number of cores then work inside each thread is sequential. This has a similar baseline as the AI provided optimisation

#### Caching the inverse matrix of the camera transform 
##### Speed up 5.2x
commit :273f6e41d6ee0a3cc0d9b19de3e80718038adb5a
[benchmark] rendered 400x200 sphere scene to PPM in 61.053ms (1310.3 pixels/ms)

#### Letting Rayon handle the multi threading since its dynamic work stealing algo will be better for uneven work while ray tracing. Each pixel will not have the same work
##### Speedup (6x)
commit : 5d3a6999c7d09ba072affec119ee3f301f368eaf
[benchmark] rendered 400x200 sphere scene to PPM in 52.241ms (1531.4 pixels/ms)

#### Canvas exposes a mutable pixel slice and the result of the computations is written there directly avoiding double allocations of memory
##### Speedup (6.25x)
commit: 92c72cabb5b194ee2586ee3a47f37a5f80b167e4
[benchmark] rendered 400x200 sphere scene to PPM in 51.271ms (1560.3 pixels/ms)

#### ShapeData now caches transform_inverse_transpose in set_transform, and normal_at / normal_to_world use that instead of calling .transpose() each time.
##### Speedup (7x speedup)
[benchmark] rendered 400x200 sphere scene to PPM in 45.732ms (1749.3 pixels/ms)

