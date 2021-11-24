use std::fs;
use glam::{Vec2, Vec3A};
use regex::Regex;

pub struct Mesh<T> {
    //  vertices
    pub vs: Vec<T>,
    //  vertex indices (triangles)
    pub vis: Vec<i32>,
    //  texture indices
    pub tex: Vec<Vec2>,
}

pub trait Vector3 {
    fn create(x: f32, y: f32, z: f32) -> Self;
}

impl Vector3 for Vec3A {
    fn create(x: f32, y: f32, z: f32) -> Self {
        Vec3A::new(x, y, z)
    }
}

pub fn load_obj<T: Vector3>(path: &str) -> Mesh<T> {
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
                    "v" => mesh.vs.push(T::create(x, y, z)),
                    "vt" => mesh.tex.push(Vec2::new(x, y)),
                    _ => { },
                }
            },
            Some("f") => {
                for _ in 0..3 {
                    let index = tokens.next().unwrap().split("/").next().unwrap().parse::<i32>().unwrap();
                    mesh.vis.push(index);
                }
            },
            Some(_) | None => { },
        }
    }
    
    mesh
}

impl<T: Vector3> Mesh<T> {
    pub fn new() -> Self {
        Self {
            vs: vec![T::create(0.0, 0.0, 0.0)],
            vis: vec![],
            tex: vec![Vec2::new(0.0, 0.0)],
        }
    }
}