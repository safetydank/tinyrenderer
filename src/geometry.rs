use glam::{IVec2, Vec3A};

pub fn barycentric(pts: &Vec<IVec2>, p: IVec2) -> Vec3A { 
    let p1 = pts[2] - pts[0];
    let p2 = pts[1] - pts[0];
    let p3 = pts[0] - p;

    let u = Vec3A::cross(Vec3A::new(p1.x as f32, p2.x as f32, p3.x as f32), 
        Vec3A::new(p1.y as f32, p2.y as f32, p3.y as f32));
    if u.z.abs() < 1.0 {
        Vec3A::new(-1.0, 1.0, 1.0)
    } else {
        Vec3A::new(1.0-(u.x+u.y)/u.z, u.y/u.z, u.x/u.z)
    }
} 


