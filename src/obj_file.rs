use std::collections::HashMap;

use crate::group::Group;
use crate::material::Material;
use crate::shape::Shape;
use crate::triangle::Triangle;
use crate::tuple::Tuple;

#[derive(Debug)]
enum ActiveGroup {
    Default,
    Named(String),
}

/// OBJ vertex list uses 1-based indices in the file format; `vertices[0]` is an unused slot.
#[derive(Debug)]
pub struct ObjParser {
    pub ignored_lines: usize,
    pub vertices: Vec<Tuple>,
    /// All triangles from `f` lines, in parse order (across default and named groups).
    triangle_refs: Vec<&'static Triangle>,
    pub default_group: Group<'static>,
    /// Groups introduced by `g name` lines; faces after a `g` go here until another `g`.
    named_groups: HashMap<String, Group<'static>>,
}

impl ObjParser {
    /// Triangles from `f` lines, in parse order (same order as encountered in the file).
    pub fn triangle_children(&self) -> &[&'static Triangle] {
        &self.triangle_refs
    }

    /// Named group from a `g name` statement, if it was present in the file.
    pub fn group(&self, name: &str) -> Option<&Group<'static>> {
        self.named_groups.get(name)
    }
}

impl Default for ObjParser {
    fn default() -> Self {
        Self {
            ignored_lines: 0,
            vertices: vec![Tuple::point(0.0, 0.0, 0.0)],
            triangle_refs: vec![],
            default_group: Group::new(),
            named_groups: HashMap::new(),
        }
    }
}

pub fn parse_obj_file(src: &str) -> ObjParser {
    parse_obj_file_inner(src, None)
}

/// Like [`parse_obj_file`], but assigns `material` to every triangle (patterns cleared).
pub fn parse_obj_file_with_material(src: &str, material: &Material) -> ObjParser {
    parse_obj_file_inner(src, Some(material))
}

fn parse_obj_file_inner(src: &str, material: Option<&Material>) -> ObjParser {
    let mut ignored_lines = 0;
    let mut vertices = vec![Tuple::point(0.0, 0.0, 0.0)];
    let mut default_group = Group::new();
    let mut named_groups: HashMap<String, Group<'static>> = HashMap::new();
    let mut active = ActiveGroup::Default;
    let mut triangle_refs: Vec<&'static Triangle> = vec![];

    for line in src.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let first = parts.next().unwrap();
        match first {
            "v" => {
                let x: f64 = parts.next().unwrap().parse().unwrap();
                let y: f64 = parts.next().unwrap().parse().unwrap();
                let z: f64 = parts.next().unwrap().parse().unwrap();
                vertices.push(Tuple::point(x, y, z));
            }
            "g" => {
                let name = parts.collect::<Vec<_>>().join(" ");
                let name = name.trim();
                if name.is_empty() {
                    continue;
                }
                named_groups
                    .entry(name.to_string())
                    .or_insert_with(Group::new);
                active = ActiveGroup::Named(name.to_string());
            }
            "f" => {
                let face_indices: Vec<usize> = parts.map(parse_face_vertex_index).collect();
                assert!(
                    face_indices.len() >= 3,
                    "face must list at least three vertices"
                );
                // Fan triangulation (convex polygons): anchor at face_indices[0].
                for j in 1..face_indices.len() - 1 {
                    let i0 = face_indices[0];
                    let i1 = face_indices[j];
                    let i2 = face_indices[j + 1];
                    let p1 = vertices[i0].clone();
                    let p2 = vertices[i1].clone();
                    let p3 = vertices[i2].clone();
                    let tri = Box::leak(Box::new(Triangle::new(p1, p2, p3)));
                    if let Some(m) = material {
                        assign_material(tri.material_mut(), m);
                    }
                    match &active {
                        ActiveGroup::Default => {
                            tri.shape_data_mut().parent = Some(default_group.id());
                            default_group.add_child(&*tri);
                        }
                        ActiveGroup::Named(name) => {
                            let g = named_groups
                                .get_mut(name)
                                .expect("named group must exist after g line");
                            tri.shape_data_mut().parent = Some(g.id());
                            g.add_child(&*tri);
                        }
                    }
                    triangle_refs.push(&*tri);
                }
            }
            "vt" | "vn" => {}
            _ => ignored_lines += 1,
        }
    }
    ObjParser {
        ignored_lines,
        vertices,
        triangle_refs,
        default_group,
        named_groups,
    }
}

fn assign_material(dst: &mut Material, src: &Material) {
    dst.color = src.color.clone();
    dst.ambient = src.ambient;
    dst.diffuse = src.diffuse;
    dst.specular = src.specular;
    dst.shininess = src.shininess;
    dst.pattern = None;
    dst.reflective = src.reflective;
    dst.transparency = src.transparency;
    dst.refractive_index = src.refractive_index;
}

/// Wraps the parsed model in a single root [`Group`] for attaching to a scene.
/// Non-empty [`ObjParser::default_group`] and every [`ObjParser::named_groups`] entry become children.
pub fn obj_to_group<'a>(parser: &'a ObjParser) -> Group<'a> {
    let mut g = Group::new();
    if parser.default_group.len() > 0 {
        g.add_child(&parser.default_group);
    }
    for sub in parser.named_groups.values() {
        g.add_child(sub);
    }
    g
}

