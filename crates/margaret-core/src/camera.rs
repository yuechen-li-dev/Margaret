use crate::image::ImageSize;
use crate::math::{Point3, Vec3};
use crate::ray::Ray;

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

    pub fn ray_for_pixel(&self, image_size: ImageSize, pixel_x: u32, pixel_y: u32) -> Ray {
        let forward = self.forward.normalized();
        let right = forward.cross(self.up).normalized();
        let camera_up = right.cross(forward).normalized();

        let aspect_ratio = image_size.width as f32 / image_size.height as f32;
        let half_height = (self.vertical_fov_degrees.to_radians() * 0.5).tan();
        let half_width = half_height * aspect_ratio;

        let pixel_center_x = (pixel_x as f32 + 0.5) / image_size.width as f32;
        let pixel_center_y = (pixel_y as f32 + 0.5) / image_size.height as f32;

        let screen_x = (2.0 * pixel_center_x - 1.0) * half_width;
        let screen_y = (1.0 - 2.0 * pixel_center_y) * half_height;

        let direction = (forward + right * screen_x + camera_up * screen_y).normalized();
        Ray::new(self.position, direction)
    }
}
