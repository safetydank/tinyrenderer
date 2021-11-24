use std::path::Path;
use std::fs::File;
use std::io::BufWriter;

use glam::{IVec2, Vec3A};

use crate::objloader::Mesh;
use crate::renderer::Renderer;

pub fn save_png(path_str: &str, width: u32, height: u32, buf: &[u32]) {
    let path = Path::new(&path_str);
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();
    // convert u32 buffer to u8
    let bbuf:Vec<u8> = buf.iter().flat_map(|v| v.to_be_bytes()).collect();
    writer.write_image_data(&bbuf).unwrap(); // Save
}

pub fn draw_mesh(r: &mut Renderer, mesh: &Mesh<Vec3A>) {
    // let mut rng = rand::thread_rng();

    let light_dir = Vec3A::new(0.0, 0.0, -1.0);

    for tri in mesh.vis.chunks_exact(3) {
        let w = (r.width - 1) as f32;
        let h = (r.height - 1) as f32;
        // println!("Triangle {} {} {}", tri[0], tri[1], tri[2]);
        
        // world space vertices
        let vs: Vec<Vec3A> = tri.iter().map(|i| {
            mesh.vs[*i as usize]
        }).collect();

        // project vertices into screen space points
        let pts: Vec<Vec3A> = vs.iter().map(|v| {
            Vec3A::new(
                (v.x + 1.0) * w / 2.0,
                (v.y + 1.0) * h / 2.0,
                v.z
            )
        }).collect();
        
        // normal
        let n = Vec3A::cross(vs[2] - vs[0], vs[1] - vs[0]).normalize();
        let intensity = (Vec3A::dot(n, light_dir) * 255.0) as u32;

        if intensity > 0 {
            let color = (intensity<<24) | (intensity<<16) | (intensity<<8) | 0xff;
            r.triangle_fill(pts, color);
        }
    }

}