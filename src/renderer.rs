use std::{cmp, mem};
use glam::Vec4Swizzles;

use crate::geometry::{Vector2, Vector2i, Vector3, Vector4, barycentric, Matrix4};

use crate::util::{buf_index, color_from_vec4, vec4_from_color};

pub struct Texture {
    pub width: f32,
    pub height: f32,
    pub buf: Vec<u32>
}

impl Texture {
    pub fn lookup(&self, x: f32, y: f32) -> u32 {
        let index = buf_index(x, y, self.width);
        self.buf[index]
    }
    
    pub fn lookup_frag(&self, x: f32, y: f32) -> Vector4 {
        vec4_from_color(self.lookup(x, y))
    }
    
    // nearest neighbour
    pub fn sample_nn(&self, u: f32, v: f32) -> u32 {
        let x = u * self.width;
        let y = self.height - (v * self.height);
        self.lookup(x.floor(), y.floor())
    }    
    
    // 2D linear interpolation
    pub fn sample_lerp(&self, u: f32, v: f32) -> u32 {
        let x = u*self.width;
        let (x1, x2) = (x.floor(), x.ceil());
        let sx = x - x1;

        let y = self.height - (v * self.height);
        let (y1, y2) = (y.floor(), y.ceil());
        let sy = y - y1;
        
        let v1 = Vector4::lerp(self.lookup_frag(x1, y1), self.lookup_frag(x2, y1), sx);
        let v2 = Vector4::lerp(self.lookup_frag(x1, y2), self.lookup_frag(x2, y2), sx);

        color_from_vec4(Vector4::lerp(v1, v2, sy))
    }

    pub fn log_debug(&self) {
        println!("Texture width {} height {} buf<u32> len {}", self.width, self.height, self.buf.len());
    }
}

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

pub struct Renderer {
    pub width: i32,
    pub height: i32,
    pub buf: Vec<u32>,
    pub zbuf: Vec<f32>
}

const DEPTH: f32 = 255.0;

pub fn viewport(x: f32, y: f32, w: f32, h: f32) -> Matrix4 {
    let mut m = Matrix4::IDENTITY;
    m.col_mut(3)[0] = x + w / 2.0;
    m.col_mut(3)[1] = y + h / 2.0;
    m.col_mut(3)[2] = DEPTH / 2.0;

    m.col_mut(0)[0] = w / 2.0;
    m.col_mut(1)[1] = h / 2.0;
    m.col_mut(2)[2] = DEPTH / 2.0;
    
    m
}

pub fn projection(z: f32) -> Matrix4 {
    let mut m = Matrix4::IDENTITY;
    m.col_mut(2)[3] = -1.0 / z;
    m
}

