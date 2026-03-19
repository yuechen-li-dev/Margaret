use margaret_core::color::{ColorRgb, ColorRgba8};
use margaret_core::image::{ImageSize, OutputPixelFormat, RenderMetadata};
use margaret_core::material::{MaterialDescription, MaterialId};
use margaret_core::math::{Point3, Vec3};
use margaret_core::ray::{HitRecord, Ray};
use margaret_core::render::{RenderDebugMode, RenderMode, RenderSettings};
use margaret_core::scene::{Geometry, SceneDescription, Triangle};
use margaret_image::OwnedImage;

const DETERMINANT_EPSILON: f32 = 0.000_1;
const MIN_HIT_DISTANCE: f32 = 0.000_1;
const SHADOW_BIAS: f32 = 0.001;
const MISS_COLOR: ColorRgba8 = ColorRgba8::new(18, 24, 32, 255);
const DEPTH_MISS_COLOR: ColorRgba8 = ColorRgba8::new(0, 0, 0, 255);
const INV_PI: f32 = 0.318_309_87;

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
            light_count: count_emissive_triangles(scene),
        }
    }

    pub fn render(
        &self,
        scene: &SceneDescription,
        image_size: ImageSize,
        render_settings: RenderSettings,
    ) -> OwnedImage {
        let mut image = OwnedImage::new(image_size, miss_color(render_settings.mode));
        let emissive_triangles = collect_emissive_triangles(scene);

        for pixel_y in 0..image_size.height {
            for pixel_x in 0..image_size.width {
                let ray = scene.camera.ray_for_pixel(image_size, pixel_x, pixel_y);
                let color = match closest_hit(scene, ray) {
                    Some(hit) => shade_hit(scene, render_settings, &hit, &emissive_triangles),
                    None => miss_color(render_settings.mode),
                };
                image.set_pixel(pixel_x, pixel_y, color);
            }
        }

        image
    }
}

