#independent-function 

Related to [[Intersections]] , [[Computations]] and 

From a hit, ray, and full intersection list, builds the world-space point, eye/normal/reflect vectors, inside flag, over/under points, and refractive indices `n1`/`n2` needed for shading.


This function precomputes the point (in world space) where the intersection occurred, the eye vector (pointing back toward the eye, or camera), and the normal vector.

#### Arguments
1. Reference to an intersection. this intersection is the one with the lowest t which was returned from the [[intersect_world]] method
2. The ray in question.
3. xs : The list of all the intersections:
4. resolve_parent: 

##### Workings
Computes the normal of the shape at the point of intersection, eye vector, and a bunch of other stuff

In this case, the surface normal (as currently computed) points away from the eye. But if the normal is pointing away from the eye, the shading algorithm from the previous chapter will color the surface far darker than it ought to be. So, how can you know—mathematically—if the normal points away from the eye vector? Take the dot product of the two vectors, and if the result is negative, they’re pointing in (roughly) opposite directions.
#### Returns
returns a new data structure [[Computations]] encapsulating some precomputed information relating to the intersection