impl Renderer {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            buf: vec![0x000000ff; (width * height) as usize],
            zbuf: vec![f32::MIN; (width * height) as usize],
        }
    }
    
    pub fn pixel(&mut self, x: i32, y: i32, color: u32) {
        //  clip pixels outside viewport
        if x < 0 || x >= self.width || y < 0 || y > self.height {
            return
        }

        let index = buf_index(x, self.height - y - 1, self.width);
        self.buf[index] = color;
    }
    
    pub fn line(&mut self, mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32, color: u32) {
        let steep = (x0-x1).abs() < (y0-y1).abs();
        if steep {
            mem::swap(&mut x0, &mut y0);
            mem::swap(&mut x1, &mut y1);
        }
        
        if x0 > x1 {
            mem::swap(&mut x0, &mut x1);
            mem::swap(&mut y0, &mut y1);
        }
        
        let dx = x1-x0;
        let dy = y1-y0;
        let derror2 = dy.abs() * 2;
        let mut error2 = 0;
        let mut y = y0;
        for x in x0..x1 {
            if steep {
                self.pixel(y, x, color);
            } else {
                self.pixel(x, y, color);
            }
            error2 += derror2;
            if error2 > dx {
                y += if y1 > y0 { 1 } else { -1 };
                error2 -= dx * 2;
            }
        }
    }
    
    pub fn triangle(&mut self, t0: Vector2i, t1: Vector2i, t2: Vector2i, color: u32) {
        self.line(t0.x, t0.y, t1.x, t1.y, color);
        self.line(t1.x, t1.y, t2.x, t2.y, color);
        self.line(t2.x, t2.y, t0.x, t0.y, color);
    }

    pub fn triangle_fill(&mut self, pts: Vec<Vector3>, uvs: Vec<Vector2>, tex: &Texture, intensity: f32) {
        let mut bboxmin = Vector2i::new(self.width-1,  self.height-1); 
        let mut bboxmax = Vector2i::new(0, 0); 
        let clamp = Vector2i::new(self.width-1, self.height-1); 
        for pt in pts.iter() {
            bboxmin.x = cmp::max(0,       cmp::min(bboxmin.x, pt.x.floor() as i32)); 
            bboxmin.y = cmp::max(0,       cmp::min(bboxmin.y, pt.y.floor() as i32)); 
            bboxmax.x = cmp::min(clamp.x, cmp::max(bboxmax.x, pt.x.ceil() as i32)); 
            bboxmax.y = cmp::min(clamp.y, cmp::max(bboxmax.y, pt.y.ceil() as i32)); 
        } 
        
        for x in bboxmin.x..bboxmax.x {
            for y in bboxmin.y..bboxmax.y {
                let p = Vector3::new(x as f32, y as f32, 0.0);
                let bc = barycentric(pts[0], pts[1], pts[2], p);
                if bc.x < 0.0 || bc.y < 0.0 || bc.z < 0.0 {
                    continue;
                }
                let bc_weights = [bc.x, bc.y, bc.z];
                // weighted z coord
                let z = pts.iter()
                    .zip(bc_weights)
                    .map(|(v, weight)| v.z * weight)
                    .sum();
                // weighted tex coords
                let texcoords = uvs.iter()
                    .zip(bc_weights)
                    .map(|(texcoord, weight)| *texcoord * weight)
                    .reduce(|l, r| l + r)
                    .unwrap();
                let zindex = buf_index(x, y, self.width);
                if self.zbuf[zindex] < z {
                    self.zbuf[zindex] = z;
                    let c = vec4_from_color(tex.sample_lerp(texcoords.x, texcoords.y)) * intensity;
                    self.pixel(x, y, color_from_vec4(c));
                }
            }
        }
    }

    pub fn draw(&self, frame: &mut [u8]) {
        for (b, p) in self.buf.iter().zip(frame.chunks_exact_mut(4)) {
            p.copy_from_slice(&b.to_be_bytes());
        }
    }

    pub fn draw_mesh(&mut self, mesh: &Mesh, tex: &Texture) {
        let light_dir = Vector3::new(0.0, 0.0, -1.0);

        for (tri, texi) in mesh.vis.chunks_exact(3).zip(mesh.tis.chunks_exact(3)) {
            let w = (self.width - 1) as f32;
            let h = (self.height - 1) as f32;
            
            // world space vertices
            let vs: Vec<Vector3> = tri.iter().map(|i| {
                mesh.vs[*i as usize]
            }).collect();

            // project vertices into screen space points
            let vp = viewport(0.0, 0.0, self.width as f32, self.height as f32);
            let proj = projection(3.0);

            let pts: Vec<Vector3> = vs.iter().map(|v| {
                let v = vp * proj * Vector4::new(v.x, v.y, v.z, 1.0);
                Vector3::new(v.x / v.w, v.y / v.w, v.z / v.w)
            }).collect();

            let uvs: Vec<Vector2> = texi.iter().map(|i| {
                mesh.tex[*i as usize]
            }).collect();
            
            // normal
            let n = Vector3::cross(vs[2] - vs[0], vs[1] - vs[0]).normalize();
            let intensity = Vector3::dot(n, light_dir);

            if intensity > 0.0 {
                self.triangle_fill(pts, uvs, tex, intensity);
            }
        }
    }
}

