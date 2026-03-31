use std::collections::HashMap;
use crate::smooth_triangle::SmoothTriangle;
use crate::triangle::Triangle;
use crate::tuple::Tuple;
use crate::group::Group;

pub struct ObjParser {
    pub ignored_lines: usize,
    pub vertices: Vec<Tuple>,
    pub normals: Vec<Tuple>,
    pub default_group: Group,
    pub named_triangles: HashMap<String, Vec<Triangle>>,
    pub smooth_triangles: Vec<SmoothTriangle>,
    named_groups: HashMap<String, Group>
}

impl ObjParser {
    pub fn new() -> Self {
        ObjParser{
            ignored_lines: 0, // number of lines ignored, note that we are not storing what lines we are ignoring from the file, neither does it denote topk lines ignored
            vertices: vec![Tuple::point(0.0, 0.0, 0.0)], // to keep the vertices array 1 based
            normals: vec![Tuple::vector(0.0, 0.0, 0.0)], // index 0 dummy
            default_group: Group::new(),
            named_triangles: HashMap::new(),
            named_groups: HashMap::new(),
            smooth_triangles: Vec::new(),
        }
    }

    pub fn triangles_for_group(&self, name: &str) -> Option<&[Triangle]> {
        self.named_triangles.get(name).map(|v| v.as_slice())
    }

    pub fn parse(&mut self, src: &str) {
        let mut active_group: &str = "default";
        self.named_triangles.insert("default".to_string(), Vec::new());
        for raw_line in src.lines() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }
    
            if let Some(rest) = line.strip_prefix("v ") {
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if parts.len() != 3 {
                    // Option A (strict): count malformed vertex lines as ignored
                    self.ignored_lines += 1;
                    continue;
                }
    
                let x = parts[0].parse::<f64>();
                let y = parts[1].parse::<f64>();
                let z = parts[2].parse::<f64>();
    
                match (x, y, z) {
                    (Ok(x), Ok(y), Ok(z)) => self.vertices.push(Tuple::point(x, y, z)),
                    _ => {
                        // Option A (strict): malformed numbers count as ignored
                        self.ignored_lines += 1;
                    }
                }
    
                continue;
            }
            
            if let Some(rest) = line.strip_prefix("vn ") {
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if parts.len() != 3 {
                    self.ignored_lines += 1;
                    continue;
                }
            
                let x = parts[0].parse::<f64>();
                let y = parts[1].parse::<f64>();
                let z = parts[2].parse::<f64>();
            
                match (x, y, z) {
                    (Ok(x), Ok(y), Ok(z)) => self.normals.push(Tuple::vector(x, y, z)),
                    _ => self.ignored_lines += 1,
                }
            
                continue;
            }

