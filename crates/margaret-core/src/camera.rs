use crate::math::{Point3, Vec3};

#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    pub name: String,
    pub position: Point3,
    pub forward: Vec3,
    pub up: Vec3,
    pub vertical_fov_degrees: f32,
}

impl Camera {
    pub fn new(
        name: impl Into<String>,
        position: Point3,
        forward: Vec3,
        up: Vec3,
        vertical_fov_degrees: f32,
    ) -> Self {
        Self {
            name: name.into(),
            position,
            forward,
            up,
            vertical_fov_degrees,
        }
    }
}
