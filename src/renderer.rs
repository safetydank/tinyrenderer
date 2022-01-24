use std::{cmp, mem};
use glam::{Vec4Swizzles};

use crate::geometry::{Vector2, Vector2i, Vector3, Vector4, Matrix4, barycentric2, Matrix3};

use crate::util::{buf_index, color_from_vec4, vec4_from_color, vec3_normal_from_color, buf_index_yinvert, color_from_ndc_vec3};

#[derive(Clone, Copy, PartialEq)]
pub enum DisplayBuffer {
    Frame,
    Depth,
}

pub struct RendererState {
    pub display_buffer: DisplayBuffer,

    pub mesh: Mesh,
    pub diffuse: Texture,
    pub normal: Texture,

    pub model: Vector3,
    pub eye: Vector3,
    pub center: Vector3,
    pub up: Vector3,
    pub light_dir: Vector3,
    pub rotation: Vector3,
}

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
    pub shadow_matrix: Matrix4,
    pub light_dir: Vector3,
    pub varying_v: [Vector4; 3],
    pub varying_n: [Vector3; 3],
    pub varying_uv: [Vector2; 3],
    pub ndc_tri: [Vector3; 3],
    pub diffuse: &'a Texture,
    pub normal: &'a Texture,
    pub shadow: &'a Texture,
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
        let p = self.ndc_tri.iter().zip(bar.to_array())
            .map(|(v, w)| *v * w)
            .reduce(|l, r| l + r)
            .unwrap();

        let bn = self.varying_n.iter().zip(bar.to_array())
            .map(|(n, w)| *n * w)
            .reduce(|l, r| l + r)
            .unwrap()
            .normalize();

        let uv: Vector2 = self.varying_uv.iter().zip(bar.to_array())
            .map(|(tex, w)| *tex * w)
            .reduce(|l, r| l + r)
            .unwrap();
        
        let a = Matrix3::from_cols(self.ndc_tri[1] - self.ndc_tri[0], self.ndc_tri[2] - self.ndc_tri[0], bn).transpose();
        let ai = a.inverse();
        
        let i = ai * Vector3::new(self.varying_uv[1].x - self.varying_uv[0].x, self.varying_uv[2].x - self.varying_uv[0].x, 0.0);
        let j = ai * Vector3::new(self.varying_uv[1].y - self.varying_uv[0].y, self.varying_uv[2].y - self.varying_uv[0].y, 0.0);

        let b = Matrix3::from_cols(i.normalize(), j.normalize(), bn);
        
        // Normal map lookup + perturb
        let n = (b * (vec3_normal_from_color(self.normal.sample_nn(uv.x, uv.y)))).normalize();
        // let n = vec3_gl_from_color(self.normal.sample_nn(uv.x, uv.y)).normalize();

        //  diffuse lighting intensity
        // let diffuse = f32::max(0.0, Vector3::dot(bn, self.light_dir));
        let diffuse = f32::max(0.0, Vector3::dot(n, self.light_dir));
        let d2 = 1.0 - (1.0 - diffuse) * (1.0 - diffuse);

        //  look up shadow factor
        let mut sbp = self.shadow_matrix * Vector4::new(p.x, p.y, p.z, 1.0);
        sbp = sbp / sbp.w;
        let shadow_map_z = vec3_normal_from_color(self.shadow.lookup_yinvert(sbp.x, sbp.y));
        let shadow = if shadow_map_z.z < sbp.z { 1.0 } else { 0.3 };

        let c = vec4_from_color(self.diffuse.sample_lerp(uv.x, uv.y)).xyz() * d2 * shadow;
        // let c = vec4_from_color(self.diffuse.sample_nn(uv.x, uv.y)).xyz() * diffuse;
        // let c = vec4_from_color(self.normal.sample_lerp(uv.x, uv.y)).xyz() * diffuse;
        // let c = Vector3::ONE * 255.0 * diffuse;
        
        *frag = color_from_vec4(Vector4::new(c.x, c.y, c.z, 255.0));

        false
    }
}

pub struct DepthShader {
    pub projection: Matrix4,
    pub modelview: Matrix4,
    // pub viewport: Matrix4,
    pub varying_v: [Vector4; 3],
    pub ndc_tri: [Vector3; 3],
}

impl Shader for DepthShader {
    fn vertex(&mut self, v: Vector4, n: Vector4, uv: Vector2, tri_index: usize) -> Vector4 {
        let gl_vertex = self.projection * self.modelview * v;
        self.varying_v[tri_index] = gl_vertex;
        self.ndc_tri[tri_index] = (gl_vertex / gl_vertex.w).xyz();
        // println!("gl vertex {}", gl_vertex);
        gl_vertex
    }

