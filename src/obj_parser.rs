use crate::triangle::Triangle;
use crate::tuple::Tuple;
use crate::group::Group;

pub struct ObjParser {
    pub ignored_lines: usize,
    pub vertices: Vec<Tuple>,
    pub default_group: Group,
    pub triangles: Vec<Triangle>,
}

impl ObjParser {
    pub fn new() -> Self {
        ObjParser{
            ignored_lines: 0, // number of lines ignored, note that we are not storing what lines we are ignoring from the file, neither does it denote topk lines ignored
            vertices: vec![Tuple::point(0.0, 0.0, 0.0)], // to keep the vertices array 1 based
            default_group: Group::new(),
            triangles: vec![],
        }
    }

    pub fn parse(&mut self, src: &str) {
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
                    // Option A (strict): count malformed vertex lines as ignored
                    self.ignored_lines += 1;
                    continue;
                }
                let i = parts[0].parse::<usize>();
                let j = parts[1].parse::<usize>();
                let k = parts[2].parse::<usize>();

                match (i,j,k) {
                    (Ok(i),Ok(j),Ok(k)) => {
                        // create a triangle with vertices i,j,k and add it to group
                        let p1 = self.vertices[i].clone();
                        let p2 = self.vertices[j].clone();
                        let p3 = self.vertices[k].clone();
                        let t = Triangle::new(p1.clone(), p2.clone(), p3.clone());
                        let t_copy = Triangle::new(p1.clone(), p2.clone(), p3.clone());

                        self.triangles.push(t_copy);
                        // add to the default group
                        self.default_group.add_child(Box::new(t));
                    },
                    _ => {self.ignored_lines += 1;}
                }

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

        let t1 = &parser.triangles[0];
        let t2 = &parser.triangles[1];
        assert_eq!(t1.p1, parser.vertices[1]);
        assert_eq!(t1.p2, parser.vertices[2]);
        assert_eq!(t1.p3, parser.vertices[3]);
        assert_eq!(t2.p1, parser.vertices[1]);
        assert_eq!(t2.p2, parser.vertices[3]);
        assert_eq!(t2.p3, parser.vertices[4]);

        assert_eq!(parser.default_group.shapes.len(),2);
    }
}