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
const LIT_SAMPLES_PER_PIXEL: u32 = 16;
const MAX_DIFFUSE_BOUNCES: u32 = 3;

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
        render_settings: RenderSettings,
    ) -> RenderMetadata {
        let sample_count = match render_settings.mode {
            RenderMode::Lit => LIT_SAMPLES_PER_PIXEL,
            RenderMode::Debug(_) => 1,
        };

        RenderMetadata {
            backend_name: self.backend_name().to_string(),
            scene_name: scene.name.clone(),
            image_size,
            pixel_format: OutputPixelFormat::Rgba8Unorm,
            sample_count,
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
                let color = match render_settings.mode {
                    RenderMode::Lit => {
                        render_lit_pixel(scene, image_size, pixel_x, pixel_y, &emissive_triangles)
                    }
                    RenderMode::Debug(_) => {
                        let ray = scene.camera.ray_for_pixel(image_size, pixel_x, pixel_y);
                        match closest_hit(scene, ray) {
                            Some(hit) => {
                                shade_hit(scene, render_settings, &hit, &emissive_triangles)
                            }
                            None => miss_color(render_settings.mode),
                        }
                    }
                };
                image.set_pixel(pixel_x, pixel_y, color);
            }
        }

        image
    }
}

fn render_lit_pixel(
    scene: &SceneDescription,
    image_size: ImageSize,
    pixel_x: u32,
    pixel_y: u32,
    emissive_triangles: &[EmissiveTriangle],
) -> ColorRgba8 {
    let mut rng = PixelRng::new(pixel_x, pixel_y);
    let mut radiance = ColorRgb::BLACK;

    for _sample_index in 0..LIT_SAMPLES_PER_PIXEL {
        let ray = scene.camera.ray_for_subpixel(
            image_size,
            pixel_x,
            pixel_y,
            rng.next_f32(),
            rng.next_f32(),
        );
        radiance += trace_diffuse_path(scene, ray, emissive_triangles, &mut rng);
    }

    let average_radiance = radiance * (1.0 / LIT_SAMPLES_PER_PIXEL as f32);
    color_rgb_to_rgba8(average_radiance)
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct PixelRng {
    pub state: u64,
}

impl PixelRng {
    fn new(pixel_x: u32, pixel_y: u32) -> Self {
        let seed = ((pixel_x as u64 + 1) << 32) ^ (pixel_y as u64 + 1) ^ 0x9E37_79B9_7F4A_7C15;
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.state >> 32) as u32
    }

    fn next_f32(&mut self) -> f32 {
        let value = self.next_u32();
        value as f32 / u32::MAX as f32
    }
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

fn trace_diffuse_path(
    scene: &SceneDescription,
    initial_ray: Ray,
    emissive_triangles: &[EmissiveTriangle],
    rng: &mut PixelRng,
) -> ColorRgb {
    let mut ray = initial_ray;
    let mut throughput = ColorRgb::WHITE;
    let mut radiance = ColorRgb::BLACK;

    for _bounce_index in 0..MAX_DIFFUSE_BOUNCES {
        let Some(hit) = closest_hit(scene, ray) else {
            break;
        };

        let material =
            find_material(scene, hit.material_id).expect("scene hit referenced a missing material");
        radiance += throughput * visible_emissive_radiance(material, hit.front_face);

        if material.is_emissive() {
            break;
        }

        let albedo = material.diffuse_albedo();
        for light in emissive_triangles {
            radiance += throughput * evaluate_direct_light(scene, &hit, albedo, light);
        }

        throughput = throughput * albedo;
        if is_black(throughput) {
            break;
        }

        let bounce_direction = sample_cosine_weighted_hemisphere(hit.normal, rng);
        let bounce_origin = hit.position + hit.normal * SHADOW_BIAS;
        ray = Ray::new(bounce_origin, bounce_direction);
    }

    radiance
}

fn is_black(color: ColorRgb) -> bool {
    color == ColorRgb::BLACK
}

fn sample_cosine_weighted_hemisphere(normal: Vec3, rng: &mut PixelRng) -> Vec3 {
    let sample_a = rng.next_f32();
    let sample_b = rng.next_f32();
    let radius = sample_a.sqrt();
    let phi = 2.0 * std::f32::consts::PI * sample_b;

    let local_x = radius * phi.cos();
    let local_y = radius * phi.sin();
    let local_z = (1.0 - sample_a).sqrt();

    let tangent = build_tangent(normal);
    let bitangent = normal.cross(tangent).normalized();
    let direction = tangent * local_x + bitangent * local_y + normal * local_z;

    direction.normalized()
}

fn build_tangent(normal: Vec3) -> Vec3 {
    let reference = if normal.y.abs() < 0.999 {
        Vec3::Y
    } else {
        Vec3::X
    };

    normal.cross(reference).normalized()
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
        sample_cosine_weighted_hemisphere, trace_diffuse_path, CpuRendererBackend, PixelRng,
        SceneHit, DEPTH_MISS_COLOR, LIT_SAMPLES_PER_PIXEL, MIN_HIT_DISTANCE, MISS_COLOR,
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
        let metadata = backend.describe_render(
            &lit_room_scene(),
            sample_image_size(),
            RenderSettings::new(RenderMode::Lit, 6.0),
        );

        assert_eq!(metadata.backend_name, "cpu");
        assert_eq!(metadata.object_count, 7);
        assert_eq!(metadata.light_count, 2);
        assert_eq!(metadata.sample_count, LIT_SAMPLES_PER_PIXEL);
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
    fn path_trace_adds_indirect_bounce_when_light_is_hidden_from_first_hit() {
        let scene = indirect_bounce_scene();
        let lights = super::collect_emissive_triangles(&scene);
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.5), Vec3::new(0.0, 0.0, -1.0));
        let mut rng = PixelRng::new(2, 3);
        let mut color = ColorRgb::BLACK;

        for _sample_index in 0..256 {
            color += trace_diffuse_path(&scene, ray, &lights, &mut rng);
        }

        assert!(color.r > 0.0);
        assert!(color.g > 0.0);
        assert!(color.b > 0.0);
    }

    #[test]
    fn cosine_weighted_samples_stay_in_surface_hemisphere() {
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let mut rng = PixelRng::new(4, 5);

        for _ in 0..32 {
            let direction = sample_cosine_weighted_hemisphere(normal, &mut rng);
            assert!(direction.dot(normal) > 0.0);
        }
    }

    #[test]
    fn miss_color_matches_render_mode() {
        assert_eq!(miss_color(RenderMode::Lit), MISS_COLOR);
        assert_eq!(
            miss_color(RenderMode::Debug(RenderDebugMode::Depth)),
            DEPTH_MISS_COLOR
        );
    }

    #[test]
    fn color_conversion_clamps_and_scales() {
        let color = ColorRgb::new(1.2, 0.5, -0.2);

        assert_eq!(color_rgb_to_rgba8(color), ColorRgba8::new(255, 128, 0, 255));
    }

    #[test]
    fn find_material_returns_none_for_missing_material() {
        let scene = lit_room_scene();

        assert!(find_material(&scene, MaterialId(99)).is_none());
    }

    fn single_triangle_scene() -> SceneDescription {
        let camera = Camera::new(
            "main",
            Point3::new(0.0, 0.0, 2.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::Y,
            45.0,
        );
        let material_id = MaterialId(0);

        let mut scene = SceneDescription::new("single-triangle", camera);
        scene.materials.push(MaterialDescription::new(
            material_id,
            "gray",
            MaterialKind::Diffuse {
                albedo: ColorRgb::new(0.6, 0.6, 0.6),
                emission: ColorRgb::BLACK,
            },
        ));
        scene.objects.push(SceneObject::new(
            "triangle",
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
        let camera = Camera::new(
            "main",
            Point3::new(0.0, 0.0, 2.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::Y,
            45.0,
        );

        let matte = MaterialId(0);
        let light = MaterialId(1);
        let occluder = MaterialId(2);

        let mut scene = SceneDescription::new("simple-lighting", camera);
        scene.materials.push(MaterialDescription::new(
            matte,
            "matte",
            MaterialKind::Diffuse {
                albedo: ColorRgb::new(0.8, 0.8, 0.8),
                emission: ColorRgb::BLACK,
            },
        ));
        scene.materials.push(MaterialDescription::new(
            light,
            "light",
            MaterialKind::Diffuse {
                albedo: ColorRgb::BLACK,
                emission: ColorRgb::new(4.0, 4.0, 4.0),
            },
        ));
        scene.materials.push(MaterialDescription::new(
            occluder,
            "occluder",
            MaterialKind::Diffuse {
                albedo: ColorRgb::new(0.2, 0.2, 0.8),
                emission: ColorRgb::BLACK,
            },
        ));

        scene.objects.push(SceneObject::new(
            "receiver",
            Geometry::TriangleMesh {
                triangles: vec![Triangle::new(
                    Point3::new(-1.0, -1.0, 0.0),
                    Point3::new(1.0, -1.0, 0.0),
                    Point3::new(0.0, 1.0, 0.0),
                )],
            },
            matte,
        ));

        scene.objects.push(SceneObject::new(
            "light",
            Geometry::TriangleMesh {
                triangles: vec![Triangle::new(
                    Point3::new(-0.4, 0.4, 1.0),
                    Point3::new(0.4, 0.4, 1.0),
                    Point3::new(0.0, -0.4, 1.0),
                )],
            },
            light,
        ));

        if with_occluder {
            scene.objects.push(SceneObject::new(
                "occluder",
                Geometry::TriangleMesh {
                    triangles: vec![Triangle::new(
                        Point3::new(-0.2, -0.2, 0.5),
                        Point3::new(0.2, -0.2, 0.5),
                        Point3::new(0.0, 0.3, 0.5),
                    )],
                },
                occluder,
            ));
        }

        scene
    }

    fn indirect_bounce_scene() -> SceneDescription {
        let camera = Camera::new(
            "main",
            Point3::new(0.0, 0.0, 1.5),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::Y,
            45.0,
        );
        let receiver = MaterialId(0);
        let bounce = MaterialId(1);
        let light = MaterialId(2);
        let blocker = MaterialId(3);

        let mut scene = SceneDescription::new("indirect-bounce", camera);
        scene.materials.push(MaterialDescription::new(
            receiver,
            "receiver",
            MaterialKind::Diffuse {
                albedo: ColorRgb::new(0.8, 0.8, 0.8),
                emission: ColorRgb::BLACK,
            },
        ));
        scene.materials.push(MaterialDescription::new(
            bounce,
            "bounce",
            MaterialKind::Diffuse {
                albedo: ColorRgb::new(0.8, 0.2, 0.2),
                emission: ColorRgb::BLACK,
            },
        ));
        scene.materials.push(MaterialDescription::new(
            light,
            "light",
            MaterialKind::Diffuse {
                albedo: ColorRgb::BLACK,
                emission: ColorRgb::new(5.0, 5.0, 5.0),
            },
        ));
        scene.materials.push(MaterialDescription::new(
            blocker,
            "blocker",
            MaterialKind::Diffuse {
                albedo: ColorRgb::new(0.7, 0.7, 0.7),
                emission: ColorRgb::BLACK,
            },
        ));

        scene.objects.push(SceneObject::new(
            "receiver",
            Geometry::TriangleMesh {
                triangles: vec![Triangle::new(
                    Point3::new(-0.8, -0.8, 0.0),
                    Point3::new(0.8, -0.8, 0.0),
                    Point3::new(0.0, 0.8, 0.0),
                )],
            },
            receiver,
        ));

        scene.objects.push(SceneObject::new(
            "blocker",
            Geometry::TriangleMesh {
                triangles: vec![
                    Triangle::new(
                        Point3::new(-0.25, -0.25, 0.25),
                        Point3::new(0.25, -0.25, 0.25),
                        Point3::new(0.25, 0.25, 0.25),
                    ),
                    Triangle::new(
                        Point3::new(-0.25, -0.25, 0.25),
                        Point3::new(0.25, 0.25, 0.25),
                        Point3::new(-0.25, 0.25, 0.25),
                    ),
                ],
            },
            blocker,
        ));

        scene.objects.push(SceneObject::new(
            "bounce-wall",
            Geometry::TriangleMesh {
                triangles: vec![
                    Triangle::new(
                        Point3::new(0.9, -0.7, 0.8),
                        Point3::new(0.9, -0.7, -0.4),
                        Point3::new(0.9, 0.7, -0.4),
                    ),
                    Triangle::new(
                        Point3::new(0.9, -0.7, 0.8),
                        Point3::new(0.9, 0.7, -0.4),
                        Point3::new(0.9, 0.7, 0.8),
                    ),
                ],
            },
            bounce,
        ));

        scene.objects.push(SceneObject::new(
            "light",
            Geometry::TriangleMesh {
                triangles: vec![
                    Triangle::new(
                        Point3::new(0.6, -0.2, 0.75),
                        Point3::new(1.1, -0.2, 0.75),
                        Point3::new(1.1, 0.2, 0.75),
                    ),
                    Triangle::new(
                        Point3::new(0.6, -0.2, 0.75),
                        Point3::new(1.1, 0.2, 0.75),
                        Point3::new(0.6, 0.2, 0.75),
                    ),
                ],
            },
            light,
        ));

        scene
    }

    fn lit_room_scene() -> SceneDescription {
        let camera = Camera::new(
            "main",
            Point3::new(0.0, 0.0, 3.4),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::Y,
            40.0,
        );

        let red = MaterialId(0);
        let green = MaterialId(1);
        let white = MaterialId(2);
        let light = MaterialId(3);

        let mut scene = SceneDescription::new("lit-room", camera);
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
            "light",
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
