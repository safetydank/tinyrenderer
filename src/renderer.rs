use std::{cmp, mem};
use glam::{IVec2, Vec3A};

pub fn barycentric(a: Vec3A, b: Vec3A, c: Vec3A, p: Vec3A) -> Vec3A { 
    let s0 = Vec3A::new(c.y-a.y, b.y-a.y, a.y-p.y);
    let s1 = Vec3A::new(c.x-a.x, b.x-a.x, a.x-p.x);
    let u = Vec3A::cross(s0, s1);
    if u.z.abs() > 1.0e-2 {
        Vec3A::new(1.0-(u.x+u.y)/u.z, u.y/u.z, u.x/u.z)
    } else {
        Vec3A::new(-1.0, 1.0, 1.0)
    }
} 

pub struct Renderer {
    pub width: i32,
    pub height: i32,
    pub buf: Vec<u32>,
    pub zbuf: Vec<f32>
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

        let offset = ((self.height - y - 1) * self.width + x) as usize;
        self.buf[offset] = color;
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
    
    pub fn triangle(&mut self, t0: IVec2, t1: IVec2, t2: IVec2, color: u32) {
        self.line(t0.x, t0.y, t1.x, t1.y, color);
        self.line(t1.x, t1.y, t2.x, t2.y, color);
        self.line(t2.x, t2.y, t0.x, t0.y, color);
    }

    pub fn triangle_fill(&mut self, tri: Vec<Vec3A>, color: u32) {
        let mut bboxmin = IVec2::new(self.width-1,  self.height-1); 
        let mut bboxmax = IVec2::new(0, 0); 
        let clamp = IVec2::new(self.width-1, self.height-1); 
        for pt in tri.iter() {
            bboxmin.x = cmp::max(0,       cmp::min(bboxmin.x, pt.x.floor() as i32)); 
            bboxmin.y = cmp::max(0,       cmp::min(bboxmin.y, pt.y.floor() as i32)); 
            bboxmax.x = cmp::min(clamp.x, cmp::max(bboxmax.x, pt.x.ceil() as i32)); 
            bboxmax.y = cmp::min(clamp.y, cmp::max(bboxmax.y, pt.y.ceil() as i32)); 
        } 
        
        for x in bboxmin.x..bboxmax.x {
            for y in bboxmin.y..bboxmax.y {
                let p = Vec3A::new(x as f32, y as f32, 0.0);
                let bc = barycentric(tri[0], tri[1], tri[2], p);
                if bc.x < 0.0 || bc.y < 0.0 || bc.z < 0.0 {
                    continue;
                }
                let z = tri.iter()
                    .zip([bc.x, bc.y, bc.z])
                    .map(|(v, bc)| v.z * bc)
                    .sum();
                let zindex = (self.width * y + x) as usize;
                if self.zbuf[zindex] < z {
                    self.zbuf[zindex] = z;
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
}