            if let Some(rest) = line.strip_prefix("f ") {
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if parts.len() < 3 {
                    // Need at least 3 indices to make a triangle
                    self.ignored_lines += 1;
                    continue;
                }

                // Parse vertex and (optional) normal indices for this face.
                let mut vertex_indices: Vec<usize> = Vec::new();
                let mut normal_indices: Vec<Option<usize>> = Vec::new();
                let mut parse_failed = false;

                for part in parts.iter() {
                    // .obj faces may be "v", "v/t", "v//n" or "v/t/n"
                    let mut split = part.split('/');
                    let v_str = split.next().unwrap_or("");
                    let _t_str = split.next(); // texture index (ignored)
                    let n_str = split.next();  // optional normal index

                    // vertex index is mandatory
                    let v_idx = match v_str.parse::<usize>() {
                        Ok(idx) => idx,
                        Err(_) => {
                            parse_failed = true;
                            break;
                        }
                    };
                    vertex_indices.push(v_idx);

                    // normal index is optional
                    let n_idx = match n_str {
                        Some(s) if !s.is_empty() => match s.parse::<usize>() {
                            Ok(idx) => Some(idx),
                            Err(_) => {
                                parse_failed = true;
                                break;
                            }
                        },
                        _ => None,
                    };
                    normal_indices.push(n_idx);
                }

                if parse_failed {
                    self.ignored_lines += 1;
                    continue;
                }

                let has_normals = !normal_indices.is_empty() && normal_indices.iter().all(|opt| opt.is_some());

                // Fan triangulation: for n vertices, create triangles
                // (v1,v2,v3), (v1,v3,v4), ... (v1,v_{n-1},v_n)
                for i in 1..(vertex_indices.len() - 1) {
                    let idx1 = vertex_indices[0];
                    let idx2 = vertex_indices[i];
                    let idx3 = vertex_indices[i + 1];
                    
                    // Defensive: check validity of indices
                    if idx1 >= self.vertices.len() || idx2 >= self.vertices.len() || idx3 >= self.vertices.len() {
                        self.ignored_lines += 1;
                        continue;
                    }
                    
                    let p1 = self.vertices[idx1].clone();
                    let p2 = self.vertices[idx2].clone();
                    let p3 = self.vertices[idx3].clone();

                    if has_normals {
                        // Build a SmoothTriangle using corresponding normals.
                        let n_idx1 = normal_indices[0].unwrap();
                        let n_idx2 = normal_indices[i].unwrap();
                        let n_idx3 = normal_indices[i + 1].unwrap();

                        // Bounds check for normals
                        if n_idx1 >= self.normals.len()
                            || n_idx2 >= self.normals.len()
                            || n_idx3 >= self.normals.len()
                        {
                            self.ignored_lines += 1;
                            continue;
                        }

                        let n1 = self.normals[n_idx1].clone();
                        let n2 = self.normals[n_idx2].clone();
                        let n3 = self.normals[n_idx3].clone();

                        let s = SmoothTriangle::new(p1.clone(), p2.clone(), p3.clone(), n1.clone(), n2.clone(), n3.clone());
                        let s_copy = SmoothTriangle::new(p1.clone(), p2.clone(), p3.clone(), n1, n2, n3);

                        // keep typed copy for tests
                        self.smooth_triangles.push(s_copy);

                        // add to default or named group as `Box<dyn Shape>`
                        if active_group == "default" {
                            self.default_group.add_child(Box::new(s));
                        } else if let Some(group) = self.named_groups.get_mut(active_group) {
                            group.add_child(Box::new(s));
                        }
                    } else {
                        // Fallback: plain Triangle without normals.
                        let t = Triangle::new(p1.clone(), p2.clone(), p3.clone());
                        let t_copy = Triangle::new(p1.clone(), p2.clone(), p3.clone());
                        
                        if active_group == "default" {
                            self.default_group.add_child(Box::new(t));
                            if let Some(tris) = self.named_triangles.get_mut(active_group) {
                                tris.push(t_copy);
                            }
                        } else {
                            if let Some(group) = self.named_groups.get_mut(active_group) {
                                group.add_child(Box::new(t));
                            }
                            if let Some(tris) = self.named_triangles.get_mut(active_group) {
                                tris.push(t_copy);
                            }
                        }
                    }
                }
                continue;
            }

