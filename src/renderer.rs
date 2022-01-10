use std::{cmp, mem};
use glam::{Vec4Swizzles};

use crate::geometry::{Vector2, Vector2i, Vector3, Vector4, barycentric, Matrix4, barycentric2};

use crate::util::{buf_index, color_from_vec4, vec4_from_color};

pub trait Shader {
    fn vertex(&mut self, v: Vector4, n: Vector4, uv: Vector2, tri_index: usize) -> Vector4;
    fn fragment(&self, bar: Vector3, frag: &mut u32) -> bool;
}

// pub struct GouraudShader<'a> {
//     pub viewport: Matrix4,
//     pub projection: Matrix4,
//     pub modelview: Matrix4,
//     pub light_dir: Vector3,
//     pub varying_intensity: [f32; 3],
//     pub varying_uv: [Vector2; 3],
//     pub texture: &'a Texture
// }
// 
// impl Shader for GouraudShader<'_> {
//     fn vertex(&mut self, v: Vector4, n: Vector4, uv: Vector2, tri_index: usize) -> Vector4 {
//         let gl_vertex = self.viewport * self.projection * self.modelview * v;
//         let intensity = f32::max(0.0, Vector3::dot(n.xyz(), self.light_dir));
//         self.varying_intensity[tri_index] = intensity;
//         self.varying_uv[tri_index] = uv;
//         // println!("gl vertex {}", gl_vertex);
//         gl_vertex
//     }
// 
//     fn fragment(&self, bar: Vector3, frag: &mut u32) -> bool {
//         let intensity: f32 = self.varying_intensity.iter().zip(bar.to_array()).map(|(intensity, bc)| intensity*bc).sum();
//         let uv: Vector2 = self.varying_uv.iter().zip(bar.to_array())
//             .map(|(tex, w)| *tex * w)
//             .reduce(|l, r| l + r)
//             .unwrap();
//         let c = vec4_from_color(self.texture.sample_lerp(uv.x, uv.y)).xyz() * intensity;
//         *frag = color_from_vec4(Vector4::new(c.x, c.y, c.z, 255.0));
// 
//         false
//     }
// }

pub struct PhongShader<'a> {
    pub projection: Matrix4,
    pub modelview: Matrix4,
    pub light_dir: Vector3,
    pub varying_v: [Vector4; 3],
    pub varying_n: [Vector3; 3],
    pub varying_uv: [Vector2; 3],
    pub ndc_tri: [Vector3; 3],
    pub texture: &'a Texture
}

impl Shader for PhongShader<'_> {
    fn vertex(&mut self, v: Vector4, n: Vector4, uv: Vector2, tri_index: usize) -> Vector4 {
        let gl_vertex = self.projection * self.modelview * v;
        self.varying_v[tri_index] = gl_vertex;
        self.varying_uv[tri_index] = uv;
        self.varying_n[tri_index] = ((self.projection * self.modelview).inverse().transpose() * n).xyz();
        self.ndc_tri[tri_index] = (gl_vertex / gl_vertex.w).xyz();
        // println!("gl vertex {}", gl_vertex);
        gl_vertex
    }

    fn fragment(&self, bar: Vector3, frag: &mut u32) -> bool {
        let bn = self.varying_n.iter().zip(bar.to_array())
            .map(|(n, w)| (*n * w))
            .reduce(|l, r| l + r)
            .unwrap();
        let uv: Vector2 = self.varying_uv.iter().zip(bar.to_array())
            .map(|(tex, w)| *tex * w)
            .reduce(|l, r| l + r)
            .unwrap();
        let diffuse = f32::max(0.0, Vector3::dot(bn, self.light_dir));
        let c = vec4_from_color(self.texture.sample_lerp(uv.x, uv.y)).xyz() * diffuse;
        *frag = color_from_vec4(Vector4::new(c.x, c.y, c.z, 255.0));

        false
    }
}


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

pub struct Index {
    //  Indexes
    pub vertex: i32,
    pub tex: i32,
    pub normal: i32
}

impl Index {
    pub fn new(vertex: i32, tex: i32, normal: i32) -> Self {
        Self {
            vertex,
            tex,
            normal
        }
    }
}

pub struct Face {
    pub points: [Index; 3]
}

