use std::fs;
use crate::geometry::Vec3f;

pub struct Mesh {
    //  vertices
    pub vs: Vec<Vec3f>,
    //  vertex indices (triangles)
    pub vis: Vec<i32>,
}

pub fn load_obj(path: &str) -> Mesh {
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
            mesh.vs.push(Vec3f::new(x, y, z));
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

impl Mesh {
    pub fn new() -> Self {
        Self {
            vs: vec![Vec3f::new(0.0, 0.0, 0.0)],
            vis: vec![],
        }
    }
}