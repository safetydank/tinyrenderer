use std::mem;

pub struct Renderer {
    width: u32,
    height: u32,
    buf: Vec<u32>
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width: width,
            height: height,
            buf: vec![0; (width * height) as usize],
        }
    }
    
    pub fn pixel(&mut self, x: u32, y: u32, color: u32) {
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
                self.pixel(y as u32, x as u32, color)
            } else {
                self.pixel(x as u32, y as u32, color)
            }
            error2 += derror2;
            if error2 > dx {
                y += if y1 > y0 { 1 } else { -1 };
                error2 -= dx * 2;
            }
        }
    }

    pub fn draw(&self, frame: &mut [u8]) {
        for (b, p) in self.buf.iter().zip(frame.chunks_exact_mut(4)) {
            p.copy_from_slice(&b.to_be_bytes());
        }
    }
}

