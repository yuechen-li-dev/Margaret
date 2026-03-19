use crate::math::{Point3, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
}

impl Ray {
    pub const fn new(origin: Point3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    pub fn at(self, distance: f32) -> Point3 {
        self.origin + self.direction * distance
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitRecord {
    pub distance: f32,
    pub position: Point3,
    pub normal: Vec3,
    pub front_face: bool,
    pub triangle_index: usize,
}
