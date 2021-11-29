use glam::{IVec2, Vec2, Vec3A};

type Vector3 = Vec3A;
type Vector2 = Vec2;
type Vector2i = IVec2;

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