/// First number in an OBJ face corner token, e.g. `1` from `1` or `1/2/3`.
fn parse_face_vertex_index(token: &str) -> usize {
    token
        .split('/')
        .next()
        .expect("empty face token")
        .parse()
        .expect("face vertex index must be an integer")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::Shape;

    #[test]
    fn ignoring_unrecognized_lines() {
        let gibberish = r"There was a young lady named Bright
who traveled much faster than light.
She set out one day
in a relative way,
and came back the previous night.";
        let parser = parse_obj_file(gibberish);
        assert_eq!(parser.ignored_lines, 5);
    }

    #[test]
    fn vertex_records() {
        let file = r"v -1 1 0
v -1.0000 0.5000 0.0000
v 1 0 0
v 1 1 0";
        let parser = parse_obj_file(file);
        assert!(parser.vertices[1].is_equal(&Tuple::point(-1.0, 1.0, 0.0)));
        assert!(parser.vertices[2].is_equal(&Tuple::point(-1.0, 0.5, 0.0)));
        assert!(parser.vertices[3].is_equal(&Tuple::point(1.0, 0.0, 0.0)));
        assert!(parser.vertices[4].is_equal(&Tuple::point(1.0, 1.0, 0.0)));
    }

    #[test]
    fn parsing_triangle_faces() {
        let file = r"v -1 1 0
v -1 0 0
v 1 0 0
v 1 1 0
f 1 2 3
f 1 3 4";
        let parser = parse_obj_file(file);
        let g = &parser.default_group;
        assert_eq!(g.len(), 2);
        let tris = parser.triangle_children();
        assert_eq!(tris.len(), 2);
        assert_eq!(g.shapes[0].id(), tris[0].id());
        assert_eq!(g.shapes[1].id(), tris[1].id());

        let t1 = tris[0];
        let t2 = tris[1];
        assert!(t1.p1.is_equal(&parser.vertices[1]));
        assert!(t1.p2.is_equal(&parser.vertices[2]));
        assert!(t1.p3.is_equal(&parser.vertices[3]));
        assert!(t2.p1.is_equal(&parser.vertices[1]));
        assert!(t2.p2.is_equal(&parser.vertices[3]));
        assert!(t2.p3.is_equal(&parser.vertices[4]));
    }

    #[test]
    fn triangulating_polygons() {
        let file = r"v -1 1 0
v -1 0 0
v 1 0 0
v 1 1 0
v 0 2 0
f 1 2 3 4 5";
        let parser = parse_obj_file(file);
        let g = &parser.default_group;
        assert_eq!(g.len(), 3);
        let tris = parser.triangle_children();
        assert_eq!(tris.len(), 3);

        let t1 = tris[0];
        let t2 = tris[1];
        let t3 = tris[2];
        assert!(t1.p1.is_equal(&parser.vertices[1]));
        assert!(t1.p2.is_equal(&parser.vertices[2]));
        assert!(t1.p3.is_equal(&parser.vertices[3]));
        assert!(t2.p1.is_equal(&parser.vertices[1]));
        assert!(t2.p2.is_equal(&parser.vertices[3]));
        assert!(t2.p3.is_equal(&parser.vertices[4]));
        assert!(t3.p1.is_equal(&parser.vertices[1]));
        assert!(t3.p2.is_equal(&parser.vertices[4]));
        assert!(t3.p3.is_equal(&parser.vertices[5]));
    }

    #[test]
    fn triangles_in_groups() {
        let src = include_str!("../files/triangles.obj");
        let parser = parse_obj_file(src);
        let g1 = parser.group("FirstGroup").expect("FirstGroup");
        let g2 = parser.group("SecondGroup").expect("SecondGroup");
        assert_eq!(g1.len(), 1);
        assert_eq!(g2.len(), 1);
        assert_eq!(g1.shapes[0].id(), parser.triangle_children()[0].id());
        assert_eq!(g2.shapes[0].id(), parser.triangle_children()[1].id());

        let t1 = parser.triangle_children()[0];
        let t2 = parser.triangle_children()[1];
        assert!(t1.p1.is_equal(&parser.vertices[1]));
        assert!(t1.p2.is_equal(&parser.vertices[2]));
        assert!(t1.p3.is_equal(&parser.vertices[3]));
        assert!(t2.p1.is_equal(&parser.vertices[1]));
        assert!(t2.p2.is_equal(&parser.vertices[3]));
        assert!(t2.p3.is_equal(&parser.vertices[4]));
    }

    #[test]
    fn converting_obj_file_to_group() {
        let src = include_str!("../files/triangles.obj");
        let parser = parse_obj_file(src);
        let g = obj_to_group(&parser);
        let g1 = parser.group("FirstGroup").expect("FirstGroup");
        let g2 = parser.group("SecondGroup").expect("SecondGroup");
        assert!(g.includes(g1));
        assert!(g.includes(g2));
    }
}
