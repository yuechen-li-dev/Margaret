use crate::image::ImageSize;
use crate::math::{Point3, Vec3};
use crate::ray::Ray;

const CAMERA_EPSILON: f32 = 0.000_001;

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
        self.ray_for_subpixel(image_size, pixel_x, pixel_y, 0.5, 0.5)
    }

    pub fn ray_for_subpixel(
        &self,
        image_size: ImageSize,
        pixel_x: u32,
        pixel_y: u32,
        subpixel_x: f32,
        subpixel_y: f32,
    ) -> Ray {
        assert!(image_size.width > 0, "camera image width must be non-zero");
        assert!(
            image_size.height > 0,
            "camera image height must be non-zero"
        );
        assert!(
            pixel_x < image_size.width,
            "camera pixel_x must be in bounds"
        );
        assert!(
            pixel_y < image_size.height,
            "camera pixel_y must be in bounds"
        );
        assert!(
            self.forward.length_squared() > CAMERA_EPSILON,
            "camera forward vector must be non-zero"
        );
        assert!(
            self.up.length_squared() > CAMERA_EPSILON,
            "camera up vector must be non-zero"
        );

        let forward = self.forward.normalized();
        let right_unnormalized = forward.cross(self.up);
        assert!(
            right_unnormalized.length_squared() > CAMERA_EPSILON,
            "camera forward and up vectors must not be parallel"
        );

        let right = right_unnormalized.normalized();
        let camera_up = right.cross(forward).normalized();

        let aspect_ratio = image_size.width as f32 / image_size.height as f32;
        let half_height = (self.vertical_fov_degrees.to_radians() * 0.5).tan();
        let half_width = half_height * aspect_ratio;

        assert!(
            (0.0..=1.0).contains(&subpixel_x),
            "camera subpixel_x must be in [0, 1]"
        );
        assert!(
            (0.0..=1.0).contains(&subpixel_y),
            "camera subpixel_y must be in [0, 1]"
        );

        let pixel_center_x = (pixel_x as f32 + subpixel_x) / image_size.width as f32;
        let pixel_center_y = (pixel_y as f32 + subpixel_y) / image_size.height as f32;

        let screen_x = (2.0 * pixel_center_x - 1.0) * half_width;
        let screen_y = (1.0 - 2.0 * pixel_center_y) * half_height;

        let direction = (forward + right * screen_x + camera_up * screen_y).normalized();
        Ray::new(self.position, direction)
    }
}

#[cfg(test)]
mod tests {
    use super::Camera;
    use crate::image::ImageSize;
    use crate::math::{Point3, Vec3};

    #[test]
    #[should_panic(expected = "camera image width must be non-zero")]
    fn ray_for_pixel_rejects_zero_width() {
        let camera = Camera::new("main", Point3::ORIGIN, -Vec3::Z, Vec3::Y, 45.0);

        let _ = camera.ray_for_pixel(ImageSize::new(0, 1), 0, 0);
    }

    #[test]
    #[should_panic(expected = "camera forward and up vectors must not be parallel")]
    fn ray_for_pixel_rejects_parallel_forward_and_up() {
        let camera = Camera::new("main", Point3::ORIGIN, Vec3::Y, Vec3::Y, 45.0);

        let _ = camera.ray_for_pixel(ImageSize::new(1, 1), 0, 0);
    }
}
