use std::{cmp, mem};
use glam::{Vec4Swizzles};

use crate::geometry::{Vector2, Vector2i, Vector3, Vector4, barycentric, Matrix4};

use crate::util::{buf_index, color_from_vec4, vec4_from_color};

pub trait Shader {
    fn vertex(&mut self, v: Vector4, n: Vector4, tri_index: usize) -> Vector4;
    fn fragment(&self, bar: Vector3, frag: &mut u32) -> bool;
}

pub struct GouraudShader {
    pub viewport: Matrix4,
    pub projection: Matrix4,
    pub modelview: Matrix4,
    pub light_dir: Vector3,
    pub varying_intensity: [f32; 3]
}

impl Shader for GouraudShader {
    fn vertex(&mut self, v: Vector4, n: Vector4, tri_index: usize) -> Vector4 {
        let gl_vertex = self.viewport * self.projection * self.modelview * v;
        let intensity = f32::max(0.0, Vector3::dot(n.xyz(), self.light_dir));
        self.varying_intensity[tri_index] = intensity;
        // println!("gl vertex {}", gl_vertex);
        gl_vertex
    }

    fn fragment(&self, bar: Vector3, frag: &mut u32) -> bool {
        let intensity: f32 = self.varying_intensity.iter().zip(bar.to_array()).map(|(intensity, bc)| intensity*bc).sum();
        *frag = color_from_vec4(Vector4::new(255.0, 255.0, 255.0, 255.0) * intensity);

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
    pub zbuf: Vec<f32>
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

    pub fn triangle_shade(&mut self, shader: &impl Shader, pts: Vec<Vector4>) {
        // println!("Triangle {} {} {}", pts[0], pts[1], pts[2]);
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
                let bc = barycentric(pts[0].xyz() * (1.0 / pts[0].w), pts[1].xyz() * (1.0 / pts[1].w), pts[2].xyz() * (1.0 / pts[2].w), p);
                if bc.x < 0.0 || bc.y < 0.0 || bc.z < 0.0 {
                    continue;
                }
                println!("BC pts {} {} {} P {} == {}", pts[0], pts[1], pts[2], p, bc);
                // println!("DRAW");
                let bc_weights = [bc.x, bc.y, bc.z];
                // weighted z coord
                let z: f32 = pts.iter()
                    .zip(bc_weights)
                    .map(|(v, weight)| v.z * weight)
                    .sum();
                // weighted w coord
                let w: f32 = pts.iter()
                    .zip(bc_weights)
                    .map(|(v, weight)| v.w * weight)
                    .sum();
                let frag_depth = f32::max(0.0, f32::min(255.0, z/w));
                let zindex = buf_index(x, y, self.width);
                if self.zbuf[zindex] > frag_depth {
                    continue
                }
                
                let mut color: u32 = 0;
                let discard = shader.fragment(bc, &mut color);
                if !discard {
                    self.zbuf[zindex] = frag_depth;
                    self.pixel(x, y, color);
                    // self.pixel(x, y, 0xff0000ff);
                }
            }
        }
    }
    
    pub fn draw_mesh_shader(&mut self, mesh: &Mesh, tex: &Texture) {
        let eye = Vector3::new(0.0, -1.0, 3.0);
        let center = Vector3::new(0.0, 0.0, 0.0);
        let up = Vector3::new(0.0, 1.0, 0.0);

        let mut shader = GouraudShader{
            viewport: viewport(
                self.width as f32 / 8.0, self.height as f32 / 8.0,
                self.width as f32 * 3.0/4.0, self.height as f32 * 3.0/4.0
            ),
            projection: projection((eye - center).length()),
            modelview: look_at(eye, center, up),
            light_dir: Vector3::new(1.0, 1.0, 1.0).normalize(),
            varying_intensity: [0.0; 3],
        };

        println!("vp {}\nproj {}\nmv {}\n", shader.viewport, shader.projection, shader.modelview);
        // for (tri, texi, ni) in mesh.vis.chunks_exact(3).zip(mesh.tis.chunks_exact(3)).zip(mesh.nis.chunks_exact(3)) {
        for tri_indexes in mesh.indexes.chunks_exact(3) {
            // object space vertices
            let vs: Vec<Vector4> = tri_indexes.iter().map(|i| {
                let v = mesh.vs[i.vertex as usize];
                Vector4::new(v.x, v.y, v.z, 1.0)
            }).collect();

            // normals
            let ns: Vec<Vector4> = tri_indexes.iter().map(|i| {
                let n = mesh.ns[i.normal as usize].normalize();
                Vector4::new(n.x, n.y, n.z, 1.0)
            }).collect();

            // project vertices into screen space points
            let pts: Vec<Vector4> = vs.iter().zip(ns.iter()).enumerate().map(|(tri_index, (v, n))| {
                shader.vertex(*v, *n, tri_index)
            }).collect();
            // println!("Vertex intensities {} {} {}", shader.varying_intensity[0], shader.varying_intensity[1],  shader.varying_intensity[2]);
            
            self.triangle_shade(&shader, pts);

            // let uvs: Vec<Vector2> = texi.iter().map(|i| {
            //     mesh.tex[*i as usize]
            // }).collect();
            // 
            // // normal
            // let n = Vector3::cross(vs[2] - vs[0], vs[1] - vs[0]).normalize();
            // let intensity = Vector3::dot(n, light_dir);

            // if intensity > 0.0 {
            //     self.triangle_fill(pts, uvs, tex, intensity);
            // }
        }
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
        
        let eye = Vector3::new(1.0, 1.0, 3.0);
        let center = Vector3::new(0.0, 0.0, 0.0);
        let vp = viewport(0.0, 0.0, self.width as f32, self.height as f32);
        let proj = projection((eye - center).length());
        let modelview = look_at(eye, center, Vector3::new(0.0, 1.0, 0.0));

        for (tri, texi) in mesh.vis.chunks_exact(3).zip(mesh.tis.chunks_exact(3)) {
            // world space vertices
            let vs: Vec<Vector3> = tri.iter().map(|i| {
                mesh.vs[*i as usize]
            }).collect();

            // project vertices into screen space points
            let pts: Vec<Vector3> = vs.iter().map(|v| {
                let v = vp * proj * modelview * Vector4::new(v.x, v.y, v.z, 1.0);
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