fn miss_color(render_mode: RenderMode) -> ColorRgba8 {
    match render_mode {
        RenderMode::Debug(RenderDebugMode::Depth) => DEPTH_MISS_COLOR,
        RenderMode::Debug(RenderDebugMode::GeometricNormals)
        | RenderMode::Debug(RenderDebugMode::FlatAlbedo)
        | RenderMode::Lit => MISS_COLOR,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SceneHit {
    pub distance: f32,
    pub position: Point3,
    pub normal: Vec3,
    pub front_face: bool,
    pub material_id: MaterialId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct EmissiveTriangle {
    pub triangle: Triangle,
    pub radiance: ColorRgb,
}

fn shade_hit(
    scene: &SceneDescription,
    render_settings: RenderSettings,
    hit: &SceneHit,
    emissive_triangles: &[EmissiveTriangle],
) -> ColorRgba8 {
    match render_settings.mode {
        RenderMode::Debug(RenderDebugMode::GeometricNormals) => shade_normal(hit.normal),
        RenderMode::Debug(RenderDebugMode::FlatAlbedo) => shade_albedo(scene, hit.material_id),
        RenderMode::Debug(RenderDebugMode::Depth) => {
            shade_depth(hit.distance, render_settings.depth_max_distance)
        }
        RenderMode::Lit => color_rgb_to_rgba8(shade_lit(scene, hit, emissive_triangles)),
    }
}

fn shade_normal(normal: Vec3) -> ColorRgba8 {
    let mapped = (normal + Vec3::new(1.0, 1.0, 1.0)) * 0.5;
    ColorRgba8::new(to_u8(mapped.x), to_u8(mapped.y), to_u8(mapped.z), 255)
}

fn shade_albedo(scene: &SceneDescription, material_id: MaterialId) -> ColorRgba8 {
    let material =
        find_material(scene, material_id).expect("scene hit referenced a missing material");
    color_rgb_to_rgba8(material.diffuse_albedo())
}

fn shade_depth(distance: f32, depth_max_distance: f32) -> ColorRgba8 {
    assert!(
        depth_max_distance > 0.0,
        "depth max distance must be greater than zero"
    );

    let depth = (1.0 - (distance / depth_max_distance)).clamp(0.0, 1.0);
    let channel = to_u8(depth);
    ColorRgba8::new(channel, channel, channel, 255)
}

fn shade_lit(
    scene: &SceneDescription,
    hit: &SceneHit,
    emissive_triangles: &[EmissiveTriangle],
) -> ColorRgb {
    let material =
        find_material(scene, hit.material_id).expect("scene hit referenced a missing material");
    let mut radiance = visible_emissive_radiance(material, hit.front_face);

    if material.is_emissive() {
        return radiance;
    }

    let albedo = material.diffuse_albedo();

    for light in emissive_triangles {
        radiance += evaluate_direct_light(scene, hit, albedo, light);
    }

    radiance
}

fn visible_emissive_radiance(material: &MaterialDescription, front_face: bool) -> ColorRgb {
    if !material.is_emissive() {
        return ColorRgb::BLACK;
    }

    if front_face {
        material.emissive_radiance()
    } else {
        ColorRgb::BLACK
    }
}

fn evaluate_direct_light(
    scene: &SceneDescription,
    hit: &SceneHit,
    albedo: ColorRgb,
    light: &EmissiveTriangle,
) -> ColorRgb {
    // M2a uses one centroid sample per emissive triangle and weights it by area.
    let light_position = light.triangle.centroid();
    let to_light = light_position - hit.position;
    let distance_squared = to_light.length_squared();
    if distance_squared <= SHADOW_BIAS * SHADOW_BIAS {
        return ColorRgb::BLACK;
    }

    let light_direction = to_light.normalized();
    let surface_cosine = hit.normal.dot(light_direction);
    if surface_cosine <= 0.0 {
        return ColorRgb::BLACK;
    }

    let light_normal = light.triangle.geometric_normal();
    let light_cosine = light_normal.dot(-light_direction);
    if light_cosine <= 0.0 {
        return ColorRgb::BLACK;
    }

    let shadow_origin = hit.position + hit.normal * SHADOW_BIAS;
    let shadow_distance = (light_position - shadow_origin).length();
    let shadow_ray = Ray::new(shadow_origin, light_direction);

    if is_occluded(scene, shadow_ray, shadow_distance - SHADOW_BIAS) {
        return ColorRgb::BLACK;
    }

    let geometry_term = (surface_cosine * light_cosine * light.triangle.area()) / distance_squared;
    let brdf = albedo * INV_PI;
    brdf * light.radiance * geometry_term
}

fn is_occluded(scene: &SceneDescription, ray: Ray, max_distance: f32) -> bool {
    trace_hit(scene, ray, MIN_HIT_DISTANCE, max_distance).is_some()
}

fn closest_hit(scene: &SceneDescription, ray: Ray) -> Option<SceneHit> {
    let hit = trace_hit(scene, ray, MIN_HIT_DISTANCE, f32::INFINITY)?;

    Some(SceneHit {
        distance: hit.distance,
        position: hit.position,
        normal: hit.normal,
        front_face: hit.front_face,
        material_id: hit.material_id,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TraceHit {
    pub distance: f32,
    pub position: Point3,
    pub normal: Vec3,
    pub front_face: bool,
    pub material_id: MaterialId,
}

fn trace_hit(scene: &SceneDescription, ray: Ray, t_min: f32, t_max: f32) -> Option<TraceHit> {
    let mut closest_hit = None;
    let mut closest_distance = t_max;

    for object in &scene.objects {
        match &object.geometry {
            Geometry::TriangleMesh { triangles } => {
                for triangle in triangles {
                    let hit = intersect_triangle(ray, triangle, t_min, closest_distance);
                    if let Some(hit) = hit {
                        closest_distance = hit.distance;
                        closest_hit = Some(TraceHit {
                            distance: hit.distance,
                            position: hit.position,
                            normal: hit.normal,
                            front_face: hit.front_face,
                            material_id: object.material_id,
                        });
                    }
                }
            }
        }
    }

    closest_hit
}

fn collect_emissive_triangles(scene: &SceneDescription) -> Vec<EmissiveTriangle> {
    let mut lights = Vec::new();

    for object in &scene.objects {
        let material = find_material(scene, object.material_id)
            .expect("scene object referenced a missing material");
        if !material.is_emissive() {
            continue;
        }

        match &object.geometry {
            Geometry::TriangleMesh { triangles } => {
                for triangle in triangles {
                    lights.push(EmissiveTriangle {
                        triangle: *triangle,
                        radiance: material.emissive_radiance(),
                    });
                }
            }
        }
    }

    lights
}

fn count_emissive_triangles(scene: &SceneDescription) -> usize {
    collect_emissive_triangles(scene).len()
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
    let front_face = ray.direction.dot(normal) < 0.0;

    Some(HitRecord {
        distance,
        position: ray.at(distance),
        normal,
        front_face,
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
        closest_hit, color_rgb_to_rgba8, find_material, intersect_triangle, miss_color,
        shade_depth, CpuRendererBackend, SceneHit, DEPTH_MISS_COLOR, MIN_HIT_DISTANCE, MISS_COLOR,
    };
    use margaret_core::camera::Camera;
    use margaret_core::color::{ColorRgb, ColorRgba8};
    use margaret_core::image::ImageSize;
    use margaret_core::material::{MaterialDescription, MaterialId, MaterialKind};
    use margaret_core::math::{Point3, Vec3};
    use margaret_core::ray::Ray;
    use margaret_core::render::{RenderDebugMode, RenderMode, RenderSettings};
    use margaret_core::scene::{Geometry, SceneDescription, SceneObject, Triangle};
    use margaret_testutil::sample_image_size;

    #[test]
    fn describe_render_reports_basic_scene_counts() {
        let backend = CpuRendererBackend::new();
        let metadata = backend.describe_render(&lit_room_scene(), sample_image_size());

        assert_eq!(metadata.backend_name, "cpu");
        assert_eq!(metadata.object_count, 7);
        assert_eq!(metadata.light_count, 2);
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
        assert!(!hit.front_face);
    }

    #[test]
    fn closest_hit_prefers_nearest_triangle() {
        let mut scene = lit_room_scene();
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
            &lit_room_scene(),
            ImageSize::new(5, 5),
            RenderSettings::new(RenderMode::Debug(RenderDebugMode::FlatAlbedo), 6.0),
        );

        assert_eq!(image.get_pixel(2, 2), ColorRgba8::new(204, 204, 204, 255));
    }

    #[test]
    fn normals_mode_returns_mapped_normal_color() {
        let backend = CpuRendererBackend::new();
        let image = backend.render(
            &single_triangle_scene(),
            ImageSize::new(3, 3),
            RenderSettings::new(RenderMode::Debug(RenderDebugMode::GeometricNormals), 6.0),
        );

        assert_eq!(image.get_pixel(1, 1), ColorRgba8::new(128, 128, 255, 255));
    }

    #[test]
    fn depth_mode_brightens_nearer_hits_and_keeps_misses_dark() {
        let backend = CpuRendererBackend::new();
        let image = backend.render(
            &single_triangle_scene(),
            ImageSize::new(5, 5),
            RenderSettings::new(RenderMode::Debug(RenderDebugMode::Depth), 6.0),
        );

        assert_eq!(image.get_pixel(0, 0), DEPTH_MISS_COLOR);

        let center = image.get_pixel(2, 2);
        assert_eq!(center.r, center.g);
        assert_eq!(center.g, center.b);
        assert!(center.r > 0);
    }

    #[test]
    fn lit_mode_receives_emissive_triangle_contribution() {
        let scene = simple_lighting_scene(false);
        let hit = SceneHit {
            distance: 3.0,
            position: Point3::new(0.0, 0.0, 0.0),
            normal: Vec3::new(0.0, 0.0, 1.0),
            front_face: true,
            material_id: MaterialId(0),
        };
        let lights = super::collect_emissive_triangles(&scene);
        let color = super::shade_lit(&scene, &hit, &lights);

        assert!(color.r > 0.0);
        assert!(color.g > 0.0);
        assert!(color.b > 0.0);
    }

    #[test]
    fn lit_mode_returns_shadow_when_occluder_blocks_light() {
        let lit_scene = simple_lighting_scene(false);
        let shadowed_scene = simple_lighting_scene(true);
        let hit = SceneHit {
            distance: 3.0,
            position: Point3::new(0.0, 0.0, 0.0),
            normal: Vec3::new(0.0, 0.0, 1.0),
            front_face: true,
            material_id: MaterialId(0),
        };

        let lit_lights = super::collect_emissive_triangles(&lit_scene);
        let shadowed_lights = super::collect_emissive_triangles(&shadowed_scene);
        let lit_color = super::shade_lit(&lit_scene, &hit, &lit_lights);
        let shadowed_color = super::shade_lit(&shadowed_scene, &hit, &shadowed_lights);

        assert!(lit_color.r > shadowed_color.r);
        assert!(lit_color.g > shadowed_color.g);
        assert!(lit_color.b > shadowed_color.b);
    }

    #[test]
    fn lit_mode_shows_visible_emissive_geometry() {
        let backend = CpuRendererBackend::new();
        let image = backend.render(
            &emissive_only_scene(),
            ImageSize::new(5, 5),
            RenderSettings::new(RenderMode::Lit, 6.0),
        );

        let center = image.get_pixel(2, 2);
        assert_eq!(center, ColorRgba8::new(255, 242, 191, 255));
    }

    #[test]
    fn lit_mode_hides_emissive_backfaces() {
        let backend = CpuRendererBackend::new();
        let image = backend.render(
            &emissive_backface_scene(),
            ImageSize::new(5, 5),
            RenderSettings::new(RenderMode::Lit, 6.0),
        );

        assert_eq!(image.get_pixel(2, 2), ColorRgba8::new(0, 0, 0, 255));
    }

    #[test]
    fn backfacing_emissive_triangle_does_not_contribute_direct_light() {
        let hit = SceneHit {
            distance: 3.0,
            position: Point3::new(0.0, 0.0, 0.0),
            normal: Vec3::new(0.0, 0.0, 1.0),
            front_face: true,
            material_id: MaterialId(0),
        };

        let front_facing_scene = simple_lighting_scene(false);
        let back_facing_scene = simple_lighting_scene_with_light_winding(false);
        let front_facing_lights = super::collect_emissive_triangles(&front_facing_scene);
        let back_facing_lights = super::collect_emissive_triangles(&back_facing_scene);
        let front_facing_color = super::shade_lit(&front_facing_scene, &hit, &front_facing_lights);
        let back_facing_color = super::shade_lit(&back_facing_scene, &hit, &back_facing_lights);

        assert!(front_facing_color.r > 0.0);
        assert_eq!(back_facing_color, ColorRgb::BLACK);
    }

    #[test]
    fn centroid_sampled_direct_light_scales_with_emitter_area() {
        let hit = SceneHit {
            distance: 3.0,
            position: Point3::new(0.0, 0.0, 0.0),
            normal: Vec3::new(0.0, 0.0, 1.0),
            front_face: true,
            material_id: MaterialId(0),
        };

        let small_light_scene = simple_lighting_scene_with_light_half_extent(0.2);
        let large_light_scene = simple_lighting_scene_with_light_half_extent(0.4);
        let small_lights = super::collect_emissive_triangles(&small_light_scene);
        let large_lights = super::collect_emissive_triangles(&large_light_scene);
        let small_color = super::shade_lit(&small_light_scene, &hit, &small_lights);
        let large_color = super::shade_lit(&large_light_scene, &hit, &large_lights);

        assert!(large_color.r > small_color.r);
        assert!(large_color.g > small_color.g);
        assert!(large_color.b > small_color.b);
    }

    #[test]
    fn depth_shading_clamps_far_hits_to_black() {
        assert_eq!(shade_depth(16.0, 6.0), ColorRgba8::new(0, 0, 0, 255));
    }

    #[test]
    fn miss_color_uses_depth_consistent_black_for_depth_mode() {
        assert_eq!(
            miss_color(RenderMode::Debug(RenderDebugMode::Depth)),
            DEPTH_MISS_COLOR
        );
        assert_eq!(
            miss_color(RenderMode::Debug(RenderDebugMode::GeometricNormals)),
            MISS_COLOR
        );
        assert_eq!(miss_color(RenderMode::Lit), MISS_COLOR);
    }

    #[test]
    fn find_material_returns_matching_material() {
        let scene = lit_room_scene();
        let material = find_material(&scene, MaterialId(2)).unwrap();

        assert_eq!(material.name, "white");
    }

    #[test]
    fn color_rgb_conversion_maps_unit_range_to_rgba8() {
        let color = color_rgb_to_rgba8(ColorRgb::new(0.25, 0.5, 0.75));

        assert_eq!(color, ColorRgba8::new(64, 128, 191, 255));
    }

    #[test]
    fn lit_room_scene_contains_emissive_quad() {
        let scene = lit_room_scene();

        assert_eq!(scene.objects.len(), 7);

        for object in &scene.objects {
            match &object.geometry {
                Geometry::TriangleMesh { triangles } => assert_eq!(triangles.len(), 2),
            }
        }
    }

    fn emissive_only_scene() -> SceneDescription {
        let camera = Camera::new(
            "main",
            Point3::new(0.0, 0.0, 3.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::Y,
            45.0,
        );
        let emissive = MaterialId(0);

        let mut scene = SceneDescription::new("emissive-only", camera);
        scene.materials.push(MaterialDescription::new(
            emissive,
            "light",
            MaterialKind::Diffuse {
                albedo: ColorRgb::new(0.0, 0.0, 0.0),
                emission: ColorRgb::new(1.0, 0.95, 0.75),
            },
        ));
        scene.objects.push(SceneObject::new(
            "light",
            Geometry::TriangleMesh {
                triangles: vec![Triangle::new(
                    Point3::new(-0.5, -0.5, 0.0),
                    Point3::new(0.5, -0.5, 0.0),
                    Point3::new(0.0, 0.5, 0.0),
                )],
            },
            emissive,
        ));
        scene
    }

    fn emissive_backface_scene() -> SceneDescription {
        let camera = Camera::new(
            "main",
            Point3::new(0.0, 0.0, -3.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::Y,
            45.0,
        );
        let emissive = MaterialId(0);

        let mut scene = SceneDescription::new("emissive-backface", camera);
        scene.materials.push(MaterialDescription::new(
            emissive,
            "light",
            MaterialKind::Diffuse {
                albedo: ColorRgb::BLACK,
                emission: ColorRgb::new(1.0, 0.95, 0.75),
            },
        ));
        scene.objects.push(SceneObject::new(
            "light",
            Geometry::TriangleMesh {
                triangles: vec![Triangle::new(
                    Point3::new(-0.5, -0.5, 0.0),
                    Point3::new(0.5, -0.5, 0.0),
                    Point3::new(0.0, 0.5, 0.0),
                )],
            },
            emissive,
        ));
        scene
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
                emission: ColorRgb::BLACK,
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

    fn simple_lighting_scene(with_occluder: bool) -> SceneDescription {
        simple_lighting_scene_with_light_winding_and_extent(with_occluder, true, 0.35)
    }

    fn simple_lighting_scene_with_light_winding(front_facing: bool) -> SceneDescription {
        simple_lighting_scene_with_light_winding_and_extent(false, front_facing, 0.35)
    }

    fn simple_lighting_scene_with_light_half_extent(light_half_extent: f32) -> SceneDescription {
        simple_lighting_scene_with_light_winding_and_extent(false, true, light_half_extent)
    }

    fn simple_lighting_scene_with_light_winding_and_extent(
        with_occluder: bool,
        front_facing_light: bool,
        light_half_extent: f32,
    ) -> SceneDescription {
        let camera = Camera::new(
            "main",
            Point3::new(0.0, 0.0, 3.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::Y,
            45.0,
        );
        let diffuse = MaterialId(0);
        let emissive = MaterialId(1);
        let blocker = MaterialId(2);

        let mut scene = SceneDescription::new("simple-lighting", camera);
        scene.materials.push(MaterialDescription::new(
            diffuse,
            "diffuse",
            MaterialKind::Diffuse {
                albedo: ColorRgb::new(0.8, 0.8, 0.8),
                emission: ColorRgb::BLACK,
            },
        ));
        scene.materials.push(MaterialDescription::new(
            emissive,
            "light",
            MaterialKind::Diffuse {
                albedo: ColorRgb::BLACK,
                emission: ColorRgb::new(6.0, 6.0, 6.0),
            },
        ));
        scene.materials.push(MaterialDescription::new(
            blocker,
            "blocker",
            MaterialKind::Diffuse {
                albedo: ColorRgb::new(0.2, 0.2, 0.2),
                emission: ColorRgb::BLACK,
            },
        ));

        scene.objects.push(SceneObject::new(
            "receiver",
            Geometry::TriangleMesh {
                triangles: vec![
                    Triangle::new(
                        Point3::new(-1.0, -1.0, 0.0),
                        Point3::new(1.0, -1.0, 0.0),
                        Point3::new(1.0, 1.0, 0.0),
                    ),
                    Triangle::new(
                        Point3::new(-1.0, -1.0, 0.0),
                        Point3::new(1.0, 1.0, 0.0),
                        Point3::new(-1.0, 1.0, 0.0),
                    ),
                ],
            },
            diffuse,
        ));
        scene.objects.push(SceneObject::new(
            "light",
            Geometry::TriangleMesh {
                triangles: make_light_triangles(light_half_extent, front_facing_light),
            },
            emissive,
        ));

        if with_occluder {
            scene.objects.push(SceneObject::new(
                "occluder",
                Geometry::TriangleMesh {
                    triangles: vec![
                        Triangle::new(
                            Point3::new(-0.25, -0.25, 0.75),
                            Point3::new(0.25, -0.25, 0.75),
                            Point3::new(0.25, 0.25, 0.75),
                        ),
                        Triangle::new(
                            Point3::new(-0.25, -0.25, 0.75),
                            Point3::new(0.25, 0.25, 0.75),
                            Point3::new(-0.25, 0.25, 0.75),
                        ),
                    ],
                },
                blocker,
            ));
        }

        scene
    }

    fn make_light_triangles(light_half_extent: f32, front_facing_light: bool) -> Vec<Triangle> {
        let bottom_left = Point3::new(-light_half_extent, -light_half_extent, 1.5);
        let bottom_right = Point3::new(light_half_extent, -light_half_extent, 1.5);
        let top_left = Point3::new(-light_half_extent, light_half_extent, 1.5);
        let top_right = Point3::new(light_half_extent, light_half_extent, 1.5);

        if front_facing_light {
            vec![
                Triangle::new(bottom_left, top_right, bottom_right),
                Triangle::new(bottom_left, top_left, top_right),
            ]
        } else {
            vec![
                Triangle::new(bottom_left, bottom_right, top_right),
                Triangle::new(bottom_left, top_right, top_left),
            ]
        }
    }

    fn lit_room_scene() -> SceneDescription {
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
        let light = MaterialId(3);

        let mut scene = SceneDescription::new("m2a-lit-room-scene", camera);
        scene.materials.push(MaterialDescription::new(
            red,
            "red",
            MaterialKind::Diffuse {
                albedo: ColorRgb::new(0.8, 0.2, 0.2),
                emission: ColorRgb::BLACK,
            },
        ));
        scene.materials.push(MaterialDescription::new(
            green,
            "green",
            MaterialKind::Diffuse {
                albedo: ColorRgb::new(0.2, 0.8, 0.2),
                emission: ColorRgb::BLACK,
            },
        ));
        scene.materials.push(MaterialDescription::new(
            white,
            "white",
            MaterialKind::Diffuse {
                albedo: ColorRgb::new(0.8, 0.8, 0.8),
                emission: ColorRgb::BLACK,
            },
        ));
        scene.materials.push(MaterialDescription::new(
            light,
            "ceiling-light",
            MaterialKind::Diffuse {
                albedo: ColorRgb::BLACK,
                emission: ColorRgb::new(5.0, 4.8, 4.4),
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
        scene.objects.push(make_quad(
            "light",
            light,
            Point3::new(-0.35, 0.99, -0.35),
            Point3::new(0.35, 0.99, -0.35),
            Point3::new(0.35, 0.99, 0.35),
            Point3::new(-0.35, 0.99, 0.35),
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
