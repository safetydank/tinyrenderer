use std::path::Path;
use std::fs::File;
use std::io::BufWriter;

use crate::geometry::{Vector2, Vector3, Vector4};

use crate::objloader::Mesh;
use crate::renderer::{Renderer, Texture};

pub fn load_png_texture(path_str: &str) -> Texture {
    let decoder = png::Decoder::new(File::open(path_str).unwrap());
    let mut reader = decoder.read_info().unwrap();
    let mut bytebuf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut bytebuf).unwrap();
    
    Texture {
        width: info.width as f32,
        height: info.height as f32,
        buf: bytebuf.chunks_exact(3).map(|b| {
            // let bytes = [b[2], b[1], b[0], 0xff];
            let bytes = [b[0], b[1], b[2], 0xff];
            u32::from_be_bytes(bytes)
        }).collect()
    }
}

pub fn vec4_from_color(c: u32) -> Vector4 {
    Vector4::new(((c & 0xff000000) >> 24) as f32, ((c & 0xff0000) >> 16) as f32, ((c & 0xff00) >> 8) as f32, (c & 0xff) as f32)
}

pub fn color_from_vec4(v: Vector4) -> u32 {
    let r = v.x as u32;
    let g = v.y as u32;
    let b = v.z as u32;
    let a = v.w as u32;
    (r << 24) | (g << 16) | (b << 8) | a
}

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

pub fn draw_mesh(r: &mut Renderer, mesh: &Mesh, tex: &Texture) {
    // let mut rng = rand::thread_rng();

    let light_dir = Vector3::new(0.0, 0.0, -1.0);

    for (tri, texi) in mesh.vis.chunks_exact(3).zip(mesh.tis.chunks_exact(3)) {
        let w = (r.width - 1) as f32;
        let h = (r.height - 1) as f32;
        
        // world space vertices
        let vs: Vec<Vector3> = tri.iter().map(|i| {
            mesh.vs[*i as usize]
        }).collect();

        // project vertices into screen space points
        let pts: Vec<Vector3> = vs.iter().map(|v| {
            Vector3::new(
                (v.x + 1.0) * w / 2.0,
                (v.y + 1.0) * h / 2.0,
                v.z
            )
        }).collect();
        let uv: Vec<Vector2> = texi.iter().map(|i| {
            mesh.tex[*i as usize]
        }).collect();
        
        // normal
        let n = Vector3::cross(vs[2] - vs[0], vs[1] - vs[0]).normalize();
        // let intensity = (Vector3::dot(n, light_dir) * 255.0) as u32;
        let intensity = Vector3::dot(n, light_dir);

        if intensity > 0.0 {
            r.triangle_fill(pts, uv, tex, intensity);
        }
    }

}