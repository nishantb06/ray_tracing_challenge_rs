#optimize

intersect_world(world, ray) function, which accepts a world and a ray, and returns the intersections

Arguments
1. a reference to a ray

It creates an empty vector of intersections. Then it loops over all the objects calling the [[intersect]] method on each of the object. Then it appends those intersections , if any with the intersection that the shapes intersect method returned. Finally it wraps the vector in an [[Intersections]] object and returns it. 

Note that it returns the Intersections in sorted order !

There is a scope of parallelisation here ! Since we iterate over all the objects one by one , multi threading can be done to compute all of these at once.