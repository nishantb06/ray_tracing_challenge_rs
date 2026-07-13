#independent-function #optimize 

Returns the color at the intersection encapsulated by comps, in the given world.

[[prepare_computations]] returned a single Computations object, this function takes this and return the color at this intersection as per the [[Phong Reflection Model]]


One of the [[Independent functions]] which takes in [[World]] and [[Computations]] and turns a hit’s precomputed data into a surface color by combining lighting, reflection, and refraction (with Schlick blending when both reflective and transparent)

Arguments
world: &World
comps: &Computations
remaining: i32
Utilises the [[lighting]] function to get the color of the point through each of the light sources
#### Return 
A [[Color]] object, not even a reference to it!

Can be optimized because it iterates over all the light sources which can be done via parallelisation.  