    fn fragment(&self, bar: Vector3, frag: &mut u32) -> bool {
        let p = self.ndc_tri.iter().zip(bar.to_array())
            .map(|(v, w)| *v * w)
            .reduce(|l, r| l + r)
            .unwrap();
        // XXX 2000.0 ???
        let color = Vector3::ONE * p.z;
        *frag = color_from_ndc_vec3(color);

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
    
    pub fn lookup_yinvert(&self, x: f32, y: f32) -> u32 {
        let index = buf_index_yinvert(x as usize, y as usize, self.width as usize, self.height as usize);
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
    pub vertex: usize,
    pub tex: usize,
    pub normal: usize
}

impl Index {
    pub fn new(vertex: usize, tex: usize, normal: usize) -> Self {
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
    let z = (eye-center).normalize();
    let x = Vector3::cross(up,z).normalize();
    let y = Vector3::cross(z,x).normalize();
    let x_axis = Vector4::new(x.x, x.y, x.z, 0.0);
    let y_axis = Vector4::new(y.x, y.y, y.z, 0.0);
    let z_axis = Vector4::new(z.x, z.y, z.z, 0.0);
    let w_axis = Vector4::new(-center.x, -center.y, -center.z, 1.0);
    Matrix4::from_cols(x_axis, y_axis, z_axis, w_axis).transpose()
}

pub fn look_at_glam(eye: Vector3, center: Vector3, up: Vector3) -> Matrix4 {
    let mv = Matrix4::look_at_rh(eye, center, up);
    let w = Vector4::new(0.0, 0.0, 0.0, 1.0);
    Matrix4::from_cols(mv.col(0), mv.col(1), mv.col(2), w)
}

pub fn projection(coeff: f32) -> Matrix4 {
    let mut m = Matrix4::IDENTITY;
    m.col_mut(2)[3] = coeff;
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
    
    pub fn clear(&mut self) {
        for pixel in &mut self.buf {
            *pixel = 0xff;
        }
        for z in &mut self.zbuf {
            *z = 0.0;
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

    pub fn draw_shadow_mesh_shader(&mut self, renderer_state: &RendererState) {
        let RendererState {
            eye,
            center,
            up,
            light_dir,
            ..
        } = *renderer_state;
        let RendererState {
            mesh,
            diffuse,
            normal,
            ..
        } = renderer_state;
        
        self.viewport = viewport(
            self.width as f32 / 8.0, self.height as f32 / 8.0,
            self.width as f32 * 3.0/4.0, self.height as f32 * 3.0/4.0
        );

        //  Draw depth scene from light POV
        let mut depth_shader = DepthShader{
            projection: projection(0.0),
            modelview: look_at_glam(light_dir, center, up.normalize()),
            // viewport: self.viewport,
            varying_v: [Vector4::ZERO; 3],
            ndc_tri: [Vector3::ZERO; 3],
        };

        println!("DEPTH: vp {}\nproj {}\nmv {}\n", self.viewport, depth_shader.projection, depth_shader.modelview);
        // println!("glam mv {}", Matrix4::look_at_rh(eye, center, up));

        let mut pts: [Vector4; 3] = [Vector4::ZERO; 3];
        let mut vs: [Vector4; 3] = [Vector4::ZERO; 3];
        let mut ns: [Vector4; 3] = [Vector4::ZERO; 3];
        let mut uvs: [Vector2; 3] = [Vector2::ZERO; 3];

        for tri_indexes in mesh.indexes.chunks_exact(3) {
            for (i, index) in tri_indexes.iter().enumerate() {
                let v = mesh.vs[index.vertex];
                vs[i] = Vector4::new(v.x, v.y, v.z, 1.0);
            }
            
            // project vertices into screen space points (same as shader's varying_v)
            for i in 0..vs.len() {
                pts[i] = depth_shader.vertex(vs[i], Vector4::ZERO, Vector2::ZERO, i);
            }
            
            self.triangle_shade(&depth_shader, pts);
        }
        
        //  Copy framebuffer to shadow texture
        let shadow_texture = Texture{
            width: self.width as f32,
            height: self.height as f32,
            buf: self.buf.clone(),
        };
        shadow_texture.log_debug();
        
        //  Now draw scene with Phong shader and shadow map
        let m = self.viewport * depth_shader.projection * depth_shader.modelview;
        let projection = projection(1.0 / (eye - center).length());
        let modelview = look_at_glam(eye, center, up.normalize());

        let mut shader = PhongShader{
            projection,
            modelview,
            light_dir: light_dir.normalize(),
            varying_v: [Vector4::ZERO; 3],
            varying_n: [Vector3::ZERO; 3],
            varying_uv: [Vector2::ZERO; 3],
            ndc_tri: [Vector3::ZERO; 3],
            diffuse,
            normal,
            shadow_matrix: m * (projection * modelview).inverse(),
            shadow: &shadow_texture
        };

        println!("PHONG: vp {}\nproj {}\nmv {}\n", self.viewport, shader.projection, shader.modelview);
        self.clear();

        for tri_indexes in mesh.indexes.chunks_exact(3) {
            for (i, index) in tri_indexes.iter().enumerate() {
                let v = mesh.vs[index.vertex];
                vs[i] = Vector4::new(v.x, v.y, v.z, 1.0);
                let n = mesh.ns[index.normal];
                ns[i] = Vector4::new(n.x, n.y, n.z, 0.0);
                uvs[i] = mesh.tex[index.tex];
            }
            
            // project vertices into screen space points (same as shader's varying_v)
            for i in 0..vs.len() {
                pts[i] = shader.vertex(vs[i], ns[i], uvs[i], i);
            }
            
            self.triangle_shade(&shader, pts);
        }
    }

    pub fn draw_mesh_shader(&mut self, renderer_state: &RendererState) {
        let RendererState {
            eye,
            center,
            up,
            light_dir,
            ..
        } = *renderer_state;
        let RendererState {
            mesh,
            diffuse,
            normal,
            ..
        } = renderer_state;
        
        self.viewport = viewport(
            self.width as f32 / 8.0, self.height as f32 / 8.0,
            self.width as f32 * 3.0/4.0, self.height as f32 * 3.0/4.0
        );

        let mut shader = PhongShader{
            projection: projection(1.0 / (eye - center).length()),
            modelview: look_at_glam(eye, center, up.normalize()),
            light_dir: light_dir.normalize(),
            varying_v: [Vector4::ZERO; 3],
            varying_n: [Vector3::ZERO; 3],
            varying_uv: [Vector2::ZERO; 3],
            ndc_tri: [Vector3::ZERO; 3],
            diffuse,
            normal,
            shadow_matrix: todo!(),
            shadow: todo!(),
        };

        println!("vp {}\nproj {}\nmv {}\n", self.viewport, shader.projection, shader.modelview);
        // println!("glam mv {}", Matrix4::look_at_rh(eye, center, up));

        let mut pts: [Vector4; 3] = [Vector4::ZERO; 3];
        let mut vs: [Vector4; 3] = [Vector4::ZERO; 3];
        let mut ns: [Vector4; 3] = [Vector4::ZERO; 3];
        let mut uvs: [Vector2; 3] = [Vector2::ZERO; 3];

        for tri_indexes in mesh.indexes.chunks_exact(3) {
            for (i, index) in tri_indexes.iter().enumerate() {
                let v = mesh.vs[index.vertex];
                vs[i] = Vector4::new(v.x, v.y, v.z, 1.0);
                let n = mesh.ns[index.normal];
                ns[i] = Vector4::new(n.x, n.y, n.z, 0.0);
                uvs[i] = mesh.tex[index.tex];
            }
            
            // project vertices into screen space points (same as shader's varying_v)
            for i in 0..vs.len() {
                pts[i] = shader.vertex(vs[i], ns[i], uvs[i], i);
            }
            
            self.triangle_shade(&shader, pts);
        }
    }

    pub fn triangle_shade(&mut self, shader: &impl Shader, clipc: [Vector4; 3]) {
        let pts = clipc.map(|v| self.viewport * v);
        let pts2 = pts.map(|v| v.xy() / v.w);
        // println!("Triangle {} {} {}", pts[0], pts[1], pts[2]);

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
                let zindex = buf_index_yinvert(x as usize, y as usize, self.width as usize, self.height as usize);
                if zindex > ((self.width * self.height) as usize) {
                    println!("ERROR? zindex {} x {} y {} width {} height {}", zindex, x, y, self.width, self.height);
                }
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

    pub fn draw(&self, frame: &mut [u8], display_buffer: DisplayBuffer) {
        let mut draw_buf = |buf: &Vec<u32>| {
            for (b, p) in buf.iter().zip(frame.chunks_exact_mut(4)) {
                p.copy_from_slice(&b.to_be_bytes());
            }
        };

        match display_buffer {
            DisplayBuffer::Frame => draw_buf(&self.buf),
            DisplayBuffer::Depth => {
                let zbuf = self.zbuf_buf();
                draw_buf(&zbuf);
            }
        };
    }

    pub fn zbuf_buf(&self) -> Vec<u32> {
        self.zbuf.iter().map(|z| {
            let mut vc = *z * Vector4::ONE;
            vc.w = 255.0;
            color_from_vec4(vc)
        }).collect()
    }
}

