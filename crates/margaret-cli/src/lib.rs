use std::path::Path;

use margaret_core::camera::Camera;
use margaret_core::image::ImageSize;
use margaret_core::material::{MaterialDescription, MaterialId, MaterialKind};
use margaret_core::math::{Point3, Vec3};
use margaret_core::scene::{Geometry, SceneDescription, SceneObject, Triangle};
use margaret_cpu::CpuRendererBackend;

pub fn run() -> std::io::Result<()> {
    let scene = hardcoded_scene();
    let image_size = ImageSize::new(320, 200);
    let backend = CpuRendererBackend::new();
    let metadata = backend.describe_render(&scene, image_size);
    let image = backend.render(&scene, image_size);
    let output_path = Path::new("margaret-m1a.ppm");

    image.write_ppm(output_path)?;

    println!("Margaret M1a CPU triangle render");
    println!("scene: {}", metadata.scene_name);
    println!("backend: {}", metadata.backend_name);
    println!(
        "image: {}x{} {:?}",
        metadata.image_size.width, metadata.image_size.height, metadata.pixel_format
    );
    println!("samples: {}", metadata.sample_count);
    println!("objects: {}", metadata.object_count);
    println!("lights: {}", metadata.light_count);
    println!("output: {}", output_path.display());

    Ok(())
}

fn hardcoded_scene() -> SceneDescription {
    let camera = Camera::new(
        "main-camera",
        Point3::new(0.0, 0.0, 2.5),
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::Y,
        45.0,
    );
    let material_id = MaterialId(0);

    let mut scene = SceneDescription::new("m1a-hardcoded-triangle-scene", camera);
    scene.materials.push(MaterialDescription::new(
        material_id,
        "debug-normals",
        MaterialKind::Diffuse {
            albedo: margaret_core::color::ColorRgb::WHITE,
        },
    ));
    scene.objects.push(SceneObject::new(
        "two-triangle-test-mesh",
        Geometry::TriangleMesh {
            triangles: vec![
                Triangle::new(
                    Point3::new(-0.9, -0.8, 0.0),
                    Point3::new(0.0, 0.9, 0.1),
                    Point3::new(0.9, -0.7, -0.2),
                ),
                Triangle::new(
                    Point3::new(-1.1, -0.9, -0.8),
                    Point3::new(1.0, -0.9, -0.8),
                    Point3::new(0.0, -0.2, 0.3),
                ),
            ],
        },
        material_id,
    ));
    scene
}

#[cfg(test)]
mod tests {
    use super::hardcoded_scene;
    use margaret_core::scene::Geometry;

    #[test]
    fn hardcoded_scene_contains_triangle_mesh() {
        let scene = hardcoded_scene();

        assert_eq!(scene.objects.len(), 1);
        assert_eq!(scene.lights.len(), 0);
        match &scene.objects[0].geometry {
            Geometry::TriangleMesh { triangles } => assert_eq!(triangles.len(), 2),
        }
    }
}
