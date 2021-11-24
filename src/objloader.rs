use std::fs;
use glam::Vec3A;

pub struct Mesh<T> {
    //  vertices
    pub vs: Vec<T>,
    //  vertex indices (triangles)
    pub vis: Vec<i32>,
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

    for line in lines {
        if line.starts_with("v ") {
            let mut values = line.split(" ").skip(1);
            let x = values.next().unwrap().parse::<f32>().unwrap();
            let y = values.next().unwrap().parse::<f32>().unwrap();
            let z = values.next().unwrap().parse::<f32>().unwrap();
            mesh.vs.push(T::create(x, y, z));
            // println!("Pushed x {} y {} z {}", x, y, z);
        } else if line.starts_with("f ") {
            let mut values = line.split(" ").skip(1);
            for _ in 0..3 {
                let index = values.next().unwrap().split("/").next().unwrap().parse::<i32>().unwrap();
                mesh.vis.push(index);
            }
        }
    }
    
    mesh
}

impl<T: Vector3> Mesh<T> {
    pub fn new() -> Self {
        Self {
            vs: vec![T::create(0.0, 0.0, 0.0)],
            vis: vec![],
        }
    }
}