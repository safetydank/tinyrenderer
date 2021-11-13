#[derive(Clone, Copy)]
pub struct Vec3f {
   pub x: f32,
   pub y: f32,
   pub z: f32,
}

impl Vec3f {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    pub fn add(&mut self, v: Vec3f) -> Vec3f {
        Vec3f::new(self.x + v.x, self.y + v.y, self.z + v.z)
    }

    pub fn sub(&mut self, v: Vec3f) -> Vec3f {
        Vec3f::new(self.x - v.x, self.y - v.y, self.z - v.z)
    }
}

#[derive(Clone, Copy)]
pub struct Vec2i {
    pub x: i32,
    pub y: i32,
}

impl Vec2i {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn add(&self, v: Vec2i) -> Vec2i {
        Vec2i::new(self.x + v.x, self.y - v.y)
    }

    pub fn sub(&self, v: Vec2i) -> Vec2i {
        Vec2i::new(self.x - v.x, self.y - v.y)
    }
}

pub fn cross(v1: Vec3f, v2: Vec3f) -> Vec3f {
    Vec3f::new(v1.y*v2.z - v1.z*v2.y, v1.z*v2.x - v1.x*v2.z, v1.x*v2.y - v1.y*v2.x)
}

pub fn barycentric(pts: &Vec<Vec2i>, p: Vec2i) -> Vec3f { 
    let p1 = pts[2].sub(pts[0]);
    let p2 = pts[1].sub(pts[0]);
    let p3 = pts[0].sub(p);

    let u = cross(Vec3f::new(p1.x as f32, p2.x as f32, p3.x as f32), 
    Vec3f::new(p1.y as f32, p2.y as f32, p3.y as f32));
    if u.z.abs() < 1.0 {
        Vec3f{x: -1.0, y: 1.0, z: 1.0}
    } else {
        Vec3f{x: 1.0-(u.x+u.y)/u.z, y: u.y/u.z, z: u.x/u.z}
    }
} 

