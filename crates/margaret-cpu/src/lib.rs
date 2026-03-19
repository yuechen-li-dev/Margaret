use margaret_core::color::{ColorRgb, ColorRgba8};
use margaret_core::image::{ImageSize, OutputPixelFormat, RenderDebugMode, RenderMetadata};
use margaret_core::material::{MaterialDescription, MaterialId};
use margaret_core::math::Vec3;
use margaret_core::ray::{HitRecord, Ray};
use margaret_core::scene::{Geometry, SceneDescription, Triangle};
use margaret_image::OwnedImage;

const DETERMINANT_EPSILON: f32 = 0.000_1;
const MIN_HIT_DISTANCE: f32 = 0.000_1;
const DEPTH_VISUALIZATION_MAX_DISTANCE: f32 = 6.0;
const MISS_COLOR: ColorRgba8 = ColorRgba8::new(18, 24, 32, 255);

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
        debug_mode: RenderDebugMode,
    ) -> RenderMetadata {
        RenderMetadata {
            backend_name: self.backend_name().to_string(),
            scene_name: scene.name.clone(),
            image_size,
            pixel_format: OutputPixelFormat::Rgba8Unorm,
            debug_mode,
            sample_count: 1,
            object_count: scene.objects.len(),
            light_count: scene.lights.len(),
        }
    }

    pub fn render(
        &self,
        scene: &SceneDescription,
        image_size: ImageSize,
        debug_mode: RenderDebugMode,
    ) -> OwnedImage {
        let mut image = OwnedImage::new(image_size, MISS_COLOR);

        for pixel_y in 0..image_size.height {
            for pixel_x in 0..image_size.width {
                let ray = scene.camera.ray_for_pixel(image_size, pixel_x, pixel_y);
                let color = match closest_hit(scene, ray) {
                    Some(hit) => shade_hit(scene, debug_mode, &hit),
                    None => MISS_COLOR,
                };
                image.set_pixel(pixel_x, pixel_y, color);
            }
        }

        image
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SceneHit {
    pub distance: f32,
    pub normal: Vec3,
    pub material_id: MaterialId,
}

fn shade_hit(scene: &SceneDescription, debug_mode: RenderDebugMode, hit: &SceneHit) -> ColorRgba8 {
    match debug_mode {
        RenderDebugMode::GeometricNormals => shade_normal(hit.normal),
        RenderDebugMode::FlatAlbedo => shade_albedo(scene, hit.material_id),
        RenderDebugMode::Depth => shade_depth(hit.distance),
    }
}

fn shade_normal(normal: Vec3) -> ColorRgba8 {
    let mapped = (normal + Vec3::new(1.0, 1.0, 1.0)) * 0.5;
    ColorRgba8::new(to_u8(mapped.x), to_u8(mapped.y), to_u8(mapped.z), 255)
}

fn shade_albedo(scene: &SceneDescription, material_id: MaterialId) -> ColorRgba8 {
    let material =
        find_material(scene, material_id).expect("scene hit referenced a missing material");
    color_rgb_to_rgba8(material.flat_albedo())
}

fn shade_depth(distance: f32) -> ColorRgba8 {
    let depth = (1.0 - (distance / DEPTH_VISUALIZATION_MAX_DISTANCE)).clamp(0.0, 1.0);
    let channel = to_u8(depth);
    ColorRgba8::new(channel, channel, channel, 255)
}

fn closest_hit(scene: &SceneDescription, ray: Ray) -> Option<SceneHit> {
    let mut closest_hit = None;
    let mut closest_distance = f32::INFINITY;

    for object in &scene.objects {
        match &object.geometry {
            Geometry::TriangleMesh { triangles } => {
                for triangle in triangles {
                    let hit = intersect_triangle(ray, triangle, MIN_HIT_DISTANCE, closest_distance);
                    if let Some(hit) = hit {
                        closest_distance = hit.distance;
                        closest_hit = Some(SceneHit {
                            distance: hit.distance,
                            normal: hit.normal,
                            material_id: object.material_id,
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

    if determinant.abs() < DETERMINANT_EPSILON {
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

    let normal = triangle.geometric_normal();

    Some(HitRecord {
        distance,
        position: ray.at(distance),
        normal,
        triangle_index: 0,
    })
}

fn color_rgb_to_rgba8(color: ColorRgb) -> ColorRgba8 {
    ColorRgba8::new(to_u8(color.r), to_u8(color.g), to_u8(color.b), 255)
}

fn find_material(
    scene: &SceneDescription,
    material_id: MaterialId,
) -> Option<&MaterialDescription> {
    scene
        .materials
        .iter()
        .find(|material| material.id == material_id)
}

fn to_u8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::{
        closest_hit, color_rgb_to_rgba8, find_material, intersect_triangle, shade_depth,
        CpuRendererBackend, DEPTH_VISUALIZATION_MAX_DISTANCE, MIN_HIT_DISTANCE, MISS_COLOR,
    };
    use margaret_core::camera::Camera;
    use margaret_core::color::{ColorRgb, ColorRgba8};
    use margaret_core::image::{ImageSize, RenderDebugMode};
    use margaret_core::material::{MaterialDescription, MaterialId, MaterialKind};
    use margaret_core::math::{Point3, Vec3};
    use margaret_core::ray::Ray;
    use margaret_core::scene::{Geometry, SceneDescription, SceneObject, Triangle};
    use margaret_testutil::sample_image_size;

    #[test]
    fn describe_render_reports_basic_scene_counts() {
        let backend = CpuRendererBackend::new();
        let metadata = backend.describe_render(
            &debug_scene(),
            sample_image_size(),
            RenderDebugMode::GeometricNormals,
        );

        assert_eq!(metadata.backend_name, "cpu");
        assert_eq!(metadata.debug_mode, RenderDebugMode::GeometricNormals);
        assert_eq!(metadata.object_count, 6);
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
    fn ray_triangle_intersection_keeps_geometric_normal_for_backface_hits() {
        let ray = Ray::new(Point3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 1.0));
        let triangle = Triangle::new(
            Point3::new(-1.0, -1.0, 0.0),
            Point3::new(1.0, -1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        );

        let hit = intersect_triangle(ray, &triangle, MIN_HIT_DISTANCE, f32::INFINITY).unwrap();

        assert_eq!(hit.normal, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn closest_hit_prefers_nearest_triangle() {
        let mut scene = debug_scene();
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
        assert_eq!(hit.material_id, MaterialId(2));
    }

    #[test]
    fn flat_albedo_mode_returns_material_color() {
        let backend = CpuRendererBackend::new();
        let image = backend.render(
            &debug_scene(),
            ImageSize::new(5, 5),
            RenderDebugMode::FlatAlbedo,
        );

        assert_eq!(image.get_pixel(2, 2), ColorRgba8::new(204, 204, 204, 255));
    }

    #[test]
    fn normals_mode_returns_mapped_normal_color() {
        let backend = CpuRendererBackend::new();
        let image = backend.render(
            &single_triangle_scene(),
            ImageSize::new(3, 3),
            RenderDebugMode::GeometricNormals,
        );

        assert_eq!(image.get_pixel(1, 1), ColorRgba8::new(128, 128, 255, 255));
    }

    #[test]
    fn depth_mode_brightens_nearer_hits_and_keeps_misses_dark() {
        let backend = CpuRendererBackend::new();
        let image = backend.render(
            &single_triangle_scene(),
            ImageSize::new(5, 5),
            RenderDebugMode::Depth,
        );

        assert_eq!(image.get_pixel(0, 0), MISS_COLOR);

        let center = image.get_pixel(2, 2);
        assert_eq!(center.r, center.g);
        assert_eq!(center.g, center.b);
        assert!(center.r > 0);
    }

    #[test]
    fn depth_shading_clamps_far_hits_to_black() {
        assert_eq!(
            shade_depth(DEPTH_VISUALIZATION_MAX_DISTANCE + 10.0),
            ColorRgba8::new(0, 0, 0, 255)
        );
    }

    #[test]
    fn find_material_returns_matching_material() {
        let scene = debug_scene();
        let material = find_material(&scene, MaterialId(2)).unwrap();

        assert_eq!(material.name, "white");
    }

    #[test]
    fn color_rgb_conversion_maps_unit_range_to_rgba8() {
        let color = color_rgb_to_rgba8(ColorRgb::new(0.25, 0.5, 0.75));

        assert_eq!(color, ColorRgba8::new(64, 128, 191, 255));
    }

    #[test]
    fn debug_scene_contains_box_like_triangle_meshes() {
        let scene = debug_scene();

        assert_eq!(scene.objects.len(), 6);

        for object in &scene.objects {
            match &object.geometry {
                Geometry::TriangleMesh { triangles } => assert_eq!(triangles.len(), 2),
            }
        }
    }

    fn single_triangle_scene() -> SceneDescription {
        let camera = Camera::new(
            "main",
            Point3::new(0.0, 0.0, 3.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::Y,
            45.0,
        );
        let material_id = MaterialId(0);

        let mut scene = SceneDescription::new("triangle-scene", camera);
        scene.materials.push(MaterialDescription::new(
            material_id,
            "matte-gray",
            MaterialKind::Diffuse {
                albedo: ColorRgb::new(0.5, 0.5, 0.5),
            },
        ));
        scene.objects.push(SceneObject::new(
            "preview-triangles",
            Geometry::TriangleMesh {
                triangles: vec![Triangle::new(
                    Point3::new(-1.0, -1.0, 0.0),
                    Point3::new(1.0, -1.0, 0.0),
                    Point3::new(0.0, 1.0, 0.0),
                )],
            },
            material_id,
        ));
        scene
    }

    fn debug_scene() -> SceneDescription {
        let camera = Camera::new(
            "main-camera",
            Point3::new(0.0, 0.0, 3.4),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::Y,
            40.0,
        );

        let red = MaterialId(0);
        let green = MaterialId(1);
        let white = MaterialId(2);

        let mut scene = SceneDescription::new("m1b-debug-scene", camera);
        scene.materials.push(MaterialDescription::new(
            red,
            "red",
            MaterialKind::Diffuse {
                albedo: ColorRgb::new(0.8, 0.2, 0.2),
            },
        ));
        scene.materials.push(MaterialDescription::new(
            green,
            "green",
            MaterialKind::Diffuse {
                albedo: ColorRgb::new(0.2, 0.8, 0.2),
            },
        ));
        scene.materials.push(MaterialDescription::new(
            white,
            "white",
            MaterialKind::Diffuse {
                albedo: ColorRgb::new(0.8, 0.8, 0.8),
            },
        ));

        scene.objects.push(make_quad(
            "floor",
            white,
            Point3::new(-1.2, -1.0, 1.2),
            Point3::new(1.2, -1.0, 1.2),
            Point3::new(1.2, -1.0, -1.2),
            Point3::new(-1.2, -1.0, -1.2),
        ));
        scene.objects.push(make_quad(
            "ceiling",
            white,
            Point3::new(-1.2, 1.0, -1.2),
            Point3::new(1.2, 1.0, -1.2),
            Point3::new(1.2, 1.0, 1.2),
            Point3::new(-1.2, 1.0, 1.2),
        ));
        scene.objects.push(make_quad(
            "back-wall",
            white,
            Point3::new(-1.2, -1.0, -1.2),
            Point3::new(1.2, -1.0, -1.2),
            Point3::new(1.2, 1.0, -1.2),
            Point3::new(-1.2, 1.0, -1.2),
        ));
        scene.objects.push(make_quad(
            "left-wall",
            red,
            Point3::new(-1.2, -1.0, -1.2),
            Point3::new(-1.2, -1.0, 1.2),
            Point3::new(-1.2, 1.0, 1.2),
            Point3::new(-1.2, 1.0, -1.2),
        ));
        scene.objects.push(make_quad(
            "right-wall",
            green,
            Point3::new(1.2, -1.0, 1.2),
            Point3::new(1.2, -1.0, -1.2),
            Point3::new(1.2, 1.0, -1.2),
            Point3::new(1.2, 1.0, 1.2),
        ));
        scene.objects.push(make_quad(
            "center-panel",
            white,
            Point3::new(-0.45, -1.0, -0.2),
            Point3::new(0.45, -1.0, -0.7),
            Point3::new(0.45, 0.2, -0.7),
            Point3::new(-0.45, 0.2, -0.2),
        ));

        scene
    }

    fn make_quad(
        name: &str,
        material_id: MaterialId,
        a: Point3,
        b: Point3,
        c: Point3,
        d: Point3,
    ) -> SceneObject {
        SceneObject::new(
            name,
            Geometry::TriangleMesh {
                triangles: vec![Triangle::new(a, b, c), Triangle::new(a, c, d)],
            },
            material_id,
        )
    }
}
