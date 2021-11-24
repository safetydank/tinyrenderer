use std::{cmp, mem};
use glam::IVec2;

use crate::geometry::{barycentric};

pub struct Renderer {
    pub width: i32,
    pub height: i32,
    pub buf: Vec<u32>,
    pub zbuf: Vec<u32>
}

impl Renderer {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            buf: vec![0x000000ff; (width * height) as usize],
            zbuf: vec![0x000000ff; (width * height) as usize],
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

    pub fn triangle_fill(&mut self, tri: Vec<IVec2>, color: u32) {
        let mut bboxmin = IVec2::new(self.width-1,  self.height-1); 
        let mut bboxmax = IVec2::new(0, 0); 
        let clamp = IVec2::new(self.width-1, self.height-1); 
        for pt in tri.iter() {
            bboxmin.x = cmp::max(0,       cmp::min(bboxmin.x, pt.x)); 
            bboxmin.y = cmp::max(0,       cmp::min(bboxmin.y, pt.y)); 
            bboxmax.x = cmp::min(clamp.x, cmp::max(bboxmax.x, pt.x)); 
            bboxmax.y = cmp::min(clamp.y, cmp::max(bboxmax.y, pt.y)); 
        } 
        
        for x in bboxmin.x..bboxmax.x {
            for y in bboxmin.y..bboxmax.y {
                let p = IVec2::new(x, y);
                let bc_screen = barycentric(&tri, p);
                if bc_screen.x < 0.0 || bc_screen.y < 0.0 || bc_screen.z < 0.0  {
                    continue;
                }
                self.pixel(x, y, color);
            }
        }
    }

    pub fn draw(&self, frame: &mut [u8]) {
        for (b, p) in self.buf.iter().zip(frame.chunks_exact_mut(4)) {
            p.copy_from_slice(&b.to_be_bytes());
        }
    }
}

