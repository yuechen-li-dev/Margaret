use margaret_core::camera::Camera;
use margaret_core::color::ColorRgb;
use margaret_core::image::ImageSize;
use margaret_core::light::Light;
use margaret_core::material::{MaterialDescription, MaterialId, MaterialKind};
use margaret_core::math::{Point3, Vec3};
use margaret_core::scene::{Geometry, SceneDescription, SceneObject};
use margaret_cpu::CpuRendererBackend;
use margaret_image::placeholder_image;

pub fn run() {
    let scene = placeholder_scene();
    let image_size = ImageSize::new(800, 600);
    let backend = CpuRendererBackend::new();
    let metadata = backend.describe_render(&scene, image_size);
    let image = placeholder_image(&metadata);

    println!("Margaret M0 scaffold");
    println!("scene: {}", metadata.scene_name);
    println!("backend: {}", metadata.backend_name);
    println!(
        "image: {}x{} {:?}",
        metadata.image_size.width, metadata.image_size.height, metadata.pixel_format
    );
    println!("samples: {}", metadata.sample_count);
    println!("objects: {}", metadata.object_count);
    println!("lights: {}", metadata.light_count);
    println!("placeholder pixels: {}", image.pixels.len());
}

fn placeholder_scene() -> SceneDescription {
    let camera = Camera::new(
        "main-camera",
        Point3::new(0.0, 1.5, 5.0),
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::Y,
        45.0,
    );
    let material_id = MaterialId(0);

    let mut scene = SceneDescription::new("m0-placeholder-scene", camera);
    scene.materials.push(MaterialDescription::new(
        material_id,
        "ground",
        MaterialKind::Diffuse {
            albedo: ColorRgb::new(0.7, 0.7, 0.7),
        },
    ));
    scene.objects.push(SceneObject::new(
        "hero-sphere",
        Geometry::Sphere {
            center: Point3::ORIGIN,
            radius: 1.0,
        },
        material_id,
    ));
    scene.lights.push(Light::Directional {
        direction: Vec3::new(-0.5, -1.0, -0.25),
        intensity: ColorRgb::WHITE,
    });
    scene
}

#[cfg(test)]
mod tests {
    use super::placeholder_scene;

    #[test]
    fn placeholder_scene_contains_one_object() {
        let scene = placeholder_scene();

        assert_eq!(scene.objects.len(), 1);
        assert_eq!(scene.lights.len(), 1);
    }
}
