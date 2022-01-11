use std::path::Path;
use std::fs::File;
use std::io::BufWriter;
use std::ops::{Add, Mul};

use glam::Vec4Swizzles;

use crate::geometry::{Vector4, Vector3};
use crate::renderer::Texture;

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

pub fn vec4_gl_from_color(c: u32) -> Vector4 {
    vec4_from_color(c) / 255.0
}

pub fn vec3_gl_from_color(c: u32) -> Vector3 {
    vec4_gl_from_color(c).xyz()
}

pub fn color_from_vec4(v: Vector4) -> u32 {
    let r = v.x as u32;
    let g = v.y as u32;
    let b = v.z as u32;
    let a = v.w as u32;
    (r << 24) | (g << 16) | (b << 8) | a
}

//  Can't implement Into/From traits on primitive types ourselves
pub trait Cast<T>: {
    fn cast(&self) -> T;
}

impl Cast<usize> for i32 {
    fn cast(&self) -> usize { *self as usize }
}

impl Cast<usize> for f32 {
    fn cast(&self) -> usize { *self as usize }
}

pub fn buf_index<T: Mul<Output = T> + Add<Output = T> + Cast<usize>>(x: T, y: T, stride: T) -> usize {
    return (y * stride + x).cast();
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
