use glam::{IVec2, Vec2, Vec3A, Vec3, Vec4, Mat3, Mat4};

pub type Vector3 = Vec3;
pub type Vector2 = Vec2;
pub type Vector2i = IVec2;
pub type Vector4 = Vec4;
pub type Matrix3 = Mat3;
pub type Matrix4 = Mat4;

pub fn barycentric(a: Vector3, b: Vector3, c: Vector3, p: Vector3) -> Vector3 { 
    let s0 = Vector3::new(c.y-a.y, b.y-a.y, a.y-p.y);
    let s1 = Vector3::new(c.x-a.x, b.x-a.x, a.x-p.x);
    let u = Vector3::cross(s0, s1);
    if u.z.abs() > 1.0e-2 {
        Vector3::new(1.0-(u.x+u.y)/u.z, u.y/u.z, u.x/u.z)
    } else {
        Vector3::new(-1.0, 1.0, 1.0)
    }
} 
