use std::collections::HashMap;

use crate::triangle::Triangle;
use crate::tuple::Tuple;
use crate::group::Group;

pub struct ObjParser {
    pub ignored_lines: usize,
    pub vertices: Vec<Tuple>,
    pub default_group: Group,
    pub named_triangles: HashMap<String, Vec<Triangle>>,
    named_groups: HashMap<String, Group>
}

impl ObjParser {
    pub fn new() -> Self {
        ObjParser{
            ignored_lines: 0, // number of lines ignored, note that we are not storing what lines we are ignoring from the file, neither does it denote topk lines ignored
            vertices: vec![Tuple::point(0.0, 0.0, 0.0)], // to keep the vertices array 1 based
            default_group: Group::new(),
            named_triangles: HashMap::new(),
            named_groups: HashMap::new(),
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
    
            // Later: handle vn/vt/f/g/usemtl/mtllib here.
            if let Some(rest) = line.strip_prefix("f ") {
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if parts.len() < 3 {
                    // Need at least 3 indices to make a triangle
                    self.ignored_lines += 1;
                    continue;
                }
                // Try to parse all indices for this face.
                let mut indices = Vec::new();
                let mut parse_failed = false;
                for part in parts.iter() {
                    // .obj faces may have slashes: "1/2/3", "1//3" etc, so take only the first part before '/'
                    let vertex_idx_str = part.split('/').next().unwrap();
                    match vertex_idx_str.parse::<usize>() {
                        Ok(idx) => indices.push(idx),
                        Err(_) => {
                            parse_failed = true;
                            break;
                        }
                    }
                }
                if parse_failed {
                    self.ignored_lines += 1;
                    continue;
                }
                // Fan triangulation: for n vertices, create triangles (v1,v2,v3), (v1,v3,v4), ... (v1,v_{n-1},v_n)
                for i in 1..(indices.len() - 1) {
                    let idx1 = indices[0];
                    let idx2 = indices[i];
                    let idx3 = indices[i + 1];
                    
                    // Defensive: check validity of indices
                    if idx1 >= self.vertices.len() || idx2 >= self.vertices.len() || idx3 >= self.vertices.len() {
                        self.ignored_lines += 1;
                        continue;
                    }
                    
                    let p1 = self.vertices[idx1].clone();
                    let p2 = self.vertices[idx2].clone();
                    let p3 = self.vertices[idx3].clone();
                    
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}