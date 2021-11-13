use std::{cmp, mem};
use crate::geometry::{Vec3f, Vec2i, barycentric};

pub struct Renderer {
    pub width: i32,
    pub height: i32,
    pub buf: Vec<u32>
}

impl Renderer {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            buf: vec![0x000000ff; (width * height) as usize],
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
    
    pub fn triangle(&mut self, t0: Vec2i, t1: Vec2i, t2: Vec2i, color: u32) {
        self.line(t0.x, t0.y, t1.x, t1.y, color);
        self.line(t1.x, t1.y, t2.x, t2.y, color);
        self.line(t2.x, t2.y, t0.x, t0.y, color);
    }

    pub fn triangle_fill(&mut self, tri: Vec<Vec2i>, color: u32) {
        let mut bboxmin = Vec2i::new(self.width-1,  self.height-1); 
        let mut bboxmax = Vec2i::new(0, 0); 
        let clamp = Vec2i::new(self.width-1, self.height-1); 
        for pt in tri.iter() {
            bboxmin.x = cmp::max(0,       cmp::min(bboxmin.x, pt.x)); 
            bboxmin.y = cmp::max(0,       cmp::min(bboxmin.y, pt.y)); 
            bboxmax.x = cmp::min(clamp.x, cmp::max(bboxmax.x, pt.x)); 
            bboxmax.y = cmp::min(clamp.y, cmp::max(bboxmax.y, pt.y)); 
        } 
        for x in bboxmin.x..bboxmax.x {
            for y in bboxmin.y..bboxmax.y {
                let p = Vec2i::new(x, y);
                let bc_screen = barycentric(&tri, p);
                if bc_screen.x < 0.0 || bc_screen.y < 0.0 || bc_screen.z < 0.0  {
                    continue;
                }
                self.pixel(x, y, color);
            }

        }
        // for (P.x=bboxmin.x; P.x<=bboxmax.x; P.x++) { 
        //     for (P.y=bboxmin.y; P.y<=bboxmax.y; P.y++) { 
        //         Vec3f bc_screen  = barycentric(pts, P); 
        //         if (bc_screen.x<0 || bc_screen.y<0 || bc_screen.z<0) continue; 
        //         image.set(P.x, P.y, color); 
        //     } 
        // } 
    }
    
    pub fn draw(&self, frame: &mut [u8]) {
        for (b, p) in self.buf.iter().zip(frame.chunks_exact_mut(4)) {
            p.copy_from_slice(&b.to_be_bytes());
        }
    }
}

