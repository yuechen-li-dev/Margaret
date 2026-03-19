use margaret_core::color::ColorRgba8;
use margaret_core::image::{ImageSize, OutputPixelFormat, RenderMetadata};
use margaret_core::ray::{HitRecord, Ray};
use margaret_core::scene::{Geometry, SceneDescription, Triangle};
use margaret_image::OwnedImage;

const EPSILON: f32 = 0.000_1;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CpuRendererBackend;

impl CpuRendererBackend {
    pub const fn new() -> Self {
        Self
    }

    pub const fn backend_name(&self) -> &'static str {
        "cpu"
    }

    pub fn describe_render(
        &self,
        scene: &SceneDescription,
        image_size: ImageSize,
    ) -> RenderMetadata {
        RenderMetadata {
            backend_name: self.backend_name().to_string(),
            scene_name: scene.name.clone(),
            image_size,
            pixel_format: OutputPixelFormat::Rgba8Unorm,
            sample_count: 1,
            object_count: scene.objects.len(),
            light_count: scene.lights.len(),
        }
    }

    pub fn render(&self, scene: &SceneDescription, image_size: ImageSize) -> OwnedImage {
        let mut image = OwnedImage::new(image_size, ColorRgba8::new(18, 24, 32, 255));

        for pixel_y in 0..image_size.height {
            for pixel_x in 0..image_size.width {
                let ray = scene.camera.ray_for_pixel(image_size, pixel_x, pixel_y);
                let color = match closest_hit(scene, ray) {
                    Some(hit) => shade_normal(hit.normal),
                    None => background_color(ray.direction.z),
                };
                image.set_pixel(pixel_x, pixel_y, color);
            }
        }

        image
    }
}

fn closest_hit(scene: &SceneDescription, ray: Ray) -> Option<HitRecord> {
    let mut closest_hit = None;
    let mut closest_distance = f32::INFINITY;

    for object in &scene.objects {
        match &object.geometry {
            Geometry::TriangleMesh { triangles } => {
                for (triangle_index, triangle) in triangles.iter().enumerate() {
                    let hit = intersect_triangle(ray, triangle, EPSILON, closest_distance);
                    if let Some(hit) = hit {
                        closest_distance = hit.distance;
                        closest_hit = Some(HitRecord {
                            triangle_index,
                            ..hit
                        });
                    }
                }
            }
        }
    }

    closest_hit
}

fn intersect_triangle(ray: Ray, triangle: &Triangle, t_min: f32, t_max: f32) -> Option<HitRecord> {
    let vertex0 = triangle.vertices[0];
    let vertex1 = triangle.vertices[1];
    let vertex2 = triangle.vertices[2];

    let edge1 = vertex1 - vertex0;
    let edge2 = vertex2 - vertex0;
    let pvec = ray.direction.cross(edge2);
    let determinant = edge1.dot(pvec);

    if determinant.abs() < EPSILON {
        return None;
    }

    let inverse_determinant = 1.0 / determinant;
    let tvec = ray.origin - vertex0;
    let u = tvec.dot(pvec) * inverse_determinant;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let qvec = tvec.cross(edge1);
    let v = ray.direction.dot(qvec) * inverse_determinant;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let distance = edge2.dot(qvec) * inverse_determinant;
    if distance < t_min || distance > t_max {
        return None;
    }

    let mut normal = edge1.cross(edge2).normalized();
    if normal.dot(ray.direction) > 0.0 {
        normal = -normal;
    }

    Some(HitRecord {
        distance,
        position: ray.at(distance),
        normal,
        triangle_index: 0,
    })
}

fn shade_normal(normal: margaret_core::math::Vec3) -> ColorRgba8 {
    let mapped = (normal + margaret_core::math::Vec3::new(1.0, 1.0, 1.0)) * 0.5;
    ColorRgba8::new(to_u8(mapped.x), to_u8(mapped.y), to_u8(mapped.z), 255)
}

fn background_color(direction_z: f32) -> ColorRgba8 {
    let blend = ((-direction_z).clamp(0.0, 1.0) * 0.35) + 0.15;
    let channel = to_u8(blend);
    ColorRgba8::new(channel / 2, channel, channel, 255)
}

fn to_u8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::{closest_hit, intersect_triangle, CpuRendererBackend};
    use margaret_core::math::{Point3, Vec3};
    use margaret_core::ray::Ray;
    use margaret_core::scene::{Geometry, Triangle};
    use margaret_testutil::{sample_image_size, sample_scene};

    #[test]
    fn describe_render_reports_basic_scene_counts() {
        let backend = CpuRendererBackend::new();
        let metadata = backend.describe_render(&sample_scene(), sample_image_size());

        assert_eq!(metadata.backend_name, "cpu");
        assert_eq!(metadata.object_count, 1);
        assert_eq!(metadata.light_count, 0);
    }

    #[test]
    fn ray_triangle_intersection_returns_expected_distance() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
        let triangle = Triangle::new(
            Point3::new(-1.0, -1.0, 0.0),
            Point3::new(1.0, -1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        );

        let hit = intersect_triangle(ray, &triangle, 0.001, f32::INFINITY).unwrap();

        assert!((hit.distance - 1.0).abs() < 0.0001);
        assert_eq!(hit.position, Point3::new(0.0, 0.0, 0.0));
        assert_eq!(hit.normal, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn ray_triangle_intersection_rejects_miss() {
        let ray = Ray::new(Point3::new(2.0, 2.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
        let triangle = Triangle::new(
            Point3::new(-1.0, -1.0, 0.0),
            Point3::new(1.0, -1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        );

        let hit = intersect_triangle(ray, &triangle, 0.001, f32::INFINITY);

        assert!(hit.is_none());
    }

    #[test]
    fn closest_hit_prefers_nearest_triangle() {
        let mut scene = sample_scene();
        scene.objects[0].geometry = Geometry::TriangleMesh {
            triangles: vec![
                Triangle::new(
                    Point3::new(-0.5, -0.5, 0.0),
                    Point3::new(0.5, -0.5, 0.0),
                    Point3::new(0.0, 0.5, 0.0),
                ),
                Triangle::new(
                    Point3::new(-0.5, -0.5, -1.0),
                    Point3::new(0.5, -0.5, -1.0),
                    Point3::new(0.0, 0.5, -1.0),
                ),
            ],
        };
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));

        let hit = closest_hit(&scene, ray).unwrap();

        assert!((hit.distance - 1.0).abs() < 0.0001);
        assert_eq!(hit.triangle_index, 0);
    }
}