            if let Some(rest) = line.strip_prefix("g ") {
                let name  = rest.trim();
                self.named_groups.insert(name.to_string(), Group::new());
                self.named_triangles.insert(name.to_string(), Vec::new());
                active_group = name;

                continue;
            }
            self.ignored_lines += 1;
        }
    }


    pub fn obj_to_group(self) -> Group {
        let mut root = Group::new();

        // include default group (even if empty)
        root.add_child(Box::new(self.default_group));

        // include all named groups
        for (_name, group) in self.named_groups {
            root.add_child(Box::new(group));
        }

        root
    }
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

        let mut parser = ObjParser::new();
        parser.parse(gibberish);
        assert_eq!(parser.ignored_lines, 5);
    }


    #[test]
    fn vertex_records() {
        let file = r#"v -1 1 0
        v -1.0000 0.5000 0.0000
        v 1 0 0
        v 1 1 0
        "#;
    
        let mut parser = ObjParser::new();
        parser.parse(file);
    
        assert_eq!(parser.vertices[1], Tuple::point(-1.0, 1.0, 0.0));
        assert_eq!(parser.vertices[2], Tuple::point(-1.0, 0.5, 0.0));
        assert_eq!(parser.vertices[3], Tuple::point(1.0, 0.0, 0.0));
        assert_eq!(parser.vertices[4], Tuple::point(1.0, 1.0, 0.0));
    }

    #[test]
    fn parsing_triangle_faces() {
        let file = r#"v -1 1 0
        v -1 0 0
        v 1 0 0
        v 1 1 0
        f 1 2 3
        f 1 3 4
        "#;

        let mut parser = ObjParser::new();
        parser.parse(file);

        let g1 = parser.triangles_for_group("default").expect("missing defualt");
        let t1 = &g1[0];
        let t2 = &g1[1];
        assert_eq!(t1.p1, parser.vertices[1]);
        assert_eq!(t1.p2, parser.vertices[2]);
        assert_eq!(t1.p3, parser.vertices[3]);
        assert_eq!(t2.p1, parser.vertices[1]);
        assert_eq!(t2.p2, parser.vertices[3]);
        assert_eq!(t2.p3, parser.vertices[4]);

        assert_eq!(parser.default_group.shapes.len(),2);
    }

    #[test]
    fn parsing_polygon_faces_gets_triangulated() {
        let file = r#"
        v -1 1 0
        v -1 0 0
        v 1 0 0
        v 1 1 0
        v 0 2 0
        f 1 2 3 4 5
        "#;

        let mut parser = ObjParser::new();
        parser.parse(file);

        // The polygon should be triangulated into three triangles:
        //  f 1 2 3 4 5
        //    --> 1 2 3, 1 3 4, 1 4 5

        assert_eq!(parser.default_group.shapes.len(), 3);
        let g1 = parser.triangles_for_group("default").expect("missing defualt");
        assert_eq!(g1.len(), 3);

        let t1 = &g1[0];
        let t2 = &g1[1];
        let t3 = &g1[2];

        // Each triangle's points should be the correct vertices
        assert_eq!(t1.p1, parser.vertices[1]);
        assert_eq!(t1.p2, parser.vertices[2]);
        assert_eq!(t1.p3, parser.vertices[3]);
        assert_eq!(t2.p1, parser.vertices[1]);
        assert_eq!(t2.p2, parser.vertices[3]);
        assert_eq!(t2.p3, parser.vertices[4]);

        assert_eq!(t3.p1, parser.vertices[1]);
        assert_eq!(t3.p2, parser.vertices[4]);
        assert_eq!(t3.p3, parser.vertices[5]);
    }

    #[test]
    fn triangles_in_groups() {
        let file = r#"v -1 1 0
        v -1 0 0
        v 1 0 0
        v 1 1 0
        g FirstGroup
        f 1 2 3
        g SecondGroup
        f 1 3 4
        "#;
    
        let mut parser = ObjParser::new();
        parser.parse(file);
    
        let g1 = parser.triangles_for_group("FirstGroup").expect("missing FirstGroup");
        let g2 = parser.triangles_for_group("SecondGroup").expect("missing SecondGroup");
    
        let t1 = &g1[0];
        let t2 = &g2[0];
    
        assert_eq!(t1.p1, parser.vertices[1]);
        assert_eq!(t1.p2, parser.vertices[2]);
        assert_eq!(t1.p3, parser.vertices[3]);
    
        assert_eq!(t2.p1, parser.vertices[1]);
        assert_eq!(t2.p2, parser.vertices[3]);
        assert_eq!(t2.p3, parser.vertices[4]);
    }

    #[test]
    fn converting_an_obj_file_to_a_group() {
        let file = r#"v -1 1 0
        v -1 0 0
        v 1 0 0
        v 1 1 0
        g FirstGroup
        f 1 2 3
        g SecondGroup
        f 1 3 4
        "#;
    
        let mut parser = ObjParser::new();
        parser.parse(file);
    
        let g1_id = parser
            .named_groups
            .get("FirstGroup")
            .expect("missing FirstGroup")
            .id();
        let g2_id = parser
            .named_groups
            .get("SecondGroup")
            .expect("missing SecondGroup")
            .id();
    
        let g = parser.obj_to_group();
    
        assert!(g.includes(g1_id));
        assert!(g.includes(g2_id));
    }

    #[test]
    fn vertex_normal_records() {
        let file = r#"vn 0 0 1
vn 0.707 0 -0.707
vn 1 2 3
"#;
        
        let mut parser = ObjParser::new();
        parser.parse(file);
        
        assert_eq!(parser.normals[1], Tuple::vector(0.0, 0.0, 1.0));
        assert_eq!(parser.normals[2], Tuple::vector(0.707, 0.0, -0.707));
        assert_eq!(parser.normals[3], Tuple::vector(1.0, 2.0, 3.0));
    }

    #[test]
    fn faces_with_normals() {
        let file = r#"v 0 1 0
v -1 0 0
v 1 0 0
vn -1 0 0
vn 1 0 0
vn 0 1 0
f 1//3 2//1 3//2
f 1/0/3 2/102/1 3/14/2
"#;

        let mut parser = ObjParser::new();
        parser.parse(file);

        assert_eq!(parser.default_group.shapes.len(), 2);
        assert_eq!(parser.smooth_triangles.len(), 2);

        let t1 = &parser.smooth_triangles[0];
        let t2 = &parser.smooth_triangles[1];

        assert_eq!(t1.p1, parser.vertices[1]);
        assert_eq!(t1.p2, parser.vertices[2]);
        assert_eq!(t1.p3, parser.vertices[3]);

        assert_eq!(t1.n1, parser.normals[3]);
        assert_eq!(t1.n2, parser.normals[1]);
        assert_eq!(t1.n3, parser.normals[2]);

        assert_eq!(t2.p1, t1.p1);
        assert_eq!(t2.p2, t1.p2);
        assert_eq!(t2.p3, t1.p3);
        assert_eq!(t2.n1, t1.n1);
        assert_eq!(t2.n2, t1.n2);
        assert_eq!(t2.n3, t1.n3);
    }
}