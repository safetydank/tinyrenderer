use std::fs;
use regex::Regex;

use crate::geometry::{Vector2, Vector3};

pub struct Mesh {
    //  vertices
    pub vs: Vec<Vector3>,
    //  vertex indices (triangles)
    pub vis: Vec<i32>,
    //  texture coords
    pub tex: Vec<Vector2>,
    //  texture indices
    pub tis: Vec<i32>
}

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
            Some(t @ ("v" | "vt")) => {
                let x = tokens.next().unwrap().parse::<f32>().unwrap();
                let y = tokens.next().unwrap().parse::<f32>().unwrap();
                let z = tokens.next().unwrap().parse::<f32>().unwrap();
                
                match t {
                    "v" => mesh.vs.push(Vector3::new(x, y, z)),
                    "vt" => {
                        mesh.tex.push(Vector2::new(x, y));
                    },
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
                }
            },
            Some(_) | None => { },
        }
    }
    
    mesh
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            vs: vec![Vector3::new(0.0, 0.0, 0.0)],
            vis: vec![],
            tex: vec![Vector2::new(0.0, 0.0)],
            tis: vec![],
        }
    }
}