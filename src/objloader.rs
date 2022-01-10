use std::fs;
use regex::Regex;

use crate::{geometry::{Vector2, Vector3}, renderer::{Mesh, Index}};

pub fn load_obj(path: &str) -> Mesh {
    let mut mesh = Mesh::new();

    // hacking an obj reader with no error handling
    let content = fs::read_to_string(path).expect("Error reading file");
    let lines = content.split("\n");

    let re = Regex::new(r" +").expect("Invalid regex");

    for line in lines {
        let mut tokens = re.split(line);
        let field = tokens.next();
        match field {
            Some(t @ ("v" | "vt" | "vn")) => {
                let x = tokens.next().unwrap().parse::<f32>().unwrap();
                let y = tokens.next().unwrap().parse::<f32>().unwrap();
                let z = tokens.next().unwrap().parse::<f32>().unwrap();
                
                match t {
                    "v" => mesh.vs.push(Vector3::new(x, y, z)),
                    "vt" => mesh.tex.push(Vector2::new(x, y)),
                    "vn" => mesh.ns.push(Vector3::new(x, y, z)),
                    _ => { },
                }
            },
            Some("f") => {
                for _ in 0..3 {
                    let mut triple_iter = tokens.next().unwrap().split("/");
                    let vi = triple_iter.next().unwrap().parse::<i32>().unwrap();
                    mesh.vis.push(vi);
                    let ti = triple_iter.next().unwrap().parse::<i32>().unwrap();
                    mesh.tis.push(ti);
                    let ni = triple_iter.next().unwrap().parse::<i32>().unwrap();
                    mesh.nis.push(ni);
                    // XXX convert negative indices to positive
                    mesh.indexes.push(Index::new(vi as usize, ti as usize, ni as usize));
                }
                // mesh.faces.push(Face::new(points[0], points[1], points[2]));
            },
            Some(_) | None => { },
        }
    }
    
    mesh
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            vs: vec![Vector3::ZERO],
            vis: vec![],
            tex: vec![Vector2::ZERO],
            tis: vec![],
            ns: vec![Vector3::ZERO],
            nis: vec![],
            indexes: vec![],
        }
    }
}