impl Face {
    pub fn new(p0: Index, p1: Index, p2: Index) -> Self {
        Self {
            points: [p0, p1, p2]
        }
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
    pub tis: Vec<i32>,
    //  normals
    pub ns: Vec<Vector3>,
    //  normal indices
    pub nis: Vec<i32>,
    pub indexes: Vec<Index>,
}

pub struct Renderer {
    pub width: i32,
    pub height: i32,
    pub buf: Vec<u32>,
    pub zbuf: Vec<f32>,
    pub viewport: Matrix4,
}

const DEPTH: f32 = 255.0;

pub fn viewport(x: f32, y: f32, w: f32, h: f32) -> Matrix4 {
    let mut m = Matrix4::IDENTITY;
    let col = m.col_mut(3);
    col[0] = x + w / 2.0;
    col[1] = y + h / 2.0;
    col[2] = DEPTH / 2.0;

    m.col_mut(0)[0] = w / 2.0;
    m.col_mut(1)[1] = h / 2.0;
    m.col_mut(2)[2] = DEPTH / 2.0;
    
    m
}

pub fn look_at(eye: Vector3, center: Vector3, up: Vector3) -> Matrix4 {
    // Matrix4::look_at_rh(eye, center, up)
    let z = (eye-center).normalize();
    let x = Vector3::cross(up,z).normalize();
    let y = Vector3::cross(z,x).normalize();
    let x_axis = Vector4::new(x.x, x.y, x.z, 0.0);
    let y_axis = Vector4::new(y.x, y.y, y.z, 0.0);
    let z_axis = Vector4::new(z.x, z.y, z.z, 0.0);
    let w_axis = Vector4::new(-center.x, -center.y, -center.z, 1.0);
    Matrix4::from_cols(x_axis, y_axis, z_axis, w_axis).transpose()
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
            zbuf: vec![0.0; (width * height) as usize],
            viewport: Matrix4::IDENTITY
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

    pub fn draw_mesh_shader(&mut self, mesh: &Mesh, tex: &Texture) {
        let eye = Vector3::new(0.0, 1.0, 3.0);
        let center = Vector3::new(0.0, 0.0, 0.0);
        let up = Vector3::new(0.0, 1.0, 0.0);
        self.viewport = viewport(
            self.width as f32 / 8.0, self.height as f32 / 8.0,
            self.width as f32 * 3.0/4.0, self.height as f32 * 3.0/4.0
        );

        let mut shader = PhongShader{
            projection: projection((eye - center).length()),
            modelview: look_at(eye, center, up),
            light_dir: Vector3::new(-1.0, 1.0, 1.0).normalize(),
            varying_v: [Vector4::ZERO; 3],
            varying_n: [Vector3::ZERO; 3],
            varying_uv: [Vector2::ZERO; 3],
            ndc_tri: [Vector3::ZERO; 3],
            texture: tex,
        };

        println!("vp {}\nproj {}\nmv {}\n", self.viewport, shader.projection, shader.modelview);

        let mut pts: [Vector4; 3] = [Vector4::ZERO; 3];
        for tri_indexes in mesh.indexes.chunks_exact(3) {
            // object space vertices
            let vs: Vec<Vector4> = tri_indexes.iter().map(|i| {
                let v = mesh.vs[i.vertex as usize];
                Vector4::new(v.x, v.y, v.z, 1.0)
            }).collect();

            // normals
            let ns: Vec<Vector4> = tri_indexes.iter().map(|i| {
                let n = mesh.ns[i.normal as usize].normalize();
                Vector4::new(n.x, n.y, n.z, 0.0)
            }).collect();

            // texture coords
            let uvs: Vec<Vector2> = tri_indexes.iter().map(|i| {
                mesh.tex[i.tex as usize]
            }).collect();
            
            // project vertices into screen space points
            for i in 0..vs.len() {
                pts[i] = shader.vertex(vs[i], ns[i], uvs[i], i);
            }
            
            self.triangle_shade(&shader, pts);
        }
    }

    pub fn triangle_shade(&mut self, shader: &impl Shader, clipc: [Vector4; 3]) {
        // println!("Triangle {} {} {}", pts[0], pts[1], pts[2]);
        let pts = clipc.map(|v| self.viewport * v);
        let pts2 = pts.map(|v| v.xy() / v.w);

        let mut bboxmin = Vector2i::new(self.width-1,  self.height-1); 
        let mut bboxmax = Vector2i::new(0, 0); 
        let clamp = Vector2i::new(self.width-1, self.height-1); 
        for pt in pts2.iter() {
            bboxmin.x = cmp::max(0,       cmp::min(bboxmin.x, pt.x.floor() as i32)); 
            bboxmin.y = cmp::max(0,       cmp::min(bboxmin.y, pt.y.floor() as i32)); 
            bboxmax.x = cmp::min(clamp.x, cmp::max(bboxmax.x, pt.x.ceil() as i32)); 
            bboxmax.y = cmp::min(clamp.y, cmp::max(bboxmax.y, pt.y.ceil() as i32)); 
        } 
        
        for x in bboxmin.x..bboxmax.x {
            for y in bboxmin.y..bboxmax.y {
                let p = Vector2::new(x as f32, y as f32);
                let bc_screen = barycentric2(pts2[0], pts2[1], pts2[2], p);
                let mut bc_clip = Vector3::new(bc_screen.x / pts[0].w, bc_screen.y / pts[1].w, bc_screen.z / pts[2].w);
                bc_clip /= bc_clip.x + bc_clip.y + bc_clip.z;

                if bc_screen.x < 0.0 || bc_screen.y < 0.0 || bc_screen.z < 0.0 {
                    continue;
                }
                
                let frag_depth = Vector3::dot(Vector3::new(pts[0].z, pts[1].z, pts[2].z), bc_clip);
                // println!("Frag depth {}", frag_depth);
                let zindex = buf_index(x, y, self.width);
                if self.zbuf[zindex] > frag_depth {
                    continue
                }
                
                let mut color: u32 = 0;
                let discard = shader.fragment(bc_clip, &mut color);
                if !discard {
                    self.zbuf[zindex] = frag_depth;
                    self.pixel(x, y, color);
                }
            }
        }
    }

    pub fn draw(&self, frame: &mut [u8]) {
        for (b, p) in self.buf.iter().zip(frame.chunks_exact_mut(4)) {
            p.copy_from_slice(&b.to_be_bytes());
        }
    }

    pub fn zbuf_buf(&self) -> Vec<u32> {
        self.zbuf.iter().map(|z| {
            let mut vc = *z * Vector4::ONE;
            vc.w = 255.0;
            color_from_vec4(vc)
        }).collect()
    }